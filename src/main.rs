use base32::Alphabet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;

use crate::profile::TotpProfile;
use crate::store::ProfileStore;

mod otp;
mod profile;
mod store;

#[derive(Debug, StructOpt)]
#[structopt(name = "totp-rs", about = "A simple TOTP app written in Rust.")]
enum Command {
    Add {
        name: String,
        key: String,
        #[structopt(short = "t", short = "timestep", default_value = "30")]
        time_step: u64,
        #[structopt(short = "l", long = "length", default_value = "6")]
        length: u32,
    },
    Show {
        name: String,
    },
    Remove {
        name: String,
    },
}

fn main() {
    let path: PathBuf = "./profilestore.bin".into();
    let mut profiles = ProfileStore::load(path.clone()).unwrap_or(ProfileStore::new(path));

    let cmd = Command::from_args();
    match cmd {
        
        // Show OTP code for profile
        Command::Show { name } => {
            if let Some(profile) = profiles.get(name) {
                let time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                println!("{}", profile.get_otp(time));
            } else {
                println!("TOTP profile not found.");
            }
        }

        // Add profile to store
        Command::Add { name, key, time_step, length, } => {
            const ALPHABET: Alphabet = Alphabet::RFC4648 { padding: false };
            let secret = base32::decode(ALPHABET, &*key).unwrap();
            profiles
                .add(TotpProfile {
                    name,
                    secret,
                    time_step,
                    digits: length,
                })
                .unwrap();
        }

        // Remove profile from store
        Command::Remove { name } => {
            profiles.remove(name);
        }
    }

    profiles.write_to_disk().unwrap();
}

