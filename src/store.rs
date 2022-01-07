use crate::TotpProfile;
use std::collections::{hash_map::Entry, HashMap};
use std::path::PathBuf;
use std::io;

#[derive(Debug, thiserror::Error)]
#[error("failed to load store file")]
pub struct StoreLoadError(#[from] csv::Error);

#[derive(Debug, thiserror::Error)]
pub enum StoreSaveError {
    #[error("file could not be written")]
    CannotWriteFile(#[from] io::Error),
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
    pub path: PathBuf,
    pub profiles: HashMap<String, TotpProfile>,
}

impl ProfileStore {
    pub fn new(path: PathBuf) -> Result<Self, StoreSaveError> {
        let store = Self {
            path,
            profiles: HashMap::new(),
        };
        Ok(store)
    }

    pub fn load(path: PathBuf) -> Result<Self, StoreLoadError> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(&path)?;

        let mut profiles = HashMap::new();
        for profile in reader.deserialize() {
            let profile: TotpProfile = profile?;
            profiles.insert(profile.name.clone(), profile);
        }
        Ok(Self { path, profiles })
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
        let mut writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_path(&self.path)?;

        for profile in self.profiles.values() {
            writer.serialize(profile)?;
        }
        writer.flush()?;
        Ok(())
    }
}
