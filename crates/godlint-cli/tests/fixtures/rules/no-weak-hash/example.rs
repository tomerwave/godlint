fn sign(payload: &[u8]) -> md5::Digest {
    md5::compute(payload)
}
