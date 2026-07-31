use openssl::sha::sha256;

pub fn digest(payload: &[u8]) -> [u8; 32] {
    sha256(payload)
}
