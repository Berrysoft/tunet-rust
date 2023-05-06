#![forbid(unsafe_code)]
#![allow(clippy::many_single_char_names)]

pub struct AuthTea {
    k: [u32; 4],
}

#[inline]
fn to_u32_with_len(a: &[u8]) -> Vec<u8> {
    let c = a.len();
    let n = (c + 3) / 4;
    let mut v = vec![0; (n + 1) * 4];
    v[..c].copy_from_slice(a);
    write_u32(&mut v[n * 4..][..4], c as u32);
    v
}

#[inline]
fn read_u32(slice: &[u8]) -> u32 {
    let mut chunk = [0u8; 4];
    chunk[..slice.len().min(4)].copy_from_slice(slice);
    u32::from_le_bytes(chunk)
}

#[inline]
fn write_u32(slice: &mut [u8], value: u32) {
    slice.copy_from_slice(&value.to_le_bytes());
}

impl AuthTea {
    pub fn new(key: &[u8]) -> Self {
        let mut k = [0u32; 4];
        for (kitem, keychunk) in k.iter_mut().zip(key.chunks(4)) {
            *kitem = read_u32(keychunk);
        }
        Self { k }
    }

    pub fn encode(&self, data: &[u8]) -> Vec<u8> {
        let mut v = to_u32_with_len(data);
        let n = v.len() / 4 - 1;
        let mut y: u32;
        let q = 6 + 52 / (n + 1);
        let mut d: u32 = 0;
        let mut z = read_u32(&v[n * 4..][..4]);
        for _i in 0..q {
            d = d.wrapping_add(0x9E3779B9);
            let e = (d >> 2) & 3;
            for p in 0..=n {
                y = read_u32(&v[((p + 1) % (n + 1)) * 4..][..4]);
                let mut m = (z >> 5) ^ (y << 2);
                m = m.wrapping_add((y >> 3) ^ (z << 4) ^ (d ^ y));
                m = m.wrapping_add(self.k[(((p & 3) as u32) ^ e) as usize] ^ z);
                m = m.wrapping_add(read_u32(&v[p * 4..][..4]));
                write_u32(&mut v[p * 4..][..4], m);
                z = m;
            }
        }
        v
    }
}

#[cfg(test)]
mod test {
    use crate::AuthTea;

    #[test]
    fn encode() {
        static KEY: &[u8] = b"b8d81680988d65bf9f07a016e8f1e36d8c67d513b1623a7cf42e21b8f2bad445";
        static DATA: &[u8] = b"Hello world!";
        static ENCODE: &[u8] = &[
            216, 3, 10, 118, 79, 58, 195, 163, 49, 56, 10, 252, 12, 128, 206, 202,
        ];

        let tea = AuthTea::new(KEY);
        assert_eq!(tea.encode(DATA), ENCODE);
    }

    #[test]
    fn encode_no_key() {
        static KEY: &[u8] = b"";
        static DATA: &[u8] = b"Hello world!";
        static ENCODE: &[u8] = &[
            153, 233, 116, 208, 214, 130, 167, 74, 156, 115, 202, 99, 214, 35, 164, 244,
        ];

        let tea = AuthTea::new(KEY);
        assert_eq!(tea.encode(DATA), ENCODE);
    }
}
