//! Handlers para el modo histórico opt-in (Phase 2).
//!
//! Endpoints (D-26..D-29):
//!   POST   /api/history          — persiste un .bed
//!   GET    /api/history          — lista entradas (directory scan)
//!   GET    /api/history/{id}     — regenera bed/armored/qr desde el .bed persistido
//!   DELETE /api/history/{id}     — borra una entrada
//!
//! HIST-03 (no leak): el endpoint POST acepta SOLO `bed_b64` ya cifrado, jamás
//! el descriptor en claro. El módulo no tiene ninguna ruta de código que
//! reciba o escriba descriptors cleartext.

use axum::{extract::Path, http::StatusCode, Json};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use time::{format_description::FormatItem, macros::format_description, OffsetDateTime};
use tokio::fs;
use uuid::Uuid;

use crate::{
    error::AppError,
    state::{data_dir, validate_history_id},
};

// === Request/Response shapes ===

#[derive(Deserialize)]
pub struct PostHistoryRequest {
    pub bed_b64: String,
    pub label: String,
}

#[derive(Serialize)]
pub struct PostHistoryResponse {
    pub id: String,
    pub timestamp: String,
    pub filename: String,
    pub label: String,
}

#[derive(Serialize)]
pub struct HistoryEntry {
    pub id: String,
    pub timestamp: String,
    pub filename: String,
    pub size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Serialize)]
pub struct ListHistoryResponse {
    pub entries: Vec<HistoryEntry>,
}

#[derive(Serialize)]
pub struct GetHistoryIdResponse {
    pub bed_b64: String,
    pub armored: String,
    pub qr_png_b64: String,
}

// === Filename format helpers ===

const FILENAME_COMPACT: &[FormatItem<'_>] =
    format_description!("[year][month][day]T[hour][minute][second]Z");
const FILENAME_ISO: &[FormatItem<'_>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");

fn now_compact() -> Result<String, AppError> {
    OffsetDateTime::now_utc()
        .format(FILENAME_COMPACT)
        .map_err(|_| AppError::Internal)
}

fn now_iso() -> Result<String, AppError> {
    OffsetDateTime::now_utc()
        .format(FILENAME_ISO)
        .map_err(|_| AppError::Internal)
}

/// Parsea filename del historial. Soporta dos formatos:
///   `20260506T115537Z-a3f7b2c1.bed`               → label=None (legacy v0.1.x)
///   `20260506T115537Z-a3f7b2c1-Mi multisig.bed`   → label=Some
/// Retorna None si el filename no matchea ningún formato.
fn parse_filename(name: &str) -> Option<(String, String, Option<String>)> {
    let stripped = name.strip_suffix(".bed")?;
    if stripped.len() < 25 {
        return None;
    }
    let compact = &stripped[0..16];
    if compact.as_bytes().get(8) != Some(&b'T') || compact.as_bytes().get(15) != Some(&b'Z') {
        return None;
    }
    let date = &compact[0..8];
    let time = &compact[9..15];
    if !date.chars().all(|c| c.is_ascii_digit()) || !time.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if stripped.as_bytes().get(16) != Some(&b'-') {
        return None;
    }
    let id = &stripped[17..25];
    if !validate_history_id(id) {
        return None;
    }
    let label = if stripped.len() == 25 {
        None
    } else if stripped.as_bytes().get(25) == Some(&b'-') && stripped.len() > 26 {
        Some(stripped[26..].to_string())
    } else {
        return None;
    };
    let iso = format!(
        "{}-{}-{}T{}:{}:{}Z",
        &date[0..4],
        &date[4..6],
        &date[6..8],
        &time[0..2],
        &time[2..4],
        &time[4..6],
    );
    Some((iso, id.to_string(), label))
}

fn make_filename(compact: &str, id: &str, label: Option<&str>) -> String {
    match label {
        Some(l) => format!("{compact}-{id}-{l}.bed"),
        None => format!("{compact}-{id}.bed"),
    }
}

/// Sanitize user-supplied label for safe embedding in filename.
/// Charset permitido: `[a-zA-Z0-9 _-]`. Otros chars → `-`. Trim. Max 80.
/// Devuelve None si tras sanitizar queda vacío o solo whitespace/dashes.
fn sanitize_label(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut out = String::with_capacity(trimmed.len().min(80));
    for ch in trimmed.chars() {
        if out.chars().count() >= 80 {
            break;
        }
        if ch.is_ascii_alphanumeric() || ch == ' ' || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    let cleaned = out.trim().to_string();
    if cleaned.is_empty() || cleaned.chars().all(|c| c == '-') {
        None
    } else {
        Some(cleaned)
    }
}

fn full_path(filename: &str) -> PathBuf {
    data_dir().join(filename)
}

async fn find_file_by_id(id: &str) -> Result<Option<PathBuf>, AppError> {
    let dir = data_dir();
    let mut rd = match fs::read_dir(&dir).await {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(AppError::Internal),
    };
    while let Some(entry) = rd.next_entry().await.map_err(|_| AppError::Internal)? {
        let name = entry.file_name();
        let Some(s) = name.to_str() else { continue };
        if let Some((_ts, parsed_id, _label)) = parse_filename(s) {
            if parsed_id == id {
                return Ok(Some(entry.path()));
            }
        }
    }
    Ok(None)
}

// === Handlers ===

/// `POST /api/history` — persiste un .bed cifrado.
#[tracing::instrument(skip_all)]
pub async fn post_history(
    Json(req): Json<PostHistoryRequest>,
) -> Result<Json<PostHistoryResponse>, AppError> {
    let bytes = B64
        .decode(req.bed_b64.as_bytes())
        .map_err(|_| AppError::BadRequest("bed_b64 no es base64 válido".to_string()))?;
    if bytes.is_empty() {
        return Err(AppError::BadRequest("bed_b64 está vacío".to_string()));
    }
    let label = sanitize_label(&req.label).ok_or_else(|| {
        AppError::BadRequest(
            "label requerido: usa letras, números, espacios, guiones o guiones bajos".to_string(),
        )
    })?;
    let id: String = Uuid::new_v4().simple().to_string()[..8].to_string();
    let compact = now_compact()?;
    let iso = now_iso()?;
    let filename = make_filename(&compact, &id, Some(&label));
    let path = full_path(&filename);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|_| AppError::HistoryWriteFailed)?;
    }
    fs::write(&path, &bytes)
        .await
        .map_err(|_| AppError::HistoryWriteFailed)?;
    Ok(Json(PostHistoryResponse {
        id,
        timestamp: iso,
        filename,
        label,
    }))
}

/// `GET /api/history` — lista entradas via directory scan.
#[tracing::instrument(skip_all)]
pub async fn get_history() -> Result<Json<ListHistoryResponse>, AppError> {
    let dir = data_dir();
    let mut rd = match fs::read_dir(&dir).await {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Json(ListHistoryResponse { entries: vec![] }));
        }
        Err(_) => return Err(AppError::Internal),
    };
    let mut entries = Vec::new();
    while let Some(entry) = rd.next_entry().await.map_err(|_| AppError::Internal)? {
        let name = entry.file_name();
        let Some(s) = name.to_str() else { continue };
        let Some((timestamp, id, label)) = parse_filename(s) else {
            continue;
        };
        let size_bytes = entry.metadata().await.map(|m| m.len()).unwrap_or(0);
        entries.push(HistoryEntry {
            id,
            timestamp,
            filename: s.to_string(),
            size_bytes,
            label,
        });
    }
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(Json(ListHistoryResponse { entries }))
}

/// `GET /api/history/{id}` — regenera bed_b64 + armored + qr_png_b64.
#[tracing::instrument(skip_all)]
pub async fn get_history_id(
    Path(id): Path<String>,
) -> Result<Json<GetHistoryIdResponse>, AppError> {
    if !validate_history_id(&id) {
        return Err(AppError::HistoryInvalidId);
    }
    let path = find_file_by_id(&id)
        .await?
        .ok_or(AppError::HistoryNotFound)?;
    let bytes = fs::read(&path).await.map_err(|_| AppError::Internal)?;
    let bed_b64 = B64.encode(&bytes);
    let armored = bed_core::encode_armored(&bytes);
    let qr_png = bed_core::render_qr_png(&armored)?;
    let qr_png_b64 = B64.encode(&qr_png);
    Ok(Json(GetHistoryIdResponse {
        bed_b64,
        armored,
        qr_png_b64,
    }))
}

/// `DELETE /api/history/{id}` — borra una entrada.
#[tracing::instrument(skip_all)]
pub async fn delete_history(Path(id): Path<String>) -> Result<StatusCode, AppError> {
    if !validate_history_id(&id) {
        return Err(AppError::HistoryInvalidId);
    }
    let path = find_file_by_id(&id)
        .await?
        .ok_or(AppError::HistoryNotFound)?;
    fs::remove_file(&path)
        .await
        .map_err(|_| AppError::Internal)?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    use super::*;

    #[test]
    fn parse_filename_legacy_round_trip() {
        let id = "a3f7b2c1";
        let compact = "20260506T115537Z";
        let name = make_filename(compact, id, None);
        assert_eq!(name, "20260506T115537Z-a3f7b2c1.bed");
        let (iso, parsed_id, label) = parse_filename(&name).expect("parse should succeed");
        assert_eq!(iso, "2026-05-06T11:55:37Z");
        assert_eq!(parsed_id, id);
        assert_eq!(label, None);
    }

    #[test]
    fn parse_filename_with_label_round_trip() {
        let id = "a3f7b2c1";
        let compact = "20260506T115537Z";
        let name = make_filename(compact, id, Some("Mi multisig"));
        assert_eq!(name, "20260506T115537Z-a3f7b2c1-Mi multisig.bed");
        let (iso, parsed_id, label) = parse_filename(&name).expect("parse should succeed");
        assert_eq!(iso, "2026-05-06T11:55:37Z");
        assert_eq!(parsed_id, id);
        assert_eq!(label.as_deref(), Some("Mi multisig"));
    }

    #[test]
    fn parse_filename_rejects_bad_format() {
        assert!(parse_filename("not-a-bed-file.txt").is_none());
        assert!(parse_filename("garbage.bed").is_none());
        // BADID0X is 7 chars after dash → wrong length for id (8 expected)
        assert!(parse_filename("20260506T115537Z-BADID0X.bed").is_none());
        // uppercase id
        assert!(parse_filename("20260506T115537Z-A3F7B2C1.bed").is_none());
        // wrong compact length
        assert!(parse_filename("20260506T11553Z-a3f7b2c1.bed").is_none());
        // missing T
        assert!(parse_filename("20260506X115537Z-a3f7b2c1.bed").is_none());
        // label vacío tras separador
        assert!(parse_filename("20260506T115537Z-a3f7b2c1-.bed").is_none());
    }

    #[test]
    fn sanitize_label_valid() {
        assert_eq!(
            sanitize_label("Mi multisig 3 de 5").as_deref(),
            Some("Mi multisig 3 de 5")
        );
        assert_eq!(sanitize_label("backup_2026-05").as_deref(), Some("backup_2026-05"));
    }

    #[test]
    fn sanitize_label_empty_or_whitespace() {
        assert_eq!(sanitize_label(""), None);
        assert_eq!(sanitize_label("   "), None);
    }

    #[test]
    fn sanitize_label_replaces_invalid_chars() {
        // tildes/acentos no-ASCII y símbolos → '-'; 'o' ASCII se preserva
        assert_eq!(sanitize_label("Ñoño@!").as_deref(), Some("-o-o--"));
        // mezcla válido + inválido
        assert_eq!(sanitize_label("a/b\\c").as_deref(), Some("a-b-c"));
    }

    #[test]
    fn sanitize_label_all_dashes_after_sanitize_returns_none() {
        // Tras sanitizar todos los chars devienen '-' → None
        assert_eq!(sanitize_label("@@@"), None);
        assert_eq!(sanitize_label("///"), None);
    }

    #[test]
    fn sanitize_label_caps_at_80_chars() {
        let long = "a".repeat(120);
        let result = sanitize_label(&long).expect("válido");
        assert_eq!(result.chars().count(), 80);
    }

    #[test]
    fn sanitize_label_trims_whitespace() {
        assert_eq!(sanitize_label("  hello  ").as_deref(), Some("hello"));
    }
}
