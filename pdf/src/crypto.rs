pub struct Rc4 {
    s: [u8; 256],
    i: u8,
    j: u8,
}

impl Rc4 {
    // Initialize RC4 with a key
    pub fn new(key: &[u8]) -> Self {
        let mut s = [0u8; 256];
        for i in 0..256 {
            s[i] = i as u8;
        }

        let mut j = 0u8;
        for i in 0..256 {
            j = j.wrapping_add(s[i]).wrapping_add(key[i % key.len()]);
            s.swap(i, j as usize);
        }

        Rc4 { s, i: 0, j: 0 }
    }

    pub fn apply_keystream(&mut self, data: &mut [u8]) {
        for byte in data.iter_mut() {
            self.i = self.i.wrapping_add(1);
            self.j = self.j.wrapping_add(self.s[self.i as usize]);

            self.s.swap(self.i as usize, self.j as usize);

            let t = self.s[self.i as usize].wrapping_add(self.s[self.j as usize]);
            *byte ^= self.s[t as usize];
        }
    }
}

pub fn rc4_decrypt(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut cipher = Rc4::new(key);
    let mut data = data.to_vec();
    cipher.apply_keystream(&mut data);
    return data;
}

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

use aes::cipher::block_padding::{NoPadding, ZeroPadding};
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};

pub fn aes128_decrypt(key: &[u8], data: &[u8]) -> Vec<u8> {
    let data = data.to_vec();
    let iv = data[..16].to_vec();
    let mut data = data[16..].to_vec();
    let key = GenericArray::clone_from_slice(key.into());
    let giv = GenericArray::from_slice(iv.as_slice());
    let _pt = Aes128CbcDec::new(&key, &giv)
        .decrypt_padded_mut::<Pkcs7>(&mut data)
        .unwrap();
    let last = data.last().unwrap().to_owned();
    let decrypt_length = data.len() - last as usize;
    let decrypt_data = data[..decrypt_length].to_vec();
    return decrypt_data;
}

pub fn aes256_decrypt(key: &[u8], data: &[u8]) -> Vec<u8> {
    let data = data.to_vec();
    let iv = data[..16].to_vec();
    let mut data = data[16..].to_vec();
    let key = GenericArray::clone_from_slice(key.into());
    let giv = GenericArray::from_slice(iv.as_slice());
    let _pt = Aes256CbcDec::new(&key, &giv)
        .decrypt_padded_mut::<Pkcs7>(&mut data)
        .unwrap();
    let last = data.last().unwrap().to_owned();
    let decrypt_length = data.len() - last as usize;
    let decrypt_data = data[..decrypt_length].to_vec();
    return decrypt_data;
}

pub fn aes_cbc_encrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Vec<u8> {
    assert_eq!(key.len(), 16);
    let mut data = data.to_vec();
    let key = GenericArray::clone_from_slice(key.into());
    let giv = GenericArray::from_slice(iv.into());
    let length = data.len();
    let _pt = Aes128CbcEnc::new(&key, &giv)
        .encrypt_padded_mut::<ZeroPadding>(&mut data, length)
        .unwrap();
    return data;
}
pub fn aes_cbc_decrypt(key: &[u8], iv: &[u8], data: &[u8]) -> Vec<u8> {
    assert_eq!(key.len(), 32);
    let mut data = data.to_vec();
    let key = GenericArray::clone_from_slice(key.into());
    let giv = GenericArray::from_slice(iv.into());
    let _pt = Aes256CbcDec::new(&key, &giv)
        .decrypt_padded_mut::<ZeroPadding>(&mut data)
        .unwrap();
    return data;
}
