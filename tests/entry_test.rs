use auth::{
    auth_core::entry::Entry,
    utils::constants::{
        INVALID_LABEL, INVALID_REMAINING_TIME, TEST_CODE_LENGTH, TEST_INVALID_NAME,
        TEST_INVALID_SECRET, TEST_MAX_REMAINING, TEST_VALID_NAME, TEST_VALID_SECRET,
    },
};

#[test]
fn test_valid_entry_totp() {
    let entry = Entry {
        name: TEST_VALID_NAME.to_string(),
        secret: TEST_VALID_SECRET.to_string(),
    };

    let (code, remaining) = entry.generate_totp_with_time();
    assert_eq!(code.len(), TEST_CODE_LENGTH);
    assert!(code.chars().all(|c| c.is_ascii_digit()));
    assert!(remaining <= TEST_MAX_REMAINING);
}

#[test]
fn test_invalid_entry_totp() {
    let entry = Entry {
        name: TEST_INVALID_NAME.to_string(),
        secret: TEST_INVALID_SECRET.to_string(),
    };

    let (code, remaining) = entry.generate_totp_with_time();
    assert_eq!(code, INVALID_LABEL);
    assert_eq!(remaining, INVALID_REMAINING_TIME);
}
