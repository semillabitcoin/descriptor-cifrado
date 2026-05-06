//! AppError — single error type for the HTTP layer (D-16, D-17).
//!
//! Variants map to status codes:
//!   MissingMultipathWildcard | DescriptorParse | XpubMismatch | QrTooLarge → 422
//!   BadRequest(_) → 400
//!   Internal → 500
//!
//! Response body shape (D-17):
//!   {"error": {"code": "<UPPER_SNAKE>", "message": "<castellano>"}}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

use bed_core::CoreError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("El descriptor debe incluir derivación <0;1>/* en todas las claves. Sin esta derivación, gastar desde la primera dirección expone tu xpub on-chain.")]
    MissingMultipathWildcard,

    #[error("No se pudo parsear el descriptor.")]
    DescriptorParse,

    #[error("La xpub proporcionada no descifra este .bed (no es un cosigner válido).")]
    XpubMismatch,

    #[error("El descriptor cifrado excede capacidad QR ({size} > {max} bytes). Usa el archivo .bed o el armored.")]
    QrTooLarge { size: usize, max: usize },

    #[error("internal error")]
    Internal,

    #[error("solicitud inválida: {0}")]
    BadRequest(String),

    #[error("Entrada de historial no encontrada.")]
    HistoryNotFound,

    #[error("No se pudo escribir en el historial.")]
    HistoryWriteFailed,

    #[error("ID de historial inválido.")]
    HistoryInvalidId,
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

#[derive(Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code): (StatusCode, &'static str) = match &self {
            AppError::MissingMultipathWildcard => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "MISSING_MULTIPATH_WILDCARD",
            ),
            AppError::DescriptorParse => (StatusCode::UNPROCESSABLE_ENTITY, "DESCRIPTOR_PARSE"),
            AppError::XpubMismatch => (StatusCode::UNPROCESSABLE_ENTITY, "XPUB_MISMATCH"),
            AppError::QrTooLarge { .. } => (StatusCode::UNPROCESSABLE_ENTITY, "QR_TOO_LARGE"),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL"),
            AppError::HistoryNotFound => (StatusCode::NOT_FOUND, "HISTORY_NOT_FOUND"),
            AppError::HistoryWriteFailed => {
                (StatusCode::INTERNAL_SERVER_ERROR, "HISTORY_WRITE_FAILED")
            }
            AppError::HistoryInvalidId => (StatusCode::UNPROCESSABLE_ENTITY, "HISTORY_INVALID_ID"),
        };
        let body = ErrorEnvelope {
            error: ErrorBody {
                code,
                message: self.to_string(),
            },
        };
        (status, Json(body)).into_response()
    }
}

impl From<CoreError> for AppError {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::MissingMultipathWildcard => AppError::MissingMultipathWildcard,
            CoreError::DescriptorParse => AppError::DescriptorParse,
            CoreError::XpubMismatch => AppError::XpubMismatch,
            CoreError::QrTooLarge { size, max } => AppError::QrTooLarge { size, max },
            CoreError::Armored(msg) => AppError::BadRequest(msg),
            CoreError::Crypto => AppError::Internal,
        }
    }
}
