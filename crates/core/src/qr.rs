//! QR PNG generator (D-14, D-15). ECC-L for max capacity; 2900-byte cap.

use image::Luma;
use qrcode::{EcLevel, QrCode};

use crate::CoreError;

pub const MAX_QR_BYTES: usize = 2900;

pub fn render_qr_png(armored: &str) -> Result<Vec<u8>, CoreError> {
    if armored.len() > MAX_QR_BYTES {
        return Err(CoreError::QrTooLarge {
            size: armored.len(),
            max: MAX_QR_BYTES,
        });
    }
    let code = QrCode::with_error_correction_level(armored.as_bytes(), EcLevel::L)
        .map_err(|_| CoreError::Crypto)?;
    let img = code.render::<Luma<u8>>().min_dimensions(256, 256).build();
    let mut buf = std::io::Cursor::new(Vec::<u8>::new());
    img.write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|_| CoreError::Crypto)?;
    Ok(buf.into_inner())
}
