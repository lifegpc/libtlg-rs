use super::*;
use crate::slide::*;
use crate::stream::*;
use overf::wrapping;
use std::io::{Seek, Write};

const BLOCK_HEIGHT: usize = 4;

pub fn save_tlg5<W: Write + Seek>(tlg: &Tlg, writer: &mut W) -> Result<()> {
    writer.write_all(b"TLG5.0\x00raw\x1a")?;
    let colors = match tlg.color {
        TlgColorType::Bgra32 => 4,
        TlgColorType::Bgr24 => 3,
        TlgColorType::Grayscale8 => 1,
    };
    writer.write_u8(colors)?;
    writer.write_u32(tlg.width)?;
    writer.write_u32(tlg.height)?;
    writer.write_u32(BLOCK_HEIGHT as u32)?;
    let blockcount = ((tlg.height as usize - 1) / BLOCK_HEIGHT) + 1;
    let mut compressor = SlideCompressor::new();
    let mut written = [0; 4];
    let mut blocksizes = vec![0; blockcount];
    let mut cmpinbuf = vec![vec![0u8; tlg.width as usize * BLOCK_HEIGHT]; colors as usize];
    let blocksizepos = writer.stream_position()?;
    for _ in 0..blockcount {
        writer.write_all(b"    ")?; // Place holders
    }
    let mut block = 0;
    for blk_y in (0..tlg.height as usize).step_by(BLOCK_HEIGHT) {
        let ylim = (blk_y + BLOCK_HEIGHT).min(tlg.height as usize);
        let mut inp = 0;
        for y in blk_y..ylim {
            let upper = if y != 0 {
                &tlg.data[(y - 1) * tlg.width as usize * colors as usize
                    ..y * tlg.width as usize * colors as usize]
            } else {
                &[]
            };
            let mut upper_pos = 0;
            let current = &tlg.data[y * tlg.width as usize * colors as usize
                ..(y + 1) * tlg.width as usize * colors as usize];
            let mut current_pos = 0;
            let mut prevcl = [0; 4];
            let mut val = [0; 4];
            for _ in 0..tlg.width as usize {
                for c in 0..colors as usize {
                    let cl = if y != 0 {
                        let c = current[current_pos];
                        current_pos += 1;
                        let p = upper[upper_pos];
                        upper_pos += 1;
                        wrapping! { c - p }
                    } else {
                        let c = current[current_pos];
                        current_pos += 1;
                        c
                    } as i32;
                    val[c] = wrapping! { cl - prevcl[c] };
                    prevcl[c] = cl;
                }
                if colors == 1 {
                    cmpinbuf[0][inp] = val[0] as u8;
                } else if colors == 3 {
                    cmpinbuf[0][inp] = wrapping! { val[0] - val[1] } as u8;
                    cmpinbuf[1][inp] = val[1] as u8;
                    cmpinbuf[2][inp] = wrapping! { val[2] - val[1] } as u8;
                } else if colors == 4 {
                    cmpinbuf[0][inp] = wrapping! { val[0] - val[1] } as u8;
                    cmpinbuf[1][inp] = val[1] as u8;
                    cmpinbuf[2][inp] = wrapping! { val[2] - val[1] } as u8;
                    cmpinbuf[3][inp] = val[3] as u8;
                }
                inp += 1;
            }
        }
        // LZSS
        let mut blocksize = 0;
        for c in 0..colors as usize {
            compressor.store();
            let mut outbuf = Vec::new();
            let wrote = compressor.encode_into(&cmpinbuf[c][..inp], &mut outbuf);
            if wrote < inp {
                writer.write_u8(0)?;
                writer.write_u32(wrote as u32)?;
                writer.write_all(&outbuf)?;
                blocksize += wrote + 4 + 1;
            } else {
                compressor.restore();
                writer.write_u8(1)?;
                writer.write_u32(inp as u32)?;
                writer.write_all(&cmpinbuf[c][..inp])?;
                blocksize += inp + 4 + 1;
            }
            written[c] += wrote;
        }
        blocksizes[block] = blocksize;
        block += 1;
    }
    let pos_save = writer.stream_position()?;
    writer.seek(std::io::SeekFrom::Start(blocksizepos))?;
    for i in 0..blockcount {
        writer.write_u32(blocksizes[i] as u32)?;
    }
    writer.seek(std::io::SeekFrom::Start(pos_save))?;
    Ok(())
}
