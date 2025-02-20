use auth::{
    auth_core::app::App,
    utils::constants::{
        TEST_ABSOLUTE_PATH, TEST_ENV_FILE, TEST_ENV_PATH, TEST_ENV_VAR, TEST_HOME_FILE,
    },
};
use std::{env, path::PathBuf};

#[test]
fn test_home_path_expansion() {
    let app = App::new().unwrap();
    let home = dirs::home_dir().unwrap();

    let result = app.expand_path(&format!("~/{}", TEST_HOME_FILE));
    assert_eq!(result, home.join(TEST_HOME_FILE));
}

#[test]
fn test_env_var_expansion() {
    let app = App::new().unwrap();
    unsafe {
        env::set_var(TEST_ENV_VAR, TEST_ENV_PATH);
    }

    let result = app.expand_path(&format!("${}/{}", TEST_ENV_VAR, TEST_ENV_FILE));
    assert_eq!(
        result,
        PathBuf::from(format!("{}/{}", TEST_ENV_PATH, TEST_ENV_FILE))
    );
}

#[test]
fn test_absolute_path() {
    let app = App::new().unwrap();

    let result = app.expand_path(TEST_ABSOLUTE_PATH);
    assert_eq!(result, PathBuf::from(TEST_ABSOLUTE_PATH));
}
