use std::i32;

use crate::crypto::{
    aes128_decrypt, aes256_decrypt, aes_cbc_decrypt, aes_cbc_encrypt, rc4_decrypt,
};
use crate::error::{PdfError, Result};
use crate::object::dictionary::PdfDict;
use crate::object::PdfObject;
use crate::xref::Xref;
use md5::{Digest, Md5};

const PASSWORD_PAD: [u8; 32] = [
    0x28, 0xbf, 0x4e, 0x5e, 0x4e, 0x75, 0x8a, 0x41, 0x64, 0x00, 0x4e, 0x56, 0xff, 0xfa, 0x01, 0x08,
    0x2e, 0x2e, 0x00, 0xb6, 0xd0, 0x68, 0x3e, 0x80, 0x2f, 0x0c, 0xa9, 0xfe, 0x64, 0x53, 0x69, 0x7a,
];

fn padd_password(password: &[u8]) -> [u8; 32] {
    let mut res = [0; 32];
    let mut i = 0;
    while i < password.len() && i < 32 {
        res[i] = password[i];
        i += 1;
    }
    let pad_length = 32 - i;
    for j in 0..pad_length {
        res[i + j] = PASSWORD_PAD[j];
    }
    res
}

#[derive(Debug, Default)]
pub struct CryptFilter {
    cfm: Option<String>,
    length: u32,
}

impl CryptFilter {
    pub fn new(cfm: String, length: u32) -> Self {
        Self {
            cfm: Some(cfm),
            length,
        }
    }
    pub fn try_new(cf: &PdfDict) -> Result<Self> {
        let mut filter = CryptFilter::default();
        if let Some(cfm) = cf.get("CFM") {
            let cfm = cfm.as_name()?.name().to_string();
            filter.cfm = Some(cfm);
        }
        if let Some(length) = cf.get("Length") {
            filter.length = length.as_number()?.integer() as u32;
        }
        Ok(filter)
    }
    fn compute_key(&self, key: &[u8], id: u32, gen: u16, length: usize) -> Result<Vec<u8>> {
        let p1 = id.to_le_bytes();
        let p2 = gen.to_le_bytes();
        let mut hasher = Md5::new();
        let mut key_data = key[..length].to_vec();
        for i in 0..3 {
            key_data.push(p1[i]);
        }
        for i in 0..2 {
            key_data.push(p2[i]);
        }
        hasher.update(key_data);
        match self.cfm.as_ref() {
            Some(s) => match s.as_str() {
                "Identity" => Ok(Vec::new()),
                "V2" => {
                    let key_digest = hasher.finalize().to_vec();
                    let key_digest = key_digest[..(length + 5).min(16)].to_vec();
                    return Ok(key_digest.to_vec());
                }
                "AESV2" | "AESV3" => {
                    hasher.update("sAlT");
                    let key_digest = hasher.finalize().to_vec();
                    let key_digest = key_digest[..(length + 5).min(16)].to_vec();
                    return Ok(key_digest);
                }
                _ => {
                    return Err(PdfError::File(format!(
                        "Cryptfilter cfm is invalid:{:?}",
                        self.cfm
                    )))
                }
            },
            None => {
                return Err(PdfError::File("Cryptfilter cfm is None".to_string()));
            }
        }
    }
    pub fn decrypt_object(
        &self,
        key: &[u8],
        data: &[u8],
        id: u32,
        gen: u16,
        length: usize,
    ) -> Result<Vec<u8>> {
        match self.cfm.as_ref() {
            Some(s) => match s.as_str() {
                "Identity" => {
                    return Ok(data.to_vec());
                }
                "V2" => {
                    let fm_key = self.compute_key(key, id, gen, length)?;
                    let de_data = rc4_decrypt(fm_key.as_slice(), data);
                    return Ok(de_data);
                }
                "AESV2" => {
                    let fm_key = self.compute_key(key, id, gen, length)?;
                    let de_data = aes128_decrypt(fm_key.as_slice(), data);
                    return Ok(de_data);
                }
                "AESV3" => {
                    let de_data = aes256_decrypt(key, data);
                    return Ok(de_data);
                }
                _ => {
                    return Err(PdfError::File(format!(
                        "Cryptfilter:{:?} not supported",
                        self.cfm
                    )));
                }
            },
            None => {
                return Ok(data.to_vec());
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct SecurityHandler {
    version: i32,
    rvision: i32,
    o: Vec<u8>,
    u: Vec<u8>,
    p: i32,
    oe: Vec<u8>,
    ue: Vec<u8>,
    stream_filter: CryptFilter,
    string_filter: CryptFilter,
    length: u32,
    id1: Vec<u8>,
    encrypt_metadata: bool,
    key: Option<Vec<u8>>,
}

impl SecurityHandler {
    pub fn try_new(encrypt: &PdfDict, xref: &Xref, password: Option<&[u8]>) -> Result<Self> {
        let mut security_handler = SecurityHandler::default();
        match encrypt.get("Filter") {
            Some(PdfObject::Name(name)) => {
                if name.name() == "Standard" {
                    // initStandard SecurityHandler
                } else {
                    return Err(PdfError::File(
                        "SecurityHandler filter invalid name ".to_string(),
                    ));
                }
            }
            Some(_) => {
                return Err(PdfError::File("SecurityHandler invalid filter".to_string()));
            }
            None => return Err(PdfError::File("SecurityHandler filter is None".to_string())),
        }
        let id1 = xref
            .trailer()
            .get("ID")
            .unwrap()
            .as_array()?
            .get(0)
            .unwrap()
            .as_hex_string()?
            .raw_bytes()?;
        security_handler.id1 = id1;

        let v = encrypt
            .get("V")
            .ok_or(PdfError::File("Encrypt invlaid V params".to_string()))?
            .as_number()
            .map_err(|_| PdfError::File("Encrypt V param must be a number".to_string()))?
            .integer();
        security_handler.version = v;
        let r = encrypt
            .get("R")
            .ok_or(PdfError::File("Encrypt invalid R params".to_string()))?
            .as_number()
            .map_err(|_| PdfError::File("Encrypt param R must be a number".to_string()))?
            .integer();
        security_handler.rvision = r;
        match encrypt.get("O") {
            Some(PdfObject::LiteralString(s)) => {
                security_handler.o = s.bytes().to_vec();
            }
            Some(PdfObject::HexString(s)) => {
                security_handler.o = s.raw_bytes()?;
            }
            _ => return Err(PdfError::File("Encrypt invalid o Params".to_string())),
        }
        match encrypt.get("U") {
            Some(PdfObject::LiteralString(s)) => {
                security_handler.u = s.bytes().to_vec();
            }
            Some(PdfObject::HexString(s)) => {
                security_handler.u = s.raw_bytes()?;
            }
            _ => return Err(PdfError::File("Encrypt invalid U Params".to_string())),
        }

        if let Some(oe) = encrypt.get("OE") {
            match oe {
                PdfObject::LiteralString(s) => {
                    security_handler.oe = s.bytes().to_vec();
                }
                PdfObject::HexString(s) => {
                    security_handler.oe = s.raw_bytes()?;
                }
                _ => return Err(PdfError::File("OE in Encrypt is not a String".to_string())),
            }
        }
        if let Some(oe) = encrypt.get("UE") {
            match oe {
                PdfObject::LiteralString(s) => {
                    security_handler.ue = s.bytes().to_vec();
                }
                PdfObject::HexString(s) => {
                    security_handler.ue = s.raw_bytes()?;
                }
                _ => return Err(PdfError::File("UE in Encrypt is not a String".to_string())),
            }
        }

        let length = encrypt
            .get("Length")
            .ok_or(PdfError::File("Length is None in Encrypt".to_string()))?
            .as_number()?
            .integer();
        security_handler.length = (length / 8) as u32;
        security_handler.stream_filter =
            CryptFilter::new("V2".to_string(), security_handler.length);
        security_handler.string_filter =
            CryptFilter::new("V2".to_string(), security_handler.length);

        if matches!(v, 4 | 5) && matches!(r, 4 | 5 | 6) {
            if let Some(filters) = encrypt.get("CF") {
                if let Some(stream_filter) = encrypt.get("StmF") {
                    let name = stream_filter.as_name()?.name();
                    let f_entry = filters
                        .get_from_dict(name)
                        .ok_or(PdfError::File(
                            "StreamFilter in CryptFilter is not found in CF".to_string(),
                        ))?
                        .as_dict()?;
                    let filter = CryptFilter::try_new(f_entry)?;
                    security_handler.stream_filter = filter;
                }
                if let Some(string_filter) = encrypt.get("StrF") {
                    let name = string_filter.as_name()?.name();
                    let f_entry = filters
                        .get_from_dict(name)
                        .ok_or(PdfError::File(
                            "StringFilter in CryptFilter is not found in CF".to_string(),
                        ))?
                        .as_dict()?;
                    let filter = CryptFilter::try_new(f_entry)?;
                    security_handler.string_filter = filter;
                }
            }
        }
        if let Some(p) = encrypt.get("P") {
            let p = p.as_number()?.integer();
            security_handler.p = p;
        }
        if let Some(encrypt_metadata) = encrypt.get("EncryptMetadata") {
            security_handler.encrypt_metadata = encrypt_metadata.as_bool()?.0;
        } else {
            security_handler.encrypt_metadata = true;
        }
        let password = password.unwrap_or(&[]);
        security_handler.key = security_handler.verify(password)?;
        if security_handler.key.is_none() {
            return Err(PdfError::WrongPassword);
        }

        return Ok(security_handler);
    }

    pub fn decrypt_object(&self, obj: &PdfObject, id: u32, gen: u16) -> Result<PdfObject> {
        match obj {
            PdfObject::Stream(s) => {
                let mut s = s.to_owned();
                let data = s.raw_data();
                let df = self.stream_filter.decrypt_object(
                    self.key.as_ref().unwrap().as_slice(),
                    data,
                    id,
                    gen,
                    self.length as usize,
                )?;
                s.set_data(df);
                return Ok(PdfObject::Stream(s));
            }
            _ => {
                unimplemented!()
            }
        }
    }

    pub fn verify(&self, password: &[u8]) -> Result<Option<Vec<u8>>> {
        if self.rvision <= 4 {
            return self.verify_v4(password);
        } else {
            return self.verify_v5(password);
        }
    }
    fn verify_v5(&self, password: &[u8]) -> Result<Option<Vec<u8>>> {
        let key = self.verify_v5_owner_password(password)?;
        if key.is_none() {
            return self.verify_v5_user_passwrod(password);
        }
        // TODO perms
        return Ok(key);
    }

    fn verify_v5_user_passwrod(&self, password: &[u8]) -> Result<Option<Vec<u8>>> {
        let password = if password.len() > 128 {
            password[..127].to_vec()
        } else {
            password.to_vec()
        };
        let ch = self.calcute_hash_v5(password.as_slice(), &self.u[32..40], &[])?;
        if ch.as_slice() != &self.u[..32] {
            return Ok(None);
        }
        let iv = [0; 16];
        let tmp_key = self.calcute_hash_v5(password.as_slice(), &self.u[40..48], &[])?;
        let dkey = aes_cbc_decrypt(tmp_key.as_slice(), &iv, self.ue.as_slice());
        return Ok(Some(dkey));
    }

    fn verify_v5_owner_password(&self, password: &[u8]) -> Result<Option<Vec<u8>>> {
        let password = if password.len() > 127 {
            password[..127].to_vec()
        } else {
            password.to_owned()
        };
        let salt = &self.o[32..40];
        let ch = self.calcute_hash_v5(password.as_slice(), salt, &self.u[..48])?;
        if ch.as_slice() != &self.o[..32] {
            return Ok(None);
        }
        let salt = &self.o[40..48];
        let iv: [u8; 16] = [0; 16];
        let tmp_key = self.calcute_hash_v5(password.as_slice(), salt, &self.u[..48])?;
        let res = aes_cbc_decrypt(tmp_key.as_slice(), &iv, self.oe.as_slice());
        return Ok(Some(res));
    }
    fn calcute_hash_v5(&self, password: &[u8], salt: &[u8], u_data: &[u8]) -> Result<Vec<u8>> {
        let mut hasher = sha2::Sha256::new();
        hasher.update(password);
        hasher.update(salt);
        hasher.update(u_data);
        let mut k = hasher.finalize().to_vec();
        if self.rvision < 6 {
            return Ok(k);
        }
        let mut count: u32 = 0;
        loop {
            count += 1;
            let k1 = [password, k.as_slice(), u_data].concat();
            let mut data = Vec::new();
            for _ in 0..64 {
                data.extend(k1.iter());
            }
            let e = aes_cbc_encrypt(&k[..16], &k[16..32], data.as_slice());
            let t = e[..16]
                .iter()
                .map(|x| x.to_owned() as u32)
                .reduce(|a, b| a + b)
                .unwrap_or(0);
            let el = e.last().unwrap().to_owned();
            match t % 3 {
                0 => {
                    let mut ha = sha2::Sha256::new();
                    ha.update(e);
                    k = ha.finalize().to_vec();
                }
                1 => {
                    let mut ha = sha2::Sha384::new();
                    ha.update(e);
                    k = ha.finalize().to_vec();
                }
                2 => {
                    let mut ha = sha2::Sha512::new();
                    ha.update(e);
                    k = ha.finalize().to_vec();
                }
                _ => {
                    panic!("t value impossiable");
                }
            }
            if count >= 64 && (el as u32) + 32 <= count {
                break;
            }
        }
        Ok(k[..32].to_vec())
    }

    fn compute_o_value_key_v4(&self, password: &[u8]) -> Result<Vec<u8>> {
        let padded_password = padd_password(password);
        let mut hasher = Md5::new();
        hasher.update(padded_password);
        let mut res = hasher.finalize();

        if self.rvision >= 3 {
            for _ in 0..50 {
                let mut hasher = Md5::new();
                hasher.update(res);
                res = hasher.finalize();
            }
        }
        let mut result = Vec::new();
        for (i, v) in res.iter().enumerate() {
            if i >= self.length as usize {
                break;
            }
            result.push(v.to_owned());
        }
        Ok(result)
    }

    fn verify_v4(&self, password: &[u8]) -> Result<Option<Vec<u8>>> {
        if let Some(key) = self.verify_v4_owner_password(password)? {
            return Ok(Some(key));
        }
        if let Some(key) = self.verify_v4_user_password(password)? {
            return Ok(Some(key));
        }
        Ok(None)
    }

    fn verify_v4_owner_password(&self, password: &[u8]) -> Result<Option<Vec<u8>>> {
        let o_key = self.compute_o_value_key_v4(password)?;
        let user_password = if self.rvision <= 2 {
            rc4_decrypt(o_key.as_slice(), self.o.as_slice())
        } else {
            let mut user_password = self.o.clone();
            for i in (0..=19).rev() {
                let key = o_key.iter().map(|v| v ^ i).collect::<Vec<u8>>();
                user_password = rc4_decrypt(key.as_slice(), user_password.as_slice());
            }
            user_password
        };
        self.verify_v4_user_password(user_password.as_slice())
    }
    fn compute_key_v4(&self, password: &[u8]) -> Result<Vec<u8>> {
        let password = padd_password(password);
        let mut hasher = Md5::new();
        hasher.update(password);
        hasher.update(self.o.as_slice());
        hasher.update(self.p.to_le_bytes());
        hasher.update(self.id1.as_slice());
        if self.version >= 4 && !self.encrypt_metadata {
            hasher.update([255, 255, 255, 255]);
        }
        let hash_digest = hasher.finalize();
        let mut hash_digest = hash_digest.to_vec();

        if self.rvision >= 3 {
            for _ in 0..50 {
                let mut hasher = Md5::new();
                hasher.update(hash_digest[..self.length as usize].to_vec());
                hash_digest = hasher.finalize().to_vec();
            }
        }
        return Ok(hash_digest[..self.length as usize].to_vec());
    }
    fn compute_u_value_v4(&self, key: &[u8]) -> Result<Vec<u8>> {
        if self.rvision <= 2 {
            return Ok(rc4_decrypt(key, PASSWORD_PAD.to_vec().as_slice()));
        }
        let mut hasher = Md5::new();
        hasher.update(PASSWORD_PAD);
        hasher.update(self.id1.as_slice());
        let data = hasher.finalize();
        let data = data.to_vec();
        let mut rc4_enc = rc4_decrypt(key, data.as_slice());
        for i in 1..20 {
            let rc4_key = key.iter().map(|v| v ^ i).collect::<Vec<u8>>();
            rc4_enc = rc4_decrypt(rc4_key.as_slice(), rc4_enc.as_slice());
        }
        let rc4_enc = padd_password(rc4_enc.as_slice());
        Ok(rc4_enc.to_vec())
    }

    fn verify_v4_user_password(&self, user_password: &[u8]) -> Result<Option<Vec<u8>>> {
        let key = self.compute_key_v4(user_password)?;
        let mut u_value = self.compute_u_value_v4(key.as_slice())?;
        let mut u = self.u.clone();
        if self.rvision >= 3 {
            u_value = u_value[..16].to_vec();
            u = u[..16].to_vec();
        }
        if u_value != u {
            return Ok(None);
        }
        Ok(Some(key))
    }
}
