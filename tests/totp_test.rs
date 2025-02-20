use auth::{
    auth_core::totp::generate_totp,
    utils::constants::{
        TEST_CODE_LENGTH, TEST_INVALID_SECRET, TEST_MAX_REMAINING, TEST_MIN_REMAINING,
        TEST_NORMALIZED_SECRET, TEST_PADDED_SECRET, TEST_VALID_SECRET,
    },
};

#[test]
fn test_valid_totp_generation() {
    let result = generate_totp(TEST_VALID_SECRET);
    assert!(result.is_ok());

    let (code, remaining) = result.unwrap();
    assert_eq!(code.len(), TEST_CODE_LENGTH);
    assert!(remaining <= TEST_MAX_REMAINING);
    assert!(remaining >= TEST_MIN_REMAINING);
}

#[test]
fn test_invalid_totp_secret() {
    let result = generate_totp(TEST_INVALID_SECRET);
    assert!(result.is_err());
}

#[test]
fn test_empty_totp_secret() {
    let result = generate_totp("");
    assert!(result.is_err(), "Empty secret should fail base32 decoding");
}

#[test]
fn test_padded_totp_secret() {
    let result = generate_totp(TEST_PADDED_SECRET);
    assert!(result.is_ok(), "Padded secret should be valid");

    let (code, remaining) = result.unwrap();
    assert_eq!(code.len(), TEST_CODE_LENGTH);
    assert!(remaining <= TEST_MAX_REMAINING);
}

#[test]
fn test_normalized_secret() {
    let result = generate_totp(TEST_NORMALIZED_SECRET);
    assert!(result.is_ok(), "Normalized secret should be valid");

    let (code, remaining) = result.unwrap();
    assert_eq!(code.len(), TEST_CODE_LENGTH);
    assert!(remaining <= TEST_MAX_REMAINING);
}
