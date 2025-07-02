use crate::stream::ReadExt;
use crate::tvpgl::*;
use crate::*;
use overf::wrapping;
use std::io::SeekFrom;

fn load_tlg5<T: Read + Seek>(src: &mut T) -> Result<Tlg> {
    let colors = src.read_u8()?;
    let width = src.read_u32()?;
    let height = src.read_u32()?;
    let blockheight = src.read_u32()?;
    let color = match colors {
        3 => TlgColorType::Bgr24,
        4 => TlgColorType::Bgra32,
        _ => return Err(TlgError::UnsupportedColorType(colors)),
    };
    let blockcount = ((height - 1) / blockheight) + 1;
    src.seek_relative(blockcount as i64 * 4)?;
    let stride = width as usize * colors as usize;
    let mut output_data = vec![0u8; width as usize * height as usize * colors as usize];
    let mut text = [0u8; 4096];
    let mut inbuf = vec![0u8; blockheight as usize * width as usize + 10];
    let mut outbuf = vec![vec![0u8; blockheight as usize * width as usize + 10]; colors as usize];
    let mut prevline: Option<Vec<u8>> = None;
    let mut r = 0;
    for y_blk in (0..height).step_by(blockheight as usize) {
        for c in 0..colors {
            let mark = src.read_u8()?;
            let size = src.read_u32()?;
            if mark == 0 {
                src.read_exact(&mut inbuf[..size as usize])?;
                r = tlg5_decompress_slide(
                    &mut outbuf[c as usize],
                    &inbuf[..size as usize],
                    size as usize,
                    &mut text,
                    r,
                );
            } else {
                src.read_exact(&mut outbuf[c as usize][..size as usize])?;
            }
        }
        let y_lim = (y_blk + blockheight).min(height);
        let mut outbufp = Vec::new();
        for c in 0..colors {
            outbufp.push(outbuf[c as usize].as_slice());
        }
        for y in y_blk..y_lim {
            let current = &mut output_data[(y as usize * stride)..(y as usize * stride + stride)];
            match prevline.take() {
                Some(prev) => match color {
                    TlgColorType::Bgr24 => {
                        tlg5_compose_colors3(current, &prev, &outbufp, width);
                        outbufp[0] = &outbufp[0][width as usize..];
                        outbufp[1] = &outbufp[1][width as usize..];
                        outbufp[2] = &outbufp[2][width as usize..];
                    }
                    TlgColorType::Bgra32 => {
                        tlg5_compose_colors4(current, &prev, &outbufp, width);
                        outbufp[0] = &outbufp[0][width as usize..];
                        outbufp[1] = &outbufp[1][width as usize..];
                        outbufp[2] = &outbufp[2][width as usize..];
                        outbufp[3] = &outbufp[3][width as usize..];
                    }
                    _ => {}
                },
                None => match color {
                    TlgColorType::Bgra32 => {
                        let mut current_pos = 0usize;
                        let mut pr = 0u8;
                        let mut pg = 0u8;
                        let mut pb = 0u8;
                        let mut pa = 0u8;
                        for x in 0..width as usize {
                            let mut b = outbufp[0][x];
                            let g = outbufp[1][x];
                            let mut r = outbufp[2][x];
                            let a = outbufp[3][x];
                            wrapping! {
                                b += g;
                                r += g;
                                pb += b;
                                pg += g;
                                pr += r;
                                pa += a;
                            }
                            current[current_pos] = pb;
                            current_pos += 1;
                            current[current_pos] = pg;
                            current_pos += 1;
                            current[current_pos] = pr;
                            current_pos += 1;
                            current[current_pos] = pa;
                            current_pos += 1;
                        }
                        outbufp[0] = &outbufp[0][width as usize..];
                        outbufp[1] = &outbufp[1][width as usize..];
                        outbufp[2] = &outbufp[2][width as usize..];
                        outbufp[3] = &outbufp[3][width as usize..];
                    }
                    TlgColorType::Bgr24 => {
                        let mut current_pos = 0usize;
                        let mut pr = 0u8;
                        let mut pg = 0u8;
                        let mut pb = 0u8;
                        for x in 0..width as usize {
                            let mut b = outbufp[0][x];
                            let g = outbufp[1][x];
                            let mut r = outbufp[2][x];
                            wrapping! {
                                b += g;
                                r += g;
                                pb += b;
                                pg += g;
                                pr += r;
                            }
                            current[current_pos] = pb;
                            current_pos += 1;
                            current[current_pos] = pg;
                            current_pos += 1;
                            current[current_pos] = pr;
                            current_pos += 1;
                        }
                        outbufp[0] = &outbufp[0][width as usize..];
                        outbufp[1] = &outbufp[1][width as usize..];
                        outbufp[2] = &outbufp[2][width as usize..];
                    }
                    _ => {}
                },
            }
            prevline = Some(current.to_vec());
        }
    }
    Ok(Tlg {
        tags: Default::default(),
        version: 5,
        width,
        height,
        color,
        data: output_data,
    })
}

fn load_tlg6<T: Read + Seek>(_src: &mut T) -> Result<Tlg> {
    Err(TlgError::Str("TLG6 is not supported yet".to_string()))
}

fn internal_load_tlg<T: Read + Seek>(src: &mut T) -> Result<Tlg> {
    let mut mark = [0; 11];
    src.read_exact(&mut mark)?;
    if &mark == b"TLG5.0\x00raw\x1a" {
        load_tlg5(src)
    } else if &mark == b"TLG6.0\x00raw\x1a" {
        load_tlg6(src)
    } else {
        Err(TlgError::InvalidFormat)
    }
}

/// Decode TLG image
pub fn load_tlg<T: Read + Seek>(mut src: T) -> Result<Tlg> {
    src.rewind()?;
    let mut mark = [0; 11];
    src.read_exact(&mut mark)?;
    if &mark == b"TLG0.0\x00sds\x1a" {
        let rawlen = src.read_u32()?;
        let mut tlg = internal_load_tlg(&mut src)?;
        let newlen = rawlen as u64 + 15;
        src.seek(SeekFrom::Start(newlen))?;
        let mut check = true;
        while check {
            let mut chunkname = [0; 4];
            if src.read(&mut chunkname)? != 4 {
                break;
            }
            let chunksize = src.read_u32()?;
            if &chunkname == b"tags" {
                let mut tag = vec![0; chunksize as usize];
                src.read_exact(&mut tag)?;
                let mut i = 0;
                let len = tag.len();
                while i < len {
                    let mut namelen = 0usize;
                    let mut c = tag[i];
                    let mut ok = true;
                    while c >= b'0' && c <= b'9' {
                        namelen = namelen * 10 + (c - b'0') as usize;
                        i += 1;
                        if i >= len {
                            ok = false;
                            break;
                        }
                        c = tag[i];
                    }
                    if !ok {
                        break;
                    }
                    if c != b':' {
                        check = false;
                        break;
                    }
                    i += 1;
                    let name = tag[i..i + namelen].to_vec();
                    i += namelen;
                    if i >= len {
                        break;
                    }
                    let mut valuelen = 0usize;
                    c = tag[i];
                    ok = true;
                    while c >= b'0' && c <= b'9' {
                        valuelen = valuelen * 10 + (c - b'0') as usize;
                        i += 1;
                        if i >= len {
                            ok = false;
                            break;
                        }
                        c = tag[i];
                    }
                    if !ok {
                        break;
                    }
                    if c != b':' {
                        check = false;
                        break;
                    }
                    i += 1;
                    let value = tag[i..i + valuelen].to_vec();
                    i += valuelen;
                    if i >= len {
                        check = false;
                        break;
                    }
                    c = tag[i];
                    if c != b',' {
                        check = false;
                        break;
                    }
                    i += 1;
                    tlg.tags.insert(name, value);
                }
            } else {
                // skip the chunk
                src.seek_relative(chunksize as i64)?;
            }
        }
        Ok(tlg)
    } else {
        src.rewind()?;
        internal_load_tlg(&mut src)
    }
}
