use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// TLG Color Type
pub enum TlgColorType {
    /// Grayscale 8-bit
    Grayscale8,
    /// BGR 8-bit
    Bgr24,
    /// BGRA 8-bit
    Bgra32,
}

#[derive(Debug, Clone)]
/// TLG Image
pub struct Tlg {
    /// Tag dictionary
    pub tags: HashMap<Vec<u8>, Vec<u8>>,
    /// TLG Version: 0=unknown, 5=v5, 6=v6
    pub version: u32,
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
    /// Color type
    pub color: TlgColorType,
    /// Image data
    pub data: Vec<u8>,
}

#[derive(Debug)]
/// TLG Error
pub enum TlgError {
    /// IO Error
    Io(std::io::Error),
    /// Invalid TLG format
    InvalidFormat,
    /// Unsupported color type
    UnsupportedColorType(u8),
    /// String type error
    Str(String),
}

impl std::fmt::Display for TlgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TlgError::Io(e) => write!(f, "IO Error: {}", e),
            TlgError::InvalidFormat => write!(f, "Invalid TLG format"),
            TlgError::UnsupportedColorType(c) => write!(f, "Unsupported color type: {}", c),
            TlgError::Str(s) => write!(f, "{}", s),
        }
    }
}

impl From<std::io::Error> for TlgError {
    fn from(err: std::io::Error) -> Self {
        TlgError::Io(err)
    }
}

impl From<String> for TlgError {
    fn from(err: String) -> Self {
        TlgError::Str(err)
    }
}

impl From<&str> for TlgError {
    fn from(err: &str) -> Self {
        TlgError::Str(err.to_string())
    }
}

impl From<&String> for TlgError {
    fn from(err: &String) -> Self {
        TlgError::Str(err.clone())
    }
}

impl std::error::Error for TlgError {}
