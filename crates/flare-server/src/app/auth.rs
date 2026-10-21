use rand::Rng;

pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    format!("flare_{}", hex::encode(bytes))
}
