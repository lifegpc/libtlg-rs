use crate::*;
use overf::wrapping;

const TLG6_GOLOMB_N_COUNT: usize = 4;
const TLG6_LEADING_ZERO_TABLE_BITS: usize = 12;
const TLG6_LEADING_ZERO_TABLE_SIZE: usize = 1 << TLG6_LEADING_ZERO_TABLE_BITS;
const TLG6_GOLOMB_COMPRESSED: [[u16; 9]; TLG6_GOLOMB_N_COUNT] = [
    [3, 7, 15, 27, 63, 108, 223, 448, 130],
    [3, 5, 13, 24, 51, 95, 192, 384, 257],
    [2, 5, 12, 21, 39, 86, 155, 320, 384],
    [2, 3, 9, 18, 33, 61, 129, 258, 511],
];
const TLG6_GLOBMB_TABLE_SIZE: usize = TLG6_GOLOMB_N_COUNT * 2 * 128;
pub const TLG6_W_BLOCK_SIZE: usize = 8;
pub const TLG6_H_BLOCK_SIZE: usize = 8;

lazy_static::lazy_static! {
    static ref TLG6_LEADING_ZERO_TABLE: [u8; TLG6_LEADING_ZERO_TABLE_SIZE] =
        tlg6_init_leading_zero_table();
    static ref TLG6_GOLOMB_BIT_LENGTH_TABLE: [[i8; TLG6_GOLOMB_N_COUNT]; TLG6_GLOBMB_TABLE_SIZE] =
        tlg6_init_golomb_table();
}

fn tlg6_init_leading_zero_table() -> [u8; TLG6_LEADING_ZERO_TABLE_SIZE] {
    let mut table = [0; TLG6_LEADING_ZERO_TABLE_SIZE];
    for i in 0..TLG6_LEADING_ZERO_TABLE_SIZE {
        let mut cnt = 0;
        let mut j = 1;
        while j != TLG6_LEADING_ZERO_TABLE_SIZE && i & j == 0 {
            j <<= 1;
            cnt += 1;
        }
        cnt += 1;
        if j == TLG6_LEADING_ZERO_TABLE_SIZE {
            cnt = 0;
        }
        table[i] = cnt as u8;
    }
    table
}

fn tlg6_init_golomb_table() -> [[i8; TLG6_GOLOMB_N_COUNT]; TLG6_GLOBMB_TABLE_SIZE] {
    let mut table = [[0; TLG6_GOLOMB_N_COUNT]; TLG6_GLOBMB_TABLE_SIZE];
    for n in 0..TLG6_GOLOMB_N_COUNT {
        let mut a = 0;
        for i in 0..9 {
            for _ in 0..TLG6_GOLOMB_COMPRESSED[n][i] {
                table[a][n] = i as i8;
                a += 1;
            }
        }
        debug_assert!(a == TLG6_GLOBMB_TABLE_SIZE);
    }
    table
}

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

pub fn tlg6_fetch_32bits(data: &[u8], loc: usize) -> Result<u32> {
    if data.len() < loc + 4 {
        return Err(TlgError::IndexOutOfRange);
    }
    Ok(u32::from_le_bytes([
        data[loc],
        data[loc + 1],
        data[loc + 2],
        data[loc + 3],
    ]))
}

pub fn tlg6_decode_golomb_values(
    pixelbuf: &mut [u32],
    pixel_count: usize,
    bit_pool: &[u8],
    is_first: bool,
    c: u8,
) -> Result<()> {
    let mut n = TLG6_GOLOMB_N_COUNT - 1;
    let mut a = 0;
    let mut bit_pos = 1;
    let mut zero = if bit_pool[0] & 1 != 0 { 0u8 } else { 1 };
    let mut index = 0;
    let mut bit_pool_index = 0;
    while index < pixel_count {
        let mut count;
        {
            let mut t = tlg6_fetch_32bits(bit_pool, bit_pool_index)? >> bit_pos;
            let mut b = TLG6_LEADING_ZERO_TABLE[(t as usize) & (TLG6_LEADING_ZERO_TABLE_SIZE - 1)];
            let mut bit_count = b as i32;
            while b == 0 {
                bit_count += TLG6_LEADING_ZERO_TABLE_BITS as i32;
                bit_pos += TLG6_LEADING_ZERO_TABLE_BITS as i32;
                bit_pool_index += bit_pos as usize >> 3;
                bit_pos &= 7;
                t = tlg6_fetch_32bits(bit_pool, bit_pool_index)? >> bit_pos;
                b = TLG6_LEADING_ZERO_TABLE[(t as usize) & (TLG6_LEADING_ZERO_TABLE_SIZE - 1)];
                bit_count += b as i32;
            }
            bit_pos += b as i32;
            bit_pool_index += bit_pos as usize >> 3;
            bit_pos &= 7;
            bit_count -= 1;
            count = 1 << bit_count;
            count += (tlg6_fetch_32bits(bit_pool, bit_pool_index)? >> bit_pos) & (count - 1);
            bit_pos += bit_count;
            bit_pool_index += bit_pos as usize >> 3;
            bit_pos &= 7;
        }
        if zero != 0 {
            loop {
                if is_first {
                    pixelbuf[index] = 0;
                } else {
                    let mut tmp = pixelbuf[index].to_le_bytes();
                    tmp[c as usize] = 0;
                    pixelbuf[index] = u32::from_le_bytes(tmp);
                }
                index += 1;
                count -= 1;
                if count == 0 {
                    break;
                }
            }
            zero ^= 1;
        } else {
            loop {
                let k = TLG6_GOLOMB_BIT_LENGTH_TABLE[a][n];
                let mut t = tlg6_fetch_32bits(bit_pool, bit_pool_index)? >> bit_pos;
                let mut bit_count;
                let mut b;
                let mut v;
                let sign;
                if t != 0 {
                    b = TLG6_LEADING_ZERO_TABLE[(t as usize) & (TLG6_LEADING_ZERO_TABLE_SIZE - 1)];
                    bit_count = b as i32;
                    while b == 0 {
                        bit_count += TLG6_LEADING_ZERO_TABLE_BITS as i32;
                        bit_pos += TLG6_LEADING_ZERO_TABLE_BITS as i32;
                        bit_pool_index += bit_pos as usize >> 3;
                        bit_pos &= 7;
                        t = tlg6_fetch_32bits(bit_pool, bit_pool_index)? >> bit_pos;
                        b = TLG6_LEADING_ZERO_TABLE
                            [(t as usize) & (TLG6_LEADING_ZERO_TABLE_SIZE - 1)];
                        bit_count += b as i32;
                    }
                    bit_count -= 1;
                } else {
                    bit_pool_index += 5;
                    bit_count = if bit_pool_index == 0 {
                        return Err(TlgError::IndexOutOfRange);
                    } else {
                        bit_pool[bit_pool_index - 1] as i32
                    };
                    bit_pos = 0;
                    t = tlg6_fetch_32bits(bit_pool, bit_pool_index)?;
                    b = 0;
                }
                v = (bit_count << k) + ((t as i32 >> b) & ((1 << k) - 1));
                sign = (v & 1) - 1;
                v >>= 1;
                a += v as usize;
                if is_first {
                    pixelbuf[index] = (wrapping!((v ^ sign) + sign + 1) as u32) & 0xFF;
                } else {
                    let mut tmp = pixelbuf[index].to_le_bytes();
                    tmp[c as usize] = wrapping!((v ^ sign) + sign + 1) as u8;
                    pixelbuf[index] = u32::from_le_bytes(tmp);
                }
                index += 1;
                bit_pos += b as i32 + k as i32;
                bit_pool_index += bit_pos as usize >> 3;
                bit_pos &= 7;
                if n == 0 {
                    n = TLG6_GOLOMB_N_COUNT - 1;
                    a >>= 1;
                } else {
                    n -= 1;
                }
                count -= 1;
                if count == 0 {
                    break;
                }
            }
            zero ^= 1;
        }
    }
    Ok(())
}

#[inline(always)]
fn make_gt_mask(a: u32, b: u32) -> u32 {
    let tmp2 = !b;
    let tmp = wrapping! {((a & tmp2) + (((a ^ tmp2) >> 1) & 0x7f7f7f7f) ) & 0x80808080};
    wrapping! { ((tmp >> 7) + 0x7f7f7f7f) ^ 0x7f7f7f7f }
}

#[inline(always)]
fn packed_bytes_add(a: u32, b: u32) -> u32 {
    let tmp = wrapping! {(((a & b) << 1) + ((a ^ b) & 0xfefefefe) ) & 0x01010100};
    wrapping!(a + b - tmp)
}

#[inline(always)]
fn med2(a: u32, b: u32, c: u32) -> u32 {
    let aa_gt_bb = make_gt_mask(a, b);
    let a_xor_b_and_aa_gt_bb = (a ^ b) & aa_gt_bb;
    let aa = a_xor_b_and_aa_gt_bb ^ a;
    let bb = a_xor_b_and_aa_gt_bb ^ b;
    let n = make_gt_mask(c, bb);
    let nn = make_gt_mask(aa, c);
    let m = !(n | nn);
    wrapping! {
        (n & aa) | (nn & bb) | ((bb & m) - (c & m) + (aa & m))
    }
}

#[inline(always)]
fn med(a: u32, b: u32, c: u32, v: u32) -> u32 {
    packed_bytes_add(med2(a, b, c), v)
}

#[inline(always)]
fn avg_packed(x: u32, y: u32) -> u32 {
    wrapping!(((x) & (y)) + ((((x) ^ (y)) & 0xfefefefe) >> 1)) + (((x) ^ (y)) & 0x01010101)
}

#[inline(always)]
fn avg(a: u32, b: u32, v: u32) -> u32 {
    packed_bytes_add(avg_packed(a, b), v)
}

#[inline(always)]
fn cal_v(b: u8, g: u8, r: u8, a: u8) -> u32 {
    (0xff0000 & ((r as u32) << 16))
        + (0xff00 & ((g as u32) << 8))
        + (0xff & (b as u32))
        + ((a as u32) << 24)
}

pub fn tlg6_decode_line(
    prevline: &[u32],
    curline: &mut [u32],
    width: u32,
    start_block: usize,
    block_limit: usize,
    filter_types: &[u8],
    skipblockbytes: usize,
    inp: &[u32],
    mut inp_pos: usize,
    initialp: u32,
    oddskip: isize,
    dir: bool,
) -> Result<()> {
    let mut p;
    let mut up;
    let step: i32;
    let mut prevline_pos = 0;
    let mut curline_pos = 0;
    if start_block != 0 {
        prevline_pos += start_block * TLG6_W_BLOCK_SIZE;
        curline_pos += start_block * TLG6_W_BLOCK_SIZE;
        p = curline[curline_pos - 1];
        up = prevline[prevline_pos - 1];
    } else {
        p = initialp;
        up = initialp;
    }
    inp_pos += skipblockbytes * start_block;
    step = if dir { 1 } else { -1 };
    for i in start_block..block_limit {
        let mut w = (width as usize - i * TLG6_W_BLOCK_SIZE).min(TLG6_W_BLOCK_SIZE);
        let ww = w;
        if step == -1 {
            inp_pos += ww - 1;
        }
        if i & 1 != 0 {
            inp_pos = (inp_pos as isize + oddskip as isize * ww as isize) as usize;
        }
        loop {
            let inpt = inp[inp_pos];
            let ia = ((inpt >> 24) & 0xFF) as u8;
            let ir = ((inpt >> 16) & 0xFF) as u8;
            let ig = ((inpt >> 8) & 0xFF) as u8;
            let ib = (inpt & 0xFF) as u8;
            let u = prevline[prevline_pos];
            p = match filter_types[i] {
                0 => med(p, u, up, cal_v(ib, ig, ir, ia)),
                1 => avg(p, u, cal_v(ib, ig, ir, ia)),
                2 => med(
                    p,
                    u,
                    up,
                    cal_v(wrapping!(ib + ig), ig, wrapping!(ir + ig), ia),
                ),
                3 => avg(p, u, cal_v(wrapping!(ib + ig), ig, wrapping!(ir + ig), ia)),
                4 => med(
                    p,
                    u,
                    up,
                    cal_v(ib, wrapping!(ig + ib), wrapping!(ir + ib + ig), ia),
                ),
                5 => avg(
                    p,
                    u,
                    cal_v(ib, wrapping!(ig + ib), wrapping!(ir + ib + ig), ia),
                ),
                6 => med(
                    p,
                    u,
                    up,
                    cal_v(wrapping!(ib + ir + ig), wrapping!(ig + ir), ir, ia),
                ),
                7 => avg(
                    p,
                    u,
                    cal_v(wrapping!(ib + ir + ig), wrapping!(ig + ir), ir, ia),
                ),
                8 => med(
                    p,
                    u,
                    up,
                    cal_v(
                        wrapping!(ib + ir),
                        wrapping!(ig + ib + ir),
                        wrapping!(ir + ib + ir + ig),
                        ia,
                    ),
                ),
                9 => avg(
                    p,
                    u,
                    cal_v(
                        wrapping!(ib + ir),
                        wrapping!(ig + ib + ir),
                        wrapping!(ir + ib + ir + ig),
                        ia,
                    ),
                ),
                10 => med(
                    p,
                    u,
                    up,
                    cal_v(wrapping!(ib + ir), wrapping!(ig + ib + ir), ir, ia),
                ),
                11 => avg(
                    p,
                    u,
                    cal_v(wrapping!(ib + ir), wrapping!(ig + ib + ir), ir, ia),
                ),
                12 => med(p, u, up, cal_v(wrapping!(ib + ig), ig, ir, ia)),
                13 => avg(p, u, cal_v(wrapping!(ib + ig), ig, ir, ia)),
                14 => med(p, u, up, cal_v(ib, wrapping!(ig + ib), ir, ia)),
                15 => avg(p, u, cal_v(ib, wrapping!(ig + ib), ir, ia)),
                16 => med(p, u, up, cal_v(ib, ig, wrapping!(ir + ig), ia)),
                17 => avg(p, u, cal_v(ib, ig, wrapping!(ir + ig), ia)),
                18 => med(
                    p,
                    u,
                    up,
                    cal_v(
                        wrapping!(ib + ig + ir + ib),
                        wrapping!(ig + ir + ib),
                        wrapping!(ir + ib),
                        ia,
                    ),
                ),
                19 => avg(
                    p,
                    u,
                    cal_v(
                        wrapping!(ib + ig + ir + ib),
                        wrapping!(ig + ir + ib),
                        wrapping!(ir + ib),
                        ia,
                    ),
                ),
                20 => med(
                    p,
                    u,
                    up,
                    cal_v(wrapping!(ib + ir), wrapping!(ig + ir), ir, ia),
                ),
                21 => avg(p, u, cal_v(wrapping!(ib + ir), wrapping!(ig + ir), ir, ia)),
                22 => med(
                    p,
                    u,
                    up,
                    cal_v(ib, wrapping!(ig + ib), wrapping!(ir + ib), ia),
                ),
                23 => avg(p, u, cal_v(ib, wrapping!(ig + ib), wrapping!(ir + ib), ia)),
                24 => med(
                    p,
                    u,
                    up,
                    cal_v(ib, wrapping!(ig + ir + ib), wrapping!(ir + ib), ia),
                ),
                25 => avg(
                    p,
                    u,
                    cal_v(ib, wrapping!(ig + ir + ib), wrapping!(ir + ib), ia),
                ),
                26 => med(
                    p,
                    u,
                    up,
                    cal_v(
                        wrapping!(ib + ig),
                        wrapping!(ig + ir + ib + ig),
                        wrapping!(ir + ib + ig),
                        ia,
                    ),
                ),
                27 => avg(
                    p,
                    u,
                    cal_v(
                        wrapping!(ib + ig),
                        wrapping!(ig + ir + ib + ig),
                        wrapping!(ir + ib + ig),
                        ia,
                    ),
                ),
                28 => med(
                    p,
                    u,
                    up,
                    cal_v(
                        wrapping!(ib + ig + ir),
                        wrapping!(ig + ir),
                        wrapping!(ir + ib + ig + ir),
                        ia,
                    ),
                ),
                29 => avg(
                    p,
                    u,
                    cal_v(
                        wrapping!(ib + ig + ir),
                        wrapping!(ig + ir),
                        wrapping!(ir + ib + ig + ir),
                        ia,
                    ),
                ),
                30 => med(
                    p,
                    u,
                    up,
                    cal_v(ib, wrapping!(ig + (ib << 1)), wrapping!(ir + (ib << 1)), ia),
                ),
                31 => avg(
                    p,
                    u,
                    cal_v(ib, wrapping!(ig + (ib << 1)), wrapping!(ir + (ib << 1)), ia),
                ),
                v => {
                    return Err(TlgError::Str(format!("Unsupported filter type: {}", v)));
                }
            };
            up = u;
            curline[curline_pos] = p;
            curline_pos += 1;
            prevline_pos += 1;
            inp_pos = (inp_pos as isize + step as isize) as usize;
            w -= 1;
            if w == 0 {
                break;
            }
        }
        if step == 1 {
            inp_pos += skipblockbytes - ww;
        } else {
            inp_pos += skipblockbytes + 1;
        }
        if i & 1 != 0 {
            inp_pos = (inp_pos as isize - oddskip as isize * ww as isize) as usize;
        }
    }
    Ok(())
}
