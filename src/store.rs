use aes_gcm_siv::{self, Aes256GcmSiv, Key, Nonce};
use aes_gcm_siv::aead::{self, Aead, NewAead};
use crate::TotpProfile;
use rand::{Rng, thread_rng};
use std::collections::{hash_map::Entry, HashMap};
use std::fs;
use std::path::PathBuf;
use std::io::{self, Write};

#[derive(Debug, thiserror::Error)]
pub enum StoreLoadError {
    #[error("file could not be read")]
    CannotReadFile(#[from] io::Error),
    #[error("failed to decrypt store file")]
    CannotDecrypt(#[from] aead::Error),
    #[error("failed to deserialize store file")]
    CannotDeserialize(#[from] csv::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum StoreSaveError {
    #[error("file could not be written")]
    CannotWriteFile(#[from] io::Error),
    #[error("failed to encrypt store file")]
    CannotDecrypt(#[from] aead::Error),
    #[error("failed to serialize data")]
    CannotSerialize(#[from] csv::Error),
}

#[derive(Debug, thiserror::Error)]
#[error("profile {0} not found in store")]
pub struct ProfileNotFoundError(String);

#[derive(Debug, thiserror::Error)]
#[error("profile already exists with name {0}")]
pub struct ExistingProfileError(String);

pub struct ProfileStore {
    path: PathBuf,
    profiles: HashMap<String, TotpProfile>,
    key: [u8; 32],
}

impl ProfileStore {
    pub fn new(path: PathBuf, key: [u8; 32]) -> Result<Self, StoreSaveError> {
        let store = Self {
            path,
            profiles: HashMap::new(),
            key,
        };
        Ok(store)
    }

    pub fn load(path: PathBuf, key: [u8; 32]) -> Result<Self, StoreLoadError> {
        let data = fs::read(&path)?;

        let cipher_key = Key::from_slice(&key);
        let cipher = Aes256GcmSiv::new(cipher_key);
        let nonce = Nonce::from_slice(&data[..12]);

        let decrypted = cipher.decrypt(nonce, data[12..].as_ref())?;

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(decrypted.as_slice());

        let mut profiles = HashMap::new();
        for profile in reader.deserialize() {
            let profile: TotpProfile = profile?;
            profiles.insert(profile.name.clone(), profile);
        }

        Ok(Self { path, profiles, key })
    }

    pub fn get(&self, name: &str) -> Option<&TotpProfile> {
        self.profiles.get(name)
    }

    pub fn add(&mut self, new_profile: TotpProfile) -> Result<(), ExistingProfileError> {
        let name = new_profile.name.clone();
        if let Entry::Vacant(e) = self.profiles.entry(name.clone()) {
            e.insert(new_profile);
            Ok(())
        } else {
            Err(ExistingProfileError(name))
        }
    }

    pub fn remove(&mut self, name: &str) -> Option<TotpProfile> {
        self.profiles.remove(name)
    }

    pub fn write_to_disk(&self) -> Result<(), StoreSaveError> {
        let mut data = Vec::new();

        {
            let mut writer = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(&mut data);

            for profile in self.profiles.values() {
                writer.serialize(profile)?;
            }

            writer.flush().unwrap();
        }
        
        let cipher_key = Key::from_slice(&self.key);
        let cipher = Aes256GcmSiv::new(cipher_key);
        let nonce_data = thread_rng().gen::<[u8; 12]>();
        let nonce = Nonce::from_slice(&nonce_data);

        let encrypted = cipher.encrypt(nonce, data.as_ref())?;
        
        let mut file = fs::File::create(&self.path)?;
        file.write_all(&nonce_data)?;
        file.write_all(&encrypted)?;

        Ok(())
    }
}
