use errors::*;

use bit_field::BitField;

pub struct AuthSessionStart {
    pub request_id: u32,
    pub hostkey: Vec<u8>
}
/* 4B Reserved | 4B RequestId | Rest Hostkey */
impl AuthSessionStart {
    pub fn encode(self) -> Result<Vec<u8>> {
        let mut bytes = pack_structure!("4xI", self.request_id);
        bytes.extend_from_slice(&self.hostkey);
        Ok(bytes)
    }
}

pub struct AuthSessionHS {
    pub session_id: u16,
    pub request_id: u32,
    pub payload: Vec<u8>
}
/* 2B Reserved | 2B SessionId | 4B RequestId | Rest Payload */
impl AuthSessionHS {
    pub fn decode(bytes: Vec<u8>) -> Result<AuthSessionHS> {
        let (session_id, request_id) = unpack_structure!("2xHI", &bytes[0..8]);
        Ok(AuthSessionHS {
            session_id: session_id,
            request_id: request_id,
            payload: bytes[8..].to_vec()
        })
    }
    pub fn encode(self) -> Result<Vec<u8>> {
        let mut bytes = pack_structure!("2xHI", self.session_id, self.request_id);
        bytes.extend_from_slice(&self.payload);
        Ok(bytes)
    }
}

pub struct AuthSessionHS1Response {
    pub request_id: u32,
    pub payload: Vec<u8>
}
impl AuthSessionHS1Response {
    pub fn encode(self) -> Result<Vec<u8>> {
        let mut bytes = pack_structure!("4xI", self.request_id);
        bytes.extend_from_slice(&self.payload);
        Ok(bytes)
    }
}

pub struct AuthCipherCrypt {
    pub session_id: u16,
    pub request_id: u32,
    pub cleartext: bool,
    pub payload: Vec<u8>
}
impl AuthCipherCrypt {
    pub fn encode(self) -> Result<Vec<u8>> {
        let mut bytes = pack_structure!("3xBIH", boolean!(self.cleartext), self.request_id, self.session_id);
        bytes.extend_from_slice(&self.payload);
        Ok(bytes)
    }
}

pub struct AuthCipherCryptResp {
    pub request_id: u32,
    pub cleartext: bool,
    pub payload: Vec<u8>
}
/* 3B Reserved | 7b1b Cleartext | 4B RequestId | Rest Payload */
impl AuthCipherCryptResp {
    pub fn decode(bytes: Vec<u8>) -> Result<AuthCipherCryptResp> {
        let (cleartext, request_id,) = unpack_structure!("3xBI", &bytes);
        Ok(AuthCipherCryptResp {
            request_id: request_id,
            cleartext: cleartext.get_bit(0),
            payload: bytes[8..].to_vec()
        })
    }
}

pub struct AuthSessionClose {
    pub session_id: u16
}
impl AuthSessionClose {
    pub fn encode(self) -> Result<Vec<u8>> {
        Ok(pack_structure!("2xH", self.session_id))
    }
}

pub struct AuthSessionError {
    pub request_id: u32
}
/* 4B Reserved | 4B RequestId */
impl AuthSessionError {
    pub fn decode(bytes: Vec<u8>) -> Result<AuthSessionError> {
        let (request_id,) = unpack_structure!("4xI", &bytes);
        Ok(AuthSessionError {
            request_id: request_id
        })
    }
}

pub enum Auth {
    SessionStart(AuthSessionStart),
    SessionHS1(AuthSessionHS),
    SessionIncommingHS1(AuthSessionHS1Response),
    SessionHS2(AuthSessionHS),
    SessionIncommingHS2(AuthSessionHS),
    CipherEncrypt(AuthCipherCrypt),
    CipherEncryptResp(AuthCipherCryptResp),
    CipherDecrypt(AuthCipherCrypt),
    CipherDecryptResp(AuthCipherCryptResp),
    SessionClose(AuthSessionClose),
    SessionError(AuthSessionError)
}
