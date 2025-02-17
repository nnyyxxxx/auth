use crate::{utils::constants::KEY_FILE, AuthError, AuthResult};
use age::{secrecy::ExposeSecret, x25519::Identity, Encryptor};
use std::{fs, io::Write, path::Path, str::FromStr};

pub struct Crypto {
    identity: Identity,
}

impl Crypto {
    pub fn new(auth_dir: &Path) -> AuthResult<Self> {
        let key_path = auth_dir.join(KEY_FILE);
        let identity = if key_path.exists() {
            let key_str = fs::read_to_string(&key_path)?;
            Identity::from_str(&key_str).map_err(|e| AuthError::InvalidKey(e.to_string()))?
        } else {
            let identity = Identity::generate();
            fs::write(&key_path, identity.to_string().expose_secret())?;
            identity
        };

        Ok(Self { identity })
    }

    pub fn encrypt(&self, data: &[u8]) -> AuthResult<Vec<u8>> {
        let recipient = self.identity.to_public();
        let encryptor =
            Encryptor::with_recipients(std::iter::once(&recipient as &dyn age::Recipient))
                .map_err(|_| AuthError::EncryptorError)?;

        let mut encrypted = vec![];
        let mut writer = encryptor.wrap_output(&mut encrypted)?;
        writer.write_all(data)?;
        writer.finish()?;

        Ok(encrypted)
    }

    pub fn decrypt(&self, data: &[u8]) -> AuthResult<Vec<u8>> {
        let decryptor = age::Decryptor::new(data)?;
        let mut decrypted = vec![];
        let mut reader =
            decryptor.decrypt(std::iter::once(&self.identity as &dyn age::Identity))?;
        std::io::copy(&mut reader, &mut decrypted)?;
        Ok(decrypted)
    }
}
