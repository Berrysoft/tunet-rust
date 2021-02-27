use std::{mem::size_of, ptr::copy_nonoverlapping};

pub struct AuthTea {
    k: [u32; 4],
}

fn to_u32_with_len(a: &[u8]) -> Vec<u8> {
    let c = a.len();
    let n = (c + 3) / 4;
    let mut v = vec![0; (n + 1) * 4];
    let pv = v.as_mut_ptr();
    unsafe {
        pv.cast::<u32>().add(n).write_unaligned(c as u32);
        copy_nonoverlapping(a.as_ptr(), pv, c);
    }
    v
}

impl AuthTea {
    pub fn new(key: &[u8]) -> Self {
        let mut k = [0; 4];
        if key.len() > 0 {
            unsafe {
                copy_nonoverlapping(
                    key.as_ptr(),
                    k.as_mut_ptr().cast(),
                    key.len().min(size_of::<[u32; 4]>()),
                );
            }
        }
        Self { k }
    }

    pub fn encrypt_str(&self, data: &str) -> Vec<u8> {
        let mut vv = to_u32_with_len(data.as_bytes());
        let n = vv.len() / 4 - 1;
        let v = vv.as_mut_ptr().cast::<u32>();
        let mut y: u32;
        let q = 6 + 52 / (n + 1);
        let mut d: u32 = 0;
        unsafe {
            let mut z = v.add(n).read_unaligned();
            for _i in 0..q {
                d = d.wrapping_add(0x9E3779B9);
                let e = (d >> 2) & 3;
                for p in 0..=n {
                    y = v.add((p + 1) % (n + 1)).read_unaligned();
                    let mut m = (z >> 5) ^ (y << 2);
                    m = m.wrapping_add((y >> 3) ^ (z << 4) ^ (d ^ y));
                    m = m.wrapping_add(self.k[(((p & 3) as u32) ^ e) as usize] ^ z);
                    m = m.wrapping_add(v.add(p).read_unaligned());
                    v.add(p).write_unaligned(m);
                    z = m;
                }
            }
        }
        vv
    }
}
