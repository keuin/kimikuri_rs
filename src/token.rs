pub fn generate() -> String {
    let mut rng = urandom::csprng();
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);
    bs58::encode(bytes).into_string()
}