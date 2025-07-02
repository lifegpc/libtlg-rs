use overf::wrapping;

pub fn tlg5_compose_colors3(outp: &mut [u8], upper: &[u8], buf: &[&[u8]], width: u32) {
    let mut outpos = 0usize;
    let mut upper_pos = 0usize;
    let mut pr = 0u8;
    let mut pg = 0u8;
    let mut pb = 0u8;
    for x in 0..width as usize {
        let mut b = buf[0][x];
        let g = buf[1][x];
        let mut r = buf[2][x];
        wrapping! {
            b += g;
            r += g;
        }
        wrapping! {
            pb += b;
            pg += g;
            pr += r;
        }
        outp[outpos] = wrapping! { pb + upper[upper_pos]};
        outpos += 1;
        upper_pos += 1;
        outp[outpos] = wrapping! { pg + upper[upper_pos]};
        outpos += 1;
        upper_pos += 1;
        outp[outpos] = wrapping! { pr + upper[upper_pos]};
        outpos += 1;
        upper_pos += 1;
    }
}

pub fn tlg5_compose_colors4(outp: &mut [u8], upper: &[u8], buf: &[&[u8]], width: u32) {
    let mut outpos = 0usize;
    let mut upper_pos = 0usize;
    let mut pr = 0u8;
    let mut pg = 0u8;
    let mut pb = 0u8;
    let mut pa = 0u8;
    for x in 0..width as usize {
        let mut b = buf[0][x];
        let g = buf[1][x];
        let mut r = buf[2][x];
        let a = buf[3][x];
        wrapping! {
            b += g;
            r += g;
        }
        wrapping! {
            pb += b;
            pg += g;
            pr += r;
            pa += a;
        }
        outp[outpos] = wrapping! { pb + upper[upper_pos]};
        outpos += 1;
        upper_pos += 1;
        outp[outpos] = wrapping! { pg + upper[upper_pos]};
        outpos += 1;
        upper_pos += 1;
        outp[outpos] = wrapping! { pr + upper[upper_pos]};
        outpos += 1;
        upper_pos += 1;
        outp[outpos] = wrapping! { pa + upper[upper_pos]};
        outpos += 1;
        upper_pos += 1;
    }
}

pub fn tlg5_decompress_slide(
    out: &mut [u8],
    inp: &[u8],
    insize: usize,
    text: &mut [u8],
    mut r: usize,
) -> usize {
    let mut flags = 0u32;
    let mut inpos = 0usize;
    let mut outpos = 0usize;
    while inpos < insize {
        wrapping! { flags >>= 1 };
        if flags & 256 == 0 {
            flags = inp[inpos] as u32 | 0xff00;
            inpos += 1;
        }
        if flags & 1 != 0 {
            let mut mpos =
                wrapping! { inp[inpos] as usize | ((inp[inpos + 1] as usize & 0xf) << 8) };
            let mut mlen = wrapping! { (inp[inpos + 1] as usize & 0xf0) >> 4 };
            inpos += 2;
            mlen += 3;
            if mlen == 18 {
                mlen += inp[inpos] as usize;
                inpos += 1;
            }
            while mlen > 0 {
                out[outpos] = text[mpos];
                outpos += 1;
                text[r] = text[mpos];
                r += 1;
                mpos += 1;
                wrapping! {
                    mpos &= 4095;
                    r &= 4095;
                }
                mlen -= 1;
            }
        } else {
            let c = inp[inpos];
            inpos += 1;
            out[outpos] = c;
            outpos += 1;
            text[r] = c;
            r += 1;
            wrapping! {
                r &= 4095;
            }
        }
    }
    r
}
