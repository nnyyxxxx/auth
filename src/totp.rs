use totp_rs::{Secret, TOTP};

pub fn generate_totp(secret: &str, time: u64) -> String {
    let totp = TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Encoded(secret.to_string()).to_bytes().unwrap(),
    )
    .unwrap();

    totp.generate(time)
}
