//! Slide Compressor
#[derive(Clone, Copy)]
struct Chain {
    prev: i32,
    next: i32,
}

const SLIDE_N: usize = 4096;
const SLIDE_M: usize = 18 + 255;
const TEXT_SIZE: usize = SLIDE_N + SLIDE_M;
const MAP_SIZE: usize = 256 * 256;

pub struct SlideCompressor {
    text: Vec<u8>,
    map: Vec<i32>,
    chains: Vec<Chain>,
    text2: Vec<u8>,
    map2: Vec<i32>,
    chains2: Vec<Chain>,
    s: i32,
    s2: i32,
}

impl SlideCompressor {
    pub fn new() -> Self {
        let mut data = Self {
            text: vec![0; TEXT_SIZE],
            map: vec![-1; MAP_SIZE],
            chains: vec![Chain { prev: -1, next: -1 }; SLIDE_N],
            text2: vec![0; TEXT_SIZE],
            map2: vec![0; MAP_SIZE],
            chains2: vec![Chain { prev: 0, next: 0 }; SLIDE_N],
            s: 0,
            s2: 0,
        };
        for i in (0..SLIDE_N).rev() {
            data.add_map(i as i32);
        }
        data
    }

    fn add_map(&mut self, p: i32) {
        let place = self.text[p as usize] as i32
            + ((self.text[(p as usize + 1) & (SLIDE_N - 1)] as i32) << 8);
        if self.map[place as usize] == -1 {
            self.map[place as usize] = p;
        } else {
            let old = self.map[place as usize];
            self.map[place as usize] = p;
            self.chains[old as usize].prev = p;
            self.chains[p as usize].next = old;
            self.chains[p as usize].prev = -1;
        }
    }

    fn delete_map(&mut self, p: i32) {
        let p_us = p as usize;
        let mut n = self.chains[p_us].next;
        if n != -1 {
            self.chains[n as usize].prev = self.chains[p_us].prev;
        }
        n = self.chains[p_us].prev;
        if n != -1 {
            self.chains[n as usize].next = self.chains[p_us].next;
        } else if self.chains[p_us].next != -1 {
            let place =
                self.text[p_us] as i32 + ((self.text[(p_us + 1) & (SLIDE_N - 1)] as i32) << 8);
            self.map[place as usize] = self.chains[p_us].next;
        } else {
            let place =
                self.text[p_us] as i32 + ((self.text[(p_us + 1) & (SLIDE_N - 1)] as i32) << 8);
            self.map[place as usize] = -1;
        }
        self.chains[p_us].prev = -1;
        self.chains[p_us].next = -1;
    }

    fn get_match(&self, cur: &[u8], s: i32) -> (i32, i32) {
        if cur.len() < 3 {
            return (0, 0);
        }
        let mut curlen = cur.len() as i32;
        let place = cur[0] as i32 + ((cur[1] as i32) << 8);
        let mut pos = 0;
        let mut maxlen = 0;
        let mut head = self.map[place as usize];
        if head == -1 {
            return (0, 0);
        }
        curlen -= 1;
        while head != -1 {
            let place_org = head;
            if s == place_org || s == ((place_org + 1) & (SLIDE_N as i32 - 1)) {
                head = self.chains[place_org as usize].next;
                continue;
            }
            let mut p = place_org + 2;
            let mut lim = (if (SLIDE_M as i32) < curlen {
                SLIDE_M as i32
            } else {
                curlen
            }) + place_org;
            if lim >= SLIDE_N as i32 {
                if place_org <= s && s < SLIDE_N as i32 {
                    lim = s;
                } else if s < (lim & (SLIDE_N as i32 - 1)) {
                    lim = s + SLIDE_N as i32;
                }
            } else {
                if place_org <= s && s < lim {
                    lim = s;
                }
            }
            let mut c_index = 2;
            while p < lim
                && (c_index as usize) < cur.len()
                && self.text[p as usize] == cur[c_index as usize]
            {
                p += 1;
                c_index += 1;
            }
            let matchlen = p - place_org;
            if matchlen > maxlen {
                maxlen = matchlen;
                pos = place_org;
                if matchlen == SLIDE_M as i32 {
                    return (maxlen, pos);
                }
            }
            head = self.chains[place_org as usize].next;
        }
        (maxlen, pos)
    }

    pub fn encode_into(&mut self, input: &[u8], output: &mut Vec<u8>) -> usize {
        if input.is_empty() {
            return 0;
        }
        let mut code = [0u8; 40];
        let mut codeptr: usize = 1;
        let mut mask: u8 = 1;
        code[0] = 0;
        let mut idx: usize = 0;
        let mut remain = input.len();
        let mut s = self.s;
        while remain > 0 {
            let (len, pos) = {
                let (l, p) = self.get_match(&input[idx..], s);
                (l, p)
            };
            if len >= 3 {
                code[0] |= mask;
                if len >= 18 {
                    code[codeptr] = (pos & 0xff) as u8;
                    codeptr += 1;
                    code[codeptr] = (((pos & 0xf00) >> 8) as u8) | 0xf0;
                    codeptr += 1;
                    code[codeptr] = (len - 18) as u8;
                    codeptr += 1;
                } else {
                    code[codeptr] = (pos & 0xff) as u8;
                    codeptr += 1;
                    code[codeptr] = (((pos & 0xf00) >> 8) as u8) | (((len - 3) as u8) << 4);
                    codeptr += 1;
                }
                let mut l = len as usize;
                while l > 0 {
                    let c = input[idx];
                    idx += 1;
                    remain -= 1;
                    l -= 1;
                    let s_prev = (s - 1) & (SLIDE_N as i32 - 1);
                    self.delete_map(s_prev);
                    self.delete_map(s);
                    if (s as usize) < SLIDE_M - 1 {
                        self.text[(s as usize) + SLIDE_N] = c;
                    }
                    self.text[s as usize] = c;
                    self.add_map(s_prev);
                    self.add_map(s);
                    s = (s + 1) & (SLIDE_N as i32 - 1);
                }
            } else {
                let c = input[idx];
                idx += 1;
                remain -= 1;
                let s_prev = (s - 1) & (SLIDE_N as i32 - 1);
                self.delete_map(s_prev);
                self.delete_map(s);
                if (s as usize) < SLIDE_M - 1 {
                    self.text[(s as usize) + SLIDE_N] = c;
                }
                self.text[s as usize] = c;
                self.add_map(s_prev);
                self.add_map(s);
                s = (s + 1) & (SLIDE_N as i32 - 1);
                code[codeptr] = c;
                codeptr += 1;
            }
            mask <<= 1;
            if mask == 0 {
                output.extend_from_slice(&code[..codeptr]);
                mask = 1;
                codeptr = 1;
                code[0] = 0;
            }
        }
        if mask != 1 {
            output.extend_from_slice(&code[..codeptr]);
        }
        self.s = s;
        output.len()
    }

    pub fn store(&mut self) {
        self.s2 = self.s;
        self.text2.copy_from_slice(&self.text);
        self.map2.copy_from_slice(&self.map);
        self.chains2.copy_from_slice(&self.chains);
    }

    pub fn restore(&mut self) {
        self.s = self.s2;
        self.text.copy_from_slice(&self.text2);
        self.map.copy_from_slice(&self.map2);
        self.chains.copy_from_slice(&self.chains2);
    }
}
