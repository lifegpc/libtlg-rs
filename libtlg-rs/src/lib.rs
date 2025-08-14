//! A Rust library for processing TLG files.
#![cfg_attr(docsrs, feature(doc_cfg))]
mod load_tlg;
#[cfg(feature = "encode")]
mod save_tlg;
#[cfg(feature = "encode")]
mod slide;
mod stream;
#[cfg(feature = "encode")]
mod tlg5_saver;
mod tvpgl;
mod types;
use std::io::{Read, Seek};

pub use types::{Tlg, TlgColorType, TlgError};
/// The result type for TLG operations.
pub type Result<T> = std::result::Result<T, TlgError>;
pub use load_tlg::load_tlg;
#[cfg(feature = "encode")]
#[cfg_attr(docsrs, doc(cfg(feature = "encode")))]
pub use save_tlg::save_tlg;

/// Check if it's a valid TLG.
///
/// 11 bytes are needed.
pub fn is_valid_tlg(data: &[u8]) -> bool {
    if data.len() < 11 {
        return false;
    }
    data.starts_with(b"TLG0.0\x00sds\x1a")
        || data.starts_with(b"TLG5.0\x00raw\x1a")
        || data.starts_with(b"TLG6.0\x00raw\x1a")
}

/// Check if it's a valid TLG.
///
/// Same as [`is_valid_tlg`]
pub fn check_tlg<T: Read + Seek>(mut data: T) -> Result<bool> {
    let mut header = [0; 11];
    data.rewind()?;
    data.read_exact(&mut header)?;
    Ok(is_valid_tlg(&header))
}
