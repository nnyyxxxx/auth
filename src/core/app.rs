use crate::{
    AuthError, AuthResult,
    core::{
        crypto::Crypto,
        entry::{Entries, Entry},
    },
    input::mouse,
    utils::{
        clipboard::copy_to_clipboard,
        constants::{
            AUTH_DIR_NAME, ENTRIES_FILE, ENV_VAR_OFFSET, HOME_PREFIX_LEN, LAST_ENTRY_INDEX,
            LAST_ENTRY_OFFSET, NAME_FIELD, NEXT_ENTRY_STEP, PATH_SEPARATOR_OFFSET, SECRET_FIELD,
            SINGLE_CHAR_PATH, TOML_EXT,
        },
    },
};
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(PartialEq, Clone)]
pub enum InputMode {
    Normal,
    Adding,
    Editing,
    Importing,
    Exporting,
}

pub struct App {
    pub should_quit: bool,
    pub entries: Vec<Entry>,
    pub selected: usize,
    pub input_mode: InputMode,
    pub new_entry_name: String,
    pub new_entry_secret: String,
    pub edit_entry_name: String,
    pub edit_entry_secret: String,
    pub input_field: usize,
    pub entries_path: PathBuf,
    pub copy_notification_time: Option<SystemTime>,
    pub path_input: String,
    pub error_message: Option<(String, SystemTime)>,
    crypto: Crypto,
}

impl App {
    pub fn new() -> AuthResult<Self> {
        let auth_dir = Self::get_auth_directory()?;
        let entries_path = auth_dir.join(ENTRIES_FILE);
        let crypto = Self::initialize_crypto(&auth_dir)?;
        let mut app = Self::create_initial_app(entries_path, crypto);

        Self::try_load_entries(&mut app);
        Ok(app)
    }

    fn get_auth_directory() -> AuthResult<PathBuf> {
        let home = dirs::home_dir().ok_or(AuthError::HomeDirError)?;
        let auth_dir = home.join(AUTH_DIR_NAME);

        fs::create_dir_all(&auth_dir).map_err(|_| AuthError::CreateDirError)?;

        Ok(auth_dir)
    }

    fn initialize_crypto(auth_dir: &Path) -> AuthResult<Crypto> {
        Crypto::new(auth_dir).map_err(|_| AuthError::CryptoInitError)
    }

    fn create_initial_app(entries_path: PathBuf, crypto: Crypto) -> App {
        App {
            should_quit: false,
            entries: Vec::new(),
            selected: 0,
            input_mode: InputMode::Normal,
            new_entry_name: String::new(),
            new_entry_secret: String::new(),
            edit_entry_name: String::new(),
            edit_entry_secret: String::new(),
            input_field: NAME_FIELD,
            entries_path,
            copy_notification_time: None,
            path_input: String::new(),
            error_message: None,
            crypto,
        }
    }

    fn try_load_entries(app: &mut App) {
        if app.load_entries().is_err() {
            app.show_error(&AuthError::ReadError.to_string());
        }
    }

    fn load_entries(&mut self) -> AuthResult<()> {
        if !self.entries_path.exists() {
            return Ok(());
        }

        let encrypted = self.read_encrypted_file()?;
        let decrypted = self.decrypt_data(&encrypted)?;
        let contents = self.parse_utf8(&decrypted)?;
        self.parse_entries(&contents)?;

        Ok(())
    }

    fn read_encrypted_file(&mut self) -> AuthResult<Vec<u8>> {
        fs::read(&self.entries_path).map_err(|_| {
            self.show_error(&AuthError::ReadError.to_string());
            AuthError::ReadError
        })
    }

    fn decrypt_data(&mut self, encrypted: &[u8]) -> AuthResult<Vec<u8>> {
        self.crypto.decrypt(encrypted).map_err(|_| {
            self.show_error(&AuthError::DecryptError.to_string());
            AuthError::DecryptError
        })
    }

    fn parse_utf8(&mut self, data: &[u8]) -> AuthResult<String> {
        String::from_utf8(data.to_vec()).map_err(|_| {
            self.show_error(&AuthError::Utf8Error.to_string());
            AuthError::Utf8Error
        })
    }

    fn parse_entries(&mut self, contents: &str) -> AuthResult<()> {
        let entries = toml::from_str::<Entries>(contents).map_err(|_| {
            self.show_error(&AuthError::ParseError.to_string());
            AuthError::ParseError
        })?;

        self.entries = entries.entries;
        Ok(())
    }

    pub fn save_entries(&mut self) -> AuthResult<()> {
        let contents = self.serialize_entries()?;
        let encrypted = self.encrypt_contents(&contents)?;
        self.write_encrypted_file(&encrypted)?;
        Ok(())
    }

    fn serialize_entries(&mut self) -> AuthResult<String> {
        let entries = Entries {
            entries: self.entries.clone(),
        };

        toml::to_string_pretty(&entries).map_err(|_| {
            self.show_error(&AuthError::SerializeError.to_string());
            AuthError::SerializeError
        })
    }

    fn encrypt_contents(&mut self, contents: &str) -> AuthResult<Vec<u8>> {
        self.crypto.encrypt(contents.as_bytes()).map_err(|_| {
            self.show_error(&AuthError::EncryptorError.to_string());
            AuthError::EncryptorError
        })
    }

    fn write_encrypted_file(&mut self, encrypted: &[u8]) -> AuthResult<()> {
        fs::write(&self.entries_path, encrypted).map_err(|_| {
            self.show_error(&AuthError::WriteError.to_string());
            AuthError::WriteError
        })
    }

    pub fn add_entry(&mut self) -> AuthResult<()> {
        if self.new_entry_name.is_empty() || self.new_entry_secret.is_empty() {
            self.show_error(&AuthError::EmptyEntryError.to_string());
            return Ok(());
        }
        self.entries.push(Entry {
            name: self.new_entry_name.clone(),
            secret: self.new_entry_secret.clone(),
        });
        if self.save_entries().is_err() {
            self.show_error(&AuthError::SaveError.to_string());
        }
        Ok(())
    }

    pub fn delete_entry(&mut self) {
        let Some(entries) = (!self.entries.is_empty()).then_some(&mut self.entries) else {
            return;
        };

        entries.remove(self.selected);
        self.selected = self
            .selected
            .min(entries.len().saturating_sub(LAST_ENTRY_OFFSET));

        if self.save_entries().is_err() {
            self.show_error(&AuthError::SaveError.to_string());
        }
    }

    pub fn copy_current_code(&mut self) -> AuthResult<()> {
        if self.entries.is_empty() {
            return Ok(());
        }

        self.try_copy_code()?;
        Ok(())
    }

    fn try_copy_code(&mut self) -> AuthResult<()> {
        let entry = &self.entries[self.selected];
        let (code, _) = entry.generate_totp_with_time();

        if copy_to_clipboard(code).is_err() {
            self.show_error(&AuthError::ClipboardError.to_string());
            return Ok(());
        }

        self.copy_notification_time = Some(SystemTime::now());
        Ok(())
    }

    pub fn show_error(&mut self, message: &str) {
        self.error_message = Some((message.to_string(), SystemTime::now()));
    }

    pub fn expand_path(&self, path: &str) -> PathBuf {
        if path.starts_with('~') {
            return self.expand_home_path(path);
        }

        if let Some(stripped) = path.strip_prefix('$') {
            return self.expand_env_path(stripped, path);
        }

        PathBuf::from(path)
    }

    fn expand_home_path(&self, path: &str) -> PathBuf {
        let Some(home) = dirs::home_dir() else {
            return PathBuf::from(path);
        };

        if path.len() == SINGLE_CHAR_PATH {
            return home;
        }

        home.join(&path[HOME_PREFIX_LEN..])
    }

    fn expand_env_path(&self, stripped: &str, original_path: &str) -> PathBuf {
        let var_end = Self::get_var_end(stripped, original_path);
        let (var, rest) = stripped.split_at(var_end - ENV_VAR_OFFSET);
        let expanded_path = Self::expand_env_var(var, rest);
        expanded_path.unwrap_or_else(|| PathBuf::from(original_path))
    }

    fn get_var_end(stripped: &str, original_path: &str) -> usize {
        stripped
            .find('/')
            .map(|i| i + PATH_SEPARATOR_OFFSET)
            .unwrap_or(original_path.len())
    }

    fn expand_env_var(var: &str, rest: &str) -> Option<PathBuf> {
        env::var(var)
            .ok()
            .map(|val| PathBuf::from(val).join(rest.trim_start_matches('/')))
    }

    pub fn import_entries(&mut self) -> AuthResult<()> {
        if self.path_input.is_empty() {
            return Ok(());
        }

        let path = self.get_validated_import_path()?;
        let entries = self.read_and_parse_entries(&path)?;
        self.merge_and_save_entries(entries)?;

        Ok(())
    }

    fn get_validated_import_path(&mut self) -> AuthResult<PathBuf> {
        let path = self.expand_path(&self.path_input);
        self.validate_import_path(&path)?;
        Ok(path)
    }

    fn read_and_parse_entries(&mut self, path: &Path) -> AuthResult<Entries> {
        if !self.validate_file_exists(path) {
            return Ok(Entries { entries: vec![] });
        }

        let contents = self.read_file_contents(path);
        self.parse_toml_contents(contents)
    }

    fn validate_file_exists(&mut self, path: &Path) -> bool {
        if !path.exists() {
            self.show_error(&AuthError::FileNotExistError.to_string());
            return false;
        }
        true
    }

    fn read_file_contents(&mut self, path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_else(|_| {
            self.show_error(&AuthError::ReadError.to_string());
            String::new()
        })
    }

    fn parse_toml_contents(&mut self, contents: String) -> AuthResult<Entries> {
        if contents.is_empty() {
            return Ok(Entries { entries: vec![] });
        }

        Ok(toml::from_str(&contents).unwrap_or_else(|_| {
            self.show_error(&AuthError::ParseError.to_string());
            Entries { entries: vec![] }
        }))
    }

    fn merge_and_save_entries(&mut self, entries: Entries) -> AuthResult<()> {
        self.entries.extend(entries.entries);

        if self.save_entries().is_err() {
            self.show_error(&AuthError::SaveError.to_string());
        }

        Ok(())
    }

    fn validate_import_path(&mut self, path: &Path) -> AuthResult<()> {
        if !path.exists() {
            self.show_error(&AuthError::FileNotExistError.to_string());
            return Ok(());
        }

        if path.is_dir() {
            self.show_error(&AuthError::DirectoryError.to_string());
            return Ok(());
        }

        if path.extension().is_none_or(|ext| ext != TOML_EXT) {
            self.show_error(&AuthError::TomlExtError.to_string());
            return Ok(());
        }

        Ok(())
    }

    pub fn export_entries(&mut self) -> AuthResult<()> {
        if self.path_input.is_empty() {
            self.show_error(&AuthError::NoFilenameError.to_string());
            return Ok(());
        }

        if self.entries.is_empty() {
            self.show_error(&AuthError::EmptyExportError.to_string());
            return Ok(());
        }

        let path = self.get_validated_export_path()?;
        let contents = self.serialize_entries()?;
        self.write_export_file(&path, &contents)?;

        Ok(())
    }

    fn get_validated_export_path(&mut self) -> AuthResult<PathBuf> {
        let mut path = self.expand_path(&self.path_input);

        if path.is_dir() || self.path_input.ends_with('/') || self.path_input.ends_with('\\') {
            self.show_error(&AuthError::NoFilenameError.to_string());
            return Ok(path);
        }

        if !path.to_string_lossy().ends_with(&format!(".{}", TOML_EXT)) {
            path.set_extension(TOML_EXT);
        }
        Ok(path)
    }

    fn write_export_file(&mut self, path: &Path, contents: &str) -> AuthResult<()> {
        if path.is_dir() {
            self.show_error(&AuthError::DirectoryError.to_string());
            return Ok(());
        }

        fs::write(path, contents).map_err(|_| {
            self.show_error(&AuthError::WriteError.to_string());
            AuthError::WriteError
        })
    }

    pub fn handle_events(&mut self, event: Event) -> AuthResult<()> {
        match event {
            Event::Key(key) => self.handle_key_event(key),
            Event::Mouse(mouse) => mouse::handle_mouse_event(self, mouse),
            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> AuthResult<()> {
        if self.check_control_quit(key) {
            return Ok(());
        }

        match self.input_mode {
            InputMode::Normal => self.handle_normal_mode(key),
            InputMode::Adding | InputMode::Editing => self.handle_entry_mode(key),
            InputMode::Importing | InputMode::Exporting => self.handle_file_mode(key),
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> AuthResult<()> {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => self.next_entry(),
            KeyCode::Char('k') | KeyCode::Up => self.previous_entry(),
            KeyCode::Char('a') => self.input_mode = InputMode::Adding,
            KeyCode::Char('E') => self.start_editing(),
            KeyCode::Char('d') => self.delete_entry(),
            KeyCode::Char('i') => self.input_mode = InputMode::Importing,
            KeyCode::Char('e') => self.input_mode = InputMode::Exporting,
            KeyCode::Enter => self.copy_current_code()?,
            _ => {}
        }
        Ok(())
    }

    fn check_control_quit(&mut self, key: KeyEvent) -> bool {
        if matches!(key.code, KeyCode::Char('q') | KeyCode::Char('c'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
        {
            self.should_quit = true;
            return true;
        }
        false
    }

    fn handle_entry_mode(&mut self, key: KeyEvent) -> AuthResult<()> {
        match key.code {
            KeyCode::Esc => self.reset_entry_state(),
            KeyCode::Enter => self.process_entry_input()?,
            KeyCode::Char(c) => self.update_entry_field(c),
            KeyCode::Backspace => self.remove_entry_char(),
            KeyCode::Tab => self.handle_tab_key(key.modifiers.contains(KeyModifiers::SHIFT)),
            _ => {}
        }
        Ok(())
    }

    fn handle_tab_key(&mut self, is_shift: bool) {
        self.input_field = match (self.input_field, is_shift) {
            (NAME_FIELD, true) | (SECRET_FIELD, false) => NAME_FIELD,
            (NAME_FIELD, false) | (SECRET_FIELD, true) => SECRET_FIELD,
            _ => unreachable!(),
        };
    }

    fn handle_file_mode(&mut self, key: KeyEvent) -> AuthResult<()> {
        match key.code {
            KeyCode::Esc => self.reset_file_mode(),
            KeyCode::Enter => self.process_file_mode_input()?,
            KeyCode::Char(c) => self.path_input.push(c),
            KeyCode::Backspace => {
                self.path_input.pop();
            }
            _ => {}
        }
        Ok(())
    }

    fn reset_file_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.path_input.clear();
    }

    fn process_file_mode_input(&mut self) -> AuthResult<()> {
        self.handle_file_operation()?;
        self.reset_file_mode();
        Ok(())
    }

    fn handle_file_operation(&mut self) -> AuthResult<()> {
        match self.input_mode {
            InputMode::Importing => self.import_entries(),
            InputMode::Exporting => self.export_entries(),
            _ => unreachable!(),
        }
    }

    fn reset_entry_state(&mut self) {
        let fields = match self.input_mode {
            InputMode::Adding => (&mut self.new_entry_name, &mut self.new_entry_secret),
            _ => (&mut self.edit_entry_name, &mut self.edit_entry_secret),
        };

        self.input_mode = InputMode::Normal;
        fields.0.clear();
        fields.1.clear();
        self.input_field = NAME_FIELD;
    }

    fn process_entry_input(&mut self) -> AuthResult<()> {
        if self.is_name_field() {
            self.switch_to_secret_field();
            return Ok(());
        }
        self.handle_final_entry_input()
    }

    fn is_name_field(&self) -> bool {
        self.input_field == NAME_FIELD
    }

    fn handle_final_entry_input(&mut self) -> AuthResult<()> {
        match self.input_mode {
            InputMode::Adding => self.handle_add_entry()?,
            InputMode::Editing => self.handle_edit_entry()?,
            _ => unreachable!(),
        }
        self.reset_input_field();
        Ok(())
    }

    fn handle_add_entry(&mut self) -> AuthResult<()> {
        self.add_entry()?;
        self.reset_entry_state();
        Ok(())
    }

    fn handle_edit_entry(&mut self) -> AuthResult<()> {
        self.edit_entry()?;
        self.reset_entry_state();
        Ok(())
    }

    fn reset_input_field(&mut self) {
        self.input_field = NAME_FIELD;
    }

    fn switch_to_secret_field(&mut self) {
        self.input_field = SECRET_FIELD;
    }

    fn update_entry_field(&mut self, c: char) {
        let field = self.get_current_field();
        field.push(c);
    }

    fn remove_entry_char(&mut self) {
        let field = self.get_current_field();
        field.pop();
    }

    fn get_current_field(&mut self) -> &mut String {
        match (self.input_mode.clone(), self.input_field == NAME_FIELD) {
            (InputMode::Adding, true) => &mut self.new_entry_name,
            (InputMode::Adding, false) => &mut self.new_entry_secret,
            (_, true) => &mut self.edit_entry_name,
            (_, false) => &mut self.edit_entry_secret,
        }
    }

    fn edit_entry(&mut self) -> AuthResult<()> {
        if self.entries.is_empty() {
            return Ok(());
        }

        if !self.validate_edit_entry()? {
            return Ok(());
        }

        self.update_entry();
        self.try_save_entries();
        Ok(())
    }

    fn validate_edit_entry(&mut self) -> AuthResult<bool> {
        if self.edit_entry_name.is_empty() || self.edit_entry_secret.is_empty() {
            self.show_error(&AuthError::EmptyEntryError.to_string());
            return Ok(false);
        }
        Ok(true)
    }

    fn update_entry(&mut self) {
        self.entries[self.selected] = Entry {
            name: self.edit_entry_name.clone(),
            secret: self.edit_entry_secret.clone(),
        };
    }

    fn try_save_entries(&mut self) {
        if self.save_entries().is_err() {
            self.show_error(&AuthError::SaveError.to_string());
        }
    }

    fn start_editing(&mut self) {
        let Some(entry) = (!self.entries.is_empty()).then_some(&self.entries[self.selected]) else {
            return;
        };

        self.edit_entry_name = entry.name.clone();
        self.edit_entry_secret = entry.secret.clone();
        self.input_mode = InputMode::Editing;
        self.input_field = NAME_FIELD;
    }

    fn next_entry(&mut self) {
        let Some(len) = (!self.entries.is_empty()).then_some(self.entries.len()) else {
            return;
        };
        self.selected = (self.selected + NEXT_ENTRY_STEP) % len;
    }

    fn previous_entry(&mut self) {
        let Some(len) = (!self.entries.is_empty()).then_some(self.entries.len()) else {
            return;
        };
        self.selected = self
            .selected
            .checked_sub(LAST_ENTRY_INDEX)
            .unwrap_or(len - LAST_ENTRY_INDEX);
    }
}
