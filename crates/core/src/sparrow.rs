//! Soporte BIP329 JSONL exportado por Sparrow Wallet.
//!
//! ## Caveats (power-user, no UI)
//!
//! 1. Las xpubs deben tener label en Sparrow **antes** del export (Sparrow solo emite
//!    `type:"xpub"` para entradas etiquetadas). Sin labels → 0 xpubs → error.
//! 2. MVP cubre el primer wallet encontrado (primer `origin` detectado).
//! 3. Flujo de re-import asumido: usuario crea wallet en Sparrow con el descriptor
//!    compuesto, luego importa el JSONL → las labels cargan vía matching de `origin`.

use serde::Deserialize;

use crate::CoreError;

/// Entry mínima de un BIP329 JSONL Sparrow export.
/// Solo extraemos los campos que necesitamos; el resto se ignora.
#[derive(Deserialize)]
struct LabelEntry {
    #[serde(rename = "type")]
    entry_type: String,
    #[serde(rename = "ref")]
    ref_field: String,
    /// Solo presente en entradas no-xpub (`addr`, `tx`, `output`, `input`).
    origin: Option<String>,
    // `label` no se necesita para la composición.
}

/// Datos parseados de un JSONL Sparrow export.
pub(crate) struct SparrowData {
    /// xpubs en orden de aparición en el JSONL (= orden de `wallet.getKeystores()`).
    pub(crate) xpubs: Vec<String>,
    /// Template descriptor sin xpubs sustituidas (p.ej. `wsh(sortedmulti(2,[fp/path],...))`).
    pub(crate) origin: String,
}

/// Parsea un string JSONL BIP329 Sparrow y extrae xpubs + origin template.
///
/// - Líneas vacías se ignoran.
/// - Cualquier línea con JSON inválido devuelve `Err(CoreError::DescriptorParse)`.
/// - Si no hay ≥1 xpub o no hay origin → `Err(CoreError::DescriptorParse)`.
pub(crate) fn parse_sparrow_jsonl(jsonl: &str) -> Result<SparrowData, CoreError> {
    let mut xpubs: Vec<String> = Vec::new();
    let mut origin: Option<String> = None;

    for line in jsonl.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let entry: LabelEntry =
            serde_json::from_str(trimmed).map_err(|_| CoreError::DescriptorParse)?;

        match entry.entry_type.as_str() {
            "xpub" => {
                xpubs.push(entry.ref_field);
            }
            _ => {
                // addr / tx / output / input — tomar el primer origin encontrado
                if origin.is_none() {
                    if let Some(o) = entry.origin {
                        origin = Some(o);
                    }
                }
            }
        }
    }

    if xpubs.is_empty() {
        return Err(CoreError::DescriptorParse);
    }
    let origin_str = origin.ok_or(CoreError::DescriptorParse)?;

    Ok(SparrowData {
        xpubs,
        origin: origin_str,
    })
}

/// Compone el descriptor canónico a partir de los datos parseados.
///
/// Algoritmo: recorre `origin` carácter a carácter, sustituye cada slot `[...]`
/// por `[fp/path]xpubN/<0;1>/*` donde xpubN es la siguiente xpub de la lista.
///
/// Postcondición: número de slots `[...]` == número de xpubs.
/// Mismatch → `Err(CoreError::DescriptorParse)`.
pub(crate) fn compose_descriptor_from_sparrow(data: &SparrowData) -> Result<String, CoreError> {
    let origin = &data.origin;
    let xpubs = &data.xpubs;

    let mut result = String::with_capacity(origin.len() + xpubs.iter().map(|x| x.len() + 10).sum::<usize>());
    let mut xpub_idx = 0usize;
    let chars: Vec<char> = origin.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '[' {
            // Encontrado inicio de slot — buscar el cierre `]`
            let slot_start = i;
            let mut depth = 0usize;
            while i < chars.len() {
                if chars[i] == '[' {
                    depth += 1;
                }
                if chars[i] == ']' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                i += 1;
            }
            // chars[i] == ']' ahora (o fin de string si malformado)
            if i >= chars.len() || chars[i] != ']' {
                return Err(CoreError::DescriptorParse);
            }
            // Emitir `[fp/path]`
            let slot: String = chars[slot_start..=i].iter().collect();
            result.push_str(&slot);
            // Emitir xpub correspondiente
            if xpub_idx >= xpubs.len() {
                return Err(CoreError::DescriptorParse);
            }
            result.push_str(&xpubs[xpub_idx]);
            result.push_str("/<0;1>/*");
            xpub_idx += 1;
        } else {
            result.push(chars[i]);
        }
        i += 1;
    }

    // Verificar que usamos todas las xpubs (no menos slots que xpubs)
    if xpub_idx != xpubs.len() {
        return Err(CoreError::DescriptorParse);
    }

    Ok(result)
}

/// Helper público: dado un cleartext, devuelve `Some(descriptor_compuesto)` si
/// el input es un JSONL Sparrow BIP329 válido, o `None` en cualquier otro caso.
///
/// Nunca hace panic. Cualquier error interno → `None`.
///
/// ## Uso en decrypt-side
///
/// El servidor llama a esta función sobre el cleartext descifrado para poblar
/// `composed_descriptor` en la respuesta JSON. La UI usa ese campo para mostrar
/// el descriptor listo para reimportar en Sparrow.
///
/// ## Caveats
///
/// 1. xpubs deben estar etiquetadas en Sparrow antes del export.
/// 2. MVP cubre primer wallet.
/// 3. Re-import: crear wallet con descriptor compuesto → importar JSONL.
pub fn compose_descriptor_if_sparrow_jsonl(cleartext: &str) -> Option<String> {
    let parsed = parse_sparrow_jsonl(cleartext).ok()?;
    compose_descriptor_from_sparrow(&parsed).ok()
}
