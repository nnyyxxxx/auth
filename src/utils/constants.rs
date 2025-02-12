pub const MIN_WIDTH: u16 = 110;
pub const MIN_HEIGHT: u16 = 31;
pub const TOTP_PERIOD: u64 = 30;
pub const TOTP_DIGITS: usize = 6;
pub const TOTP_STEP: u8 = 1;
pub const AUTH_DIR_NAME: &str = ".local/share/auth";
pub const ENTRIES_FILE: &str = "entries.toml";
pub const KEY_FILE: &str = "key";
pub const WARNING_TITLE: &str = " Warning ";
pub const AUTH_TITLE: &str = " Auth ";
pub const COPIED_MSG: &str = " Copied! ";
pub const ADD_ENTRY_TITLE: &str = " Add Entry ";
pub const EDIT_ENTRY_TITLE: &str = " Edit Entry ";
pub const IMPORT_TITLE: &str = " Import ";
pub const EXPORT_TITLE: &str = " Export ";
pub const BINDINGS_TITLE: &str = " Bindings ";
pub const NAME_LABEL: &str = "Name:";
pub const SECRET_LABEL: &str = "Secret:";
pub const PATH_LABEL: &str = "Path:";
pub const INVALID_LABEL: &str = "Invalid";
pub const NOTIFICATION_TIMEOUT: u64 = 3;
pub const CLIPBOARD_SLEEP_DURATION: u64 = 100;
pub const EVENT_POLL_DURATION: u64 = 50;
pub const VERTICAL_PADDING_DIVISOR: u16 = 2;
pub const POPUP_WIDTH_PERCENT: u16 = 60;
pub const POPUP_HEIGHT_PERCENT: u16 = 20;
pub const TOTAL_PERCENTAGE: u16 = 100;
pub const SECRET_PADDING_BLOCK: usize = 8;
pub const SECRET_PAD_CHAR: char = '=';
pub const MIN_SECRET_LENGTH: usize = 16;
pub const FIRST_ENTRY_ROW: usize = 1;
pub const NAME_FIELD: usize = 0;
pub const SECRET_FIELD: usize = 1;
pub const HOME_PREFIX_LEN: usize = 2;
pub const LAST_ENTRY_INDEX: usize = 1;
pub const ENV_VAR_OFFSET: usize = 1;
pub const SINGLE_CHAR_PATH: usize = 1;
pub const PATH_SEPARATOR_OFFSET: usize = 1;
pub const LAST_ENTRY_OFFSET: usize = 1;
pub const NEXT_ENTRY_STEP: usize = 1;
pub const INVALID_REMAINING_TIME: u64 = 0;
pub const PADDING_BYTE: u8 = 0;
pub const REMAINDER_ZERO: usize = 0;
pub const DEFAULT_NAME_WIDTH: usize = 0;
pub const MIN_BLOCK_HEIGHT: u16 = 3;
pub const HELP_BLOCK_HEIGHT: u16 = 3;
pub const CODE_WIDTH: usize = 6;
pub const REMAINING_WIDTH: usize = 1;
pub const NAME_PADDING: usize = 2;
pub const CURSOR_CHAR: &str = "|";
pub const EMPTY_CURSOR: &str = "";
pub const XCLIP_SELECTION_ARG: &str = "-selection";
pub const XCLIP_CLIPBOARD_ARG: &str = "clipboard";
pub const XCLIP_IN_ARG: &str = "-in";
pub const XCLIP_COMMAND: &str = "xclip";
pub const WAYLAND_COPY_COMMAND: &str = "wl-copy";
pub const TOML_EXT: &str = "toml";
pub const WAYLAND_DISPLAY: &str = "WAYLAND_DISPLAY";
pub const TEST_HOME_FILE: &str = "test.toml";
pub const TEST_ENV_PATH: &str = "/tmp/test";
pub const TEST_ENV_VAR: &str = "TEST_PATH";
pub const TEST_ENV_FILE: &str = "file.toml";
pub const TEST_ABSOLUTE_PATH: &str = "/absolute/path/file.toml";
pub const TEST_VALID_NAME: &str = "Test Entry";
pub const TEST_VALID_SECRET: &str = "JBSWY3DPEHPK3PXP";
pub const TEST_INVALID_NAME: &str = "Invalid Entry";
pub const TEST_INVALID_SECRET: &str = "invalid!@#$";
pub const TEST_PADDED_SECRET: &str = "JBSW";
pub const TEST_NORMALIZED_SECRET: &str = "jbsw y3dp";
pub const TEST_CODE_LENGTH: usize = 6;
pub const TEST_MAX_REMAINING: u64 = 30;
pub const TEST_MIN_REMAINING: u64 = 1;

pub const ROOT_WARNING: &[&str] = &[
    "Running as root is not supported",
    "",
    "Press any key to exit",
];

pub const SIZE_WARNING: &[&str] = &[
    "Terminal size too small:",
    "Width = {} Height = {}",
    "",
    "Needed to display properly:",
    "Width = {} Height = {}",
];

pub const HELP_TEXT: &str = "a: add  E: edit  d: del  i: import  e: export  ↑/k: up  ↓/j: down  enter: copy  q: quit  tab: cycle fields";

pub const LINUX_WARNING: &[&str] = &["Only Linux is supported", "", "Press any key to exit"];
