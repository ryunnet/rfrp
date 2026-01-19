use anyhow::Result;

/// Hash a password using bcrypt with cost 12
pub fn hash_password(password: &str) -> Result<String> {
    let cost = 12;
    bcrypt::hash(password, cost).map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    bcrypt::verify(password, hash).map_err(|e| anyhow::anyhow!("Failed to verify password: {}", e))
}

/// Generate a random password of specified length
pub fn generate_random_password(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
