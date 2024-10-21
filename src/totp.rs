use base32::Alphabet;
use totp_rs::{Algorithm, TOTP};

pub fn generate_totp(secret: &str, time: u64) -> Result<String, String> {
    let decoded_secret = base32::decode(Alphabet::RFC4648 { padding: false }, secret)
        .ok_or_else(|| "Failed to decode secret".to_string())?;

    let padded_secret = if decoded_secret.len() < 16 {
        let mut padded = decoded_secret;
        padded.resize(16, 0);
        padded
    } else {
        decoded_secret
    };

    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, padded_secret)
        .map_err(|e| format!("Failed to create TOTP: {}", e))?;

    Ok(totp.generate(time))
}
