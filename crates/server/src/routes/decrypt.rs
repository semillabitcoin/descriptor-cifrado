//! POST /api/decrypt — multipart in, JSON out (D-06).
//!
//! Fields:
//!   bed:  text armored OR binary .bed file (auto-detected by leading bytes)
//!   xpub: text xpub OR file containing xpub (multipart treats both as bytes)
//!
//! NOTE: ENC-02/ENC-05/DEC-02/DEC-03 are API substrate only — UI presentation
//! is Phase 2. This handler accepts both pasted-text and uploaded-file via the
//! same multipart contract.

use axum::{extract::Multipart, Json};
use serde::Serialize;
use zeroize::{Zeroize, Zeroizing};

use crate::AppError;

#[derive(Serialize)]
pub struct DecryptResponse {
    pub descriptor: String,
}

#[tracing::instrument(skip_all)]
pub async fn post_decrypt(mut form: Multipart) -> Result<Json<DecryptResponse>, AppError> {
    let mut bed: Option<Vec<u8>> = None;
    let mut xpub: Option<String> = None;

    while let Some(field) = form
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "bed" => {
                let raw = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                bed = Some(raw.to_vec());
            }
            "xpub" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                xpub = Some(text.trim().to_string());
            }
            _ => { /* ignore unknown fields */ }
        }
    }

    let bed_bytes = bed.ok_or_else(|| AppError::BadRequest("missing 'bed' field".to_string()))?;
    let xpub_str = xpub.ok_or_else(|| AppError::BadRequest("missing 'xpub' field".to_string()))?;

    // Auto-detect armored: if bytes start with "-----BEGIN", strip PEM headers
    // via decode_armored. Otherwise pass raw bytes to crate (which auto-detects
    // binary "BIPXXX" magic vs raw base64).
    let payload: Vec<u8> = if bed_bytes.starts_with(b"-----BEGIN") {
        let text = std::str::from_utf8(&bed_bytes)
            .map_err(|_| AppError::BadRequest("invalid utf-8 in armored".to_string()))?;
        bed_core::decode_armored(text).map_err(|e| AppError::BadRequest(format!("armored: {e}")))?
    } else {
        bed_bytes
    };

    let mut cleartext: Zeroizing<String> = bed_core::decrypt_payload(&payload, &xpub_str)?;

    // Snapshot the cleartext to a String for JSON serialization. This is the
    // documented residual exposure (RESEARCH.md note in §Pattern 6): once it
    // crosses the JSON boundary it cannot be zeroized in serde's intermediate
    // buffer. We zeroize the source Zeroizing immediately after the clone.
    let descriptor = cleartext.as_str().to_string();
    cleartext.zeroize();
    drop(cleartext);

    Ok(Json(DecryptResponse { descriptor }))
}
