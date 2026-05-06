//! PEM-style armored encoder/decoder for .bed binary payloads (D-12).
//!
//! The bitcoin-encrypted-backup crate provides only base64 (single line via
//! `encrypt_base64`); it does NOT provide PEM headers. This module wraps
//! base64 with `-----BEGIN/END BITCOIN ENCRYPTED BACKUP-----` and line-wrap.

use base64::{engine::general_purpose::STANDARD, Engine as _};

pub const ARMOR_BEGIN: &str = "-----BEGIN BITCOIN ENCRYPTED BACKUP-----";
pub const ARMOR_END: &str = "-----END BITCOIN ENCRYPTED BACKUP-----";
const LINE_WIDTH: usize = 64;

pub fn encode_armored(bed_bytes: &[u8]) -> String {
    let b64 = STANDARD.encode(bed_bytes);
    let mut out = String::with_capacity(b64.len() + 128);
    out.push_str(ARMOR_BEGIN);
    out.push('\n');
    for chunk in b64.as_bytes().chunks(LINE_WIDTH) {
        // base64 alphabet is pure ASCII → from_utf8 cannot fail; use a defensive
        // fallback that returns empty rather than panic (lint forbids unwrap/expect).
        let s = std::str::from_utf8(chunk).unwrap_or("");
        out.push_str(s);
        out.push('\n');
    }
    out.push_str(ARMOR_END);
    out.push('\n');
    out
}

#[derive(thiserror::Error, Debug)]
pub enum ArmoredError {
    #[error("missing or wrong BEGIN header")]
    WrongHeader,
    #[error("missing or wrong END footer")]
    WrongFooter,
    #[error("empty payload between headers")]
    EmptyPayload,
    #[error("invalid base64 payload")]
    Base64,
}

impl From<ArmoredError> for crate::CoreError {
    fn from(e: ArmoredError) -> Self {
        crate::CoreError::Armored(e.to_string())
    }
}

pub fn decode_armored(input: &str) -> Result<Vec<u8>, ArmoredError> {
    // Strip BOM if present
    let s = input.strip_prefix('\u{FEFF}').unwrap_or(input);
    let mut payload_lines: Vec<&str> = Vec::new();
    let mut in_block = false;
    for raw_line in s.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("-----BEGIN") {
            if line == ARMOR_BEGIN {
                in_block = true;
                continue;
            }
            return Err(ArmoredError::WrongHeader);
        }
        if line.starts_with("-----END") {
            if line == ARMOR_END {
                break;
            }
            return Err(ArmoredError::WrongFooter);
        }
        if in_block {
            payload_lines.push(line);
        }
    }
    if payload_lines.is_empty() {
        return Err(ArmoredError::EmptyPayload);
    }
    let joined: String = payload_lines.concat();
    STANDARD
        .decode(joined.as_bytes())
        .map_err(|_| ArmoredError::Base64)
}
