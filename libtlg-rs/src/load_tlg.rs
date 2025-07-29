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
        1 => TlgColorType::Grayscale8,
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
                    TlgColorType::Grayscale8 => {
                        tlg5_compose_colors1(current, &prev, &outbufp, width);
                        outbufp[0] = &outbufp[0][width as usize..];
                    }
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
                    TlgColorType::Grayscale8 => {
                        let mut current_pos = 0usize;
                        let mut pb = 0u8;
                        for x in 0..width as usize {
                            let b = outbufp[0][x];
                            wrapping! {
                                pb += b;
                            }
                            current[current_pos] = pb;
                            current_pos += 1;
                        }
                        outbufp[0] = &outbufp[0][width as usize..];
                    }
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

fn load_tlg6<T: Read + Seek>(src: &mut T) -> Result<Tlg> {
    let mut buf = [0u8; 4];
    src.read_exact(&mut buf)?;
    let colors = buf[0];
    let color_type = match colors {
        3 => TlgColorType::Bgr24,
        4 => TlgColorType::Bgra32,
        1 => TlgColorType::Grayscale8,
        _ => return Err(TlgError::UnsupportedColorType(colors)),
    };
    if buf[1] != 0 {
        return Err(TlgError::Str("Data flags must be 0".to_string()));
    }
    if buf[2] != 0 {
        return Err(TlgError::Str("Color types must be 0".to_string()));
    }
    if buf[3] != 0 {
        return Err(TlgError::Str(
            "External golomb bit length table is not yet supported.".to_string(),
        ));
    }
    let width = src.read_u32()?;
    let height = src.read_u32()?;
    let max_bit_length = src.read_u32()?;
    let x_block_count = (width - 1) / (TLG6_W_BLOCK_SIZE as u32) + 1;
    let y_block_count = (height - 1) / (TLG6_H_BLOCK_SIZE as u32) + 1;
    let main_count = width / (TLG6_W_BLOCK_SIZE as u32);
    let fraction = width - main_count * TLG6_W_BLOCK_SIZE as u32;
    let mut bit_pool = vec![0u8; max_bit_length as usize / 8 + 5];
    let mut pixelbuf = vec![0u32; width as usize * TLG6_H_BLOCK_SIZE + 1];
    let mut filter_types = vec![0u8; x_block_count as usize * y_block_count as usize];
    let mut lzss_text = [0u8; 4096];
    let zero = if colors == 3 { 0xff_00_00_00u32 } else { 0 };
    let zeroline = vec![zero; width as usize];
    {
        let mut p = 0;
        let mut i = 0;
        while i < 0x20u8 {
            let mut j = 0;
            while j < 0x10u8 {
                lzss_text[p] = i;
                p += 1;
                lzss_text[p] = i;
                p += 1;
                lzss_text[p] = i;
                p += 1;
                lzss_text[p] = i;
                p += 1;
                lzss_text[p] = j;
                p += 1;
                lzss_text[p] = j;
                p += 1;
                lzss_text[p] = j;
                p += 1;
                lzss_text[p] = j;
                p += 1;
                j += 1;
            }
            i += 1;
        }
    }
    {
        let inbuf_size = src.read_u32()? as usize;
        let mut inbuf = vec![0u8; inbuf_size];
        src.read_exact(&mut inbuf)?;
        tlg5_decompress_slide(&mut filter_types, &inbuf, inbuf_size, &mut lzss_text, 0);
    }
    let mut prevline = zeroline;
    let mut outbuf = vec![0u32; width as usize * height as usize];
    for y in (0..height).step_by(TLG6_H_BLOCK_SIZE) {
        let y_lim = (y + TLG6_H_BLOCK_SIZE as u32).min(height);
        let pixel_count = (y_lim - y) as usize * width as usize;
        for c in 0..colors {
            let mut bit_length = src.read_u32()?;
            let method = (bit_length >> 30) & 3;
            bit_length &= 0x3fff_ffff;
            let byte_length = (bit_length + 7) / 8;
            if byte_length as usize >= bit_pool.len() {
                return Err(TlgError::Str(
                    "Bit pool is too small for the given bit length".to_string(),
                ));
            }
            src.read_exact(&mut bit_pool[..byte_length as usize])?;
            match method {
                0 => {
                    tlg6_decode_golomb_values(
                        &mut pixelbuf,
                        pixel_count,
                        &bit_pool,
                        c == 0 && colors != 1,
                        c,
                    )?;
                }
                _ => return Err(TlgError::UnsupportedCompressedMethod(method as u8)),
            }
        }
        let ft = &filter_types[(y as usize / TLG6_H_BLOCK_SIZE) * x_block_count as usize..];
        let skip_bytes = (y_lim - y) as usize * TLG6_W_BLOCK_SIZE;
        for yy in y..y_lim {
            let curline =
                &mut outbuf[(yy as usize * width as usize)..(yy as usize + 1) * width as usize];
            let dir = (yy & 1) ^ 1 != 0;
            let oddskip = ((y_lim - yy - 1) as isize) - (yy - y) as isize;
            if main_count != 0 {
                let start = TLG6_W_BLOCK_SIZE.min(width as usize) * (yy - y) as usize;
                tlg6_decode_line(
                    &prevline,
                    curline,
                    width,
                    0,
                    main_count as usize,
                    ft,
                    skip_bytes,
                    &pixelbuf,
                    start,
                    zero,
                    oddskip,
                    dir,
                )?;
            }
            if main_count != x_block_count {
                let ww = TLG6_W_BLOCK_SIZE.min(fraction as usize);
                let start = ww * (yy - y) as usize;
                tlg6_decode_line(
                    &prevline,
                    curline,
                    width,
                    main_count as usize,
                    x_block_count as usize,
                    ft,
                    skip_bytes,
                    &pixelbuf,
                    start,
                    zero,
                    oddskip,
                    dir,
                )?;
            }
            prevline = curline.to_vec();
        }
    }
    let mut data = Vec::with_capacity(outbuf.len() * colors as usize);
    for p in outbuf {
        let t = p.to_le_bytes();
        let b = t[0];
        let g = t[1];
        let r = t[2];
        let a = t[3];
        match color_type {
            TlgColorType::Bgr24 => {
                data.extend_from_slice(&[b, g, r]);
            }
            TlgColorType::Bgra32 => {
                data.extend_from_slice(&[b, g, r, a]);
            }
            TlgColorType::Grayscale8 => {
                data.extend_from_slice(&[b]);
            }
        };
    }
    Ok(Tlg {
        tags: Default::default(),
        version: 6,
        width,
        height,
        color: color_type,
        data,
    })
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
                    c = tag[i];
                    if c != b'=' {
                        check = false;
                        break;
                    }
                    i += 1;
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
