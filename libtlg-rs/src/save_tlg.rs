use super::*;
use crate::stream::*;
use crate::tlg5_saver::save_tlg5;
use std::io::{Seek, Write};

/// Encode TLG image
#[cfg_attr(docsrs, doc(cfg(feature = "encode")))]
pub fn save_tlg<W: Write + Seek>(img: &Tlg, mut writer: W) -> Result<()> {
    let colors = match img.color {
        TlgColorType::Bgra32 => 4,
        TlgColorType::Bgr24 => 3,
        TlgColorType::Grayscale8 => 1,
    };
    let img_size = img.width as usize * colors as usize * img.height as usize;
    if img.data.len() < img_size {
        return Err(TlgError::EncodeError(format!(
            "Image data size too small: expected {}, got {}",
            img_size,
            img.data.len()
        )));
    }
    if img.tags.is_empty() {
        if img.version == 5 {
            return save_tlg5(img, &mut writer);
        } else {
            return Err(TlgError::EncodeError(format!(
                "Unsupported TLG version: {}",
                img.version
            )));
        }
    }
    writer.write_all(b"TLG0.0\x00sds\x1a")?;
    let rawlenpos = writer.stream_position()?;
    writer.write_u32(0)?; // Placeholder for raw data length
    if img.version == 5 {
        save_tlg5(img, &mut writer)?;
    } else {
        return Err(TlgError::EncodeError(format!(
            "Unsupported TLG version: {}",
            img.version
        )));
    }
    let pos_save = writer.stream_position()?;
    writer.seek(std::io::SeekFrom::Start(rawlenpos))?;
    let size = pos_save - rawlenpos - 4;
    writer.write_u32(size as u32)?;
    writer.seek(std::io::SeekFrom::Start(pos_save))?;
    writer.write_all(b"tags")?;
    let mut ss = Vec::new();
    for (k, v) in &img.tags {
        ss.write_all(k.len().to_string().as_bytes())?;
        ss.write_all(b":")?;
        ss.write_all(k)?;
        ss.write_all(b"=")?;
        ss.write_all(v.len().to_string().as_bytes())?;
        ss.write_all(b":")?;
        ss.write_all(v)?;
        ss.write_all(b",")?;
    }
    writer.write_u32(ss.len() as u32)?;
    writer.write_all(&ss)?;
    Ok(())
}
