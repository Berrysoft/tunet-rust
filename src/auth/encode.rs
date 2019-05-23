use std::cmp;
use std::mem::size_of;
use std::ptr::copy_nonoverlapping;
use std::string::String;
use std::vec::Vec;

const BASE64N: &[u8] =
    "LVoJPiCN2R8G90yg+hmFHuacZ1OWMnrsSTXkYpUq/3dlbfKwv6xztjI7DeBE45QA".as_bytes();

pub fn base64(t: &[u8]) -> String {
    let a = t.len();
    let len = (a + 2) / 3 * 4;
    let mut u = vec![b'\0'; len];
    let r = b'=';
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
                u[ui] = BASE64N[(h >> 6 * (3 - i) & 0x3F) as usize];
            }
            ui += 1;
        }
    }
    unsafe { String::from_utf8_unchecked(u) }
}

fn s(a: &[u8], b: bool) -> Vec<u32> {
    let c = a.len();
    let n = (c + 3) / 4;
    let mut v: Vec<u32>;
    if b {
        v = vec![0; n + 1];
        v[n] = c as u32;
    } else {
        v = vec![0; cmp::max(n, 4)];
    }
    unsafe {
        copy_nonoverlapping(a.as_ptr(), v.as_mut_ptr() as *mut u8, c * size_of::<u8>());
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
    let mut aa = vec![b'\0'; n];
    unsafe {
        copy_nonoverlapping(
            a.as_ptr() as *const u8,
            aa.as_mut_ptr(),
            n * size_of::<u8>(),
        );
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
    let mut d: u32 = 0;
    for _i in 0..q {
        let (dd, _) = d.overflowing_add(0x9E3779B9);
        d = dd;
        let e = (d >> 2) & 3;
        for p in 0..=n {
            y = v[(p + 1) % (n + 1)];
            let m = (z >> 5) ^ (y << 2);
            let (m, _) = m.overflowing_add((y >> 3) ^ (z << 4) ^ (d ^ y));
            let (m, _) = m.overflowing_add(k[(((p & 3) as u32) ^ e) as usize] ^ z);
            let (m, _) = m.overflowing_add(v[p]);
            v[p] = m;
            z = v[p];
        }
    }
    l(&v, false)
}
