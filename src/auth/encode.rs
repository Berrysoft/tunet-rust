use std::cmp;
use std::string::String;
use std::vec::Vec;

const BASE64N: &'static str = "LVoJPiCN2R8G90yg+hmFHuacZ1OWMnrsSTXkYpUq/3dlbfKwv6xztjI7DeBE45QA";

pub fn base64(t: &[u8]) -> String {
    let a = t.len();
    let len = (a + 2) / 3 * 4;
    let mut u = Vec::with_capacity(len);
    let r = '=' as u8;
    let mut ui = 0;
    for o in (0..a).step_by(3) {
        let mut h = (t[o] as u32) << 16;
        if o + 1 < a {
            h += (t[o + 1] as u32) << 8;
        }
        if o + 2 < a {
            h += t[o + 2] as u32;
        }
        for i in 0..4 {
            if o * 8 + i * 6 > a * 8 {
                u[ui] = r;
            } else {
                u[ui] = BASE64N
                    .bytes()
                    .nth((h >> 6 * (3 - i) & 0x3F) as usize)
                    .unwrap();
            }
            ui += 1;
        }
    }
    String::from_utf8(u).unwrap()
}

fn s(a: &[u8], b: bool) -> Vec<u32> {
    let c = a.len();
    let n = (c + 3) / 4;
    let mut v: Vec<u32>;
    if b {
        v = Vec::with_capacity(n + 1);
        v[n] = c as u32;
    } else {
        v = Vec::with_capacity(cmp::max(n, 4))
    }
    for i in (0..c).step_by(4) {
        let mut pb = a[i] as u32;
        if i + 1 < c {
            pb += (a[i + 1] as u32) << 8;
            if i + 2 < c {
                pb += (a[i + 2] as u32) << 16;
                if i + 3 < c {
                    pb += (a[i + 3] as u32) << 24;
                }
            }
        }
        v[i / 4] = pb;
    }
    v
}

fn l(a: &[u32], b: bool) -> Vec<u8> {
    let d = a.len();
    let mut c = ((d - 1) as u32) << 2;
    if b {
        let m = a[d - 1];
        if m < c - 3 || m > c {
            return Vec::new();
        }
        c = m;
    }
    let n = d << 2;
    let mut aa = Vec::with_capacity(n);
    for i in (0..n).step_by(4) {
        aa[i] = (a[i / 4] % 0x100) as u8;
        aa[i + 1] = ((a[i / 4] >> 8) % 0x100) as u8;
        aa[i + 2] = ((a[i / 4] >> 16) % 0x100) as u8;
        aa[i + 3] = (a[i / 4] >> 24) as u8;
    }
    if b {
        aa[0..(c as usize)].to_vec()
    } else {
        aa
    }
}

pub fn xencode(st: &str, key: &str) -> Vec<u8> {
    if st.len() == 0 {
        return Vec::new();
    }
    let mut v = s(st.as_bytes(), true);
    let k = s(key.as_bytes(), false);
    let n = v.len() - 1;
    let mut z = v[n];
    let mut y: u32;
    let q = 6 + 52 / (n + 1);
    let mut d = 0;
    for _i in 0..q {
        d += 0x9E3779B9;
        let e = (d >> 2) & 3;
        for p in 0..=n {
            y = v[(p + 1) % (n + 1)];
            let mut m = (z >> 5) ^ (y << 2);
            m += (y >> 3) ^ (z << 4) ^ (d ^ y);
            m += k[(((p & 3) as u32) ^ e) as usize] ^ z;
            v[p] += m;
            z = v[p];
        }
    }
    l(&v, false)
}
