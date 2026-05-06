//! AppState + helpers de configuración runtime para Phase 2 history endpoints.
//!
//! `data_dir()` resuelve el directorio de persistencia del historial desde
//! la env var `BED_DATA_DIR` (default `/data/encrypted`). Permite que los
//! integration tests usen `tempfile::tempdir()` sin colisionar con el path
//! productivo (Trampa 7 del RESEARCH).
//!
//! `validate_history_id()` enforces anti-path-traversal (D-29): id debe ser
//! exactamente 8 caracteres hex lowercase `[a-z0-9]{8}`.

use std::path::PathBuf;

#[derive(Clone, Default)]
pub struct AppState;

/// Resuelve el directorio donde se persisten los `.bed` del historial.
///
/// Default: `/data/encrypted` (StartOS volume `main`).
/// Override: env var `BED_DATA_DIR` (usado en dev y tests).
pub fn data_dir() -> PathBuf {
    std::env::var("BED_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/data/encrypted"))
}

/// Valida un id de historial: exactamente 8 caracteres hex lowercase.
/// Cualquier otro patrón (uppercase, longitud distinta, caracteres no-hex,
/// path traversal `../`, etc.) retorna false.
pub fn validate_history_id(id: &str) -> bool {
    id.len() == 8
        && id
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    use super::*;

    #[test]
    fn validate_history_id_accepts_8_hex_lowercase() {
        assert!(validate_history_id("a3f7b2c1"));
        assert!(validate_history_id("00000000"));
        assert!(validate_history_id("ffffffff"));
        assert!(validate_history_id("0123abcd"));
    }

    #[test]
    fn validate_history_id_rejects_uppercase() {
        assert!(!validate_history_id("A3F7B2C1"));
        assert!(!validate_history_id("a3F7b2c1"));
    }

    #[test]
    fn validate_history_id_rejects_wrong_length() {
        assert!(!validate_history_id(""));
        assert!(!validate_history_id("a3f7b2c")); // 7
        assert!(!validate_history_id("a3f7b2c1x")); // 9
        assert!(!validate_history_id("a3f7b2c1a3f7b2c1")); // 16
    }

    #[test]
    fn validate_history_id_rejects_non_hex() {
        assert!(!validate_history_id("a3f7b2g1")); // g no es hex
        assert!(!validate_history_id("a3f7-2c1")); // guión
        assert!(!validate_history_id("../etc/p")); // path traversal
        assert!(!validate_history_id("a3f7 2c1")); // espacio
    }

    #[test]
    fn data_dir_default() {
        // Saving any current value, set to none to test default
        let prev = std::env::var("BED_DATA_DIR").ok();
        std::env::remove_var("BED_DATA_DIR");
        assert_eq!(data_dir(), PathBuf::from("/data/encrypted"));
        if let Some(p) = prev {
            std::env::set_var("BED_DATA_DIR", p);
        }
    }
}
