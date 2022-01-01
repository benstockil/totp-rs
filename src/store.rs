use crate::TotpProfile;
use std::collections::{HashMap, hash_map::Entry};
use std::path::PathBuf;
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum StoreLoadError {
    #[error("file could not be read")]
    CannotReadFile(#[from] io::Error),
    #[error("failed to deserialize data")]
    CannotDeserialize(#[from] csv::Error),
}

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
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            profiles: HashMap::new(),
        }
    }

    pub fn load(path: PathBuf) -> Result<Self, StoreLoadError> {
        let mut reader = csv::Reader::from_path(&path)?;
        let mut profiles = HashMap::new();
        for profile in reader.deserialize() {
            let profile: TotpProfile = profile?;
            profiles.insert(profile.name.clone(), profile);
        }
        Ok(Self { path, profiles })
    }

    pub fn get(&self, name: String) -> Option<&TotpProfile> {
        self.profiles.get(&name)
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

    pub fn remove(&mut self, name: String) -> Option<TotpProfile> {
        self.profiles.remove(&name)
    }

    pub fn write_to_disk(&self) -> Result<(), StoreSaveError> {
        let mut writer = csv::Writer::from_path(&self.path)?;
        for profile in &self.profiles {
            writer.serialize(profile)?;
        }
        writer.flush()?;
        Ok(())
    }
}
