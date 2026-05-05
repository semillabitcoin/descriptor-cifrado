//! POST /api/encrypt — JSON in, JSON out (D-05). Three outputs in one response:
//! bed_b64 (base64 of binary), armored (PEM string), qr_png_b64 (base64 of PNG).

use axum::{extract::Json, response::IntoResponse};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

use crate::AppError;

#[derive(Deserialize)]
pub struct EncryptRequest {
    pub descriptor: String,
}

#[derive(Serialize)]
pub struct EncryptResponse {
    pub bed_b64: String,
    pub armored: String,
    pub qr_png_b64: String,
}

/// Encrypt handler. The `descriptor` field is moved into `Zeroizing<String>`
/// on the FIRST line (D-10, PITFALLS #4) before any `?` early-return.
#[tracing::instrument(skip_all)]
pub async fn post_encrypt(
    Json(req): Json<EncryptRequest>,
) -> Result<impl IntoResponse, AppError> {
    // STEP 1 (D-10): wrap immediately; req.descriptor is moved INTO Zeroizing
    // on this line. Any subsequent access is via &mut, never by value.
    let mut cleartext: Zeroizing<String> = Zeroizing::new(req.descriptor);

    let out = bed_core::encrypt_descriptor(&mut cleartext)?;

    // Defense-in-depth: explicit zeroize + drop before serialization.
    cleartext.zeroize();
    drop(cleartext);

    let response = EncryptResponse {
        bed_b64: STANDARD.encode(&out.bed_bytes),
        armored: out.armored,
        qr_png_b64: STANDARD.encode(&out.qr_png),
    };
    Ok(Json(response))
}
