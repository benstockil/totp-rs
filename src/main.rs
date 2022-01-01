use anyhow::Context;
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
    #[structopt(about = "Add new profile to store")]
    Add {
        name: String,
        key: String,
        #[structopt(short = "t", short = "timestep", default_value = "30")]
        time_step: u64,
        #[structopt(short = "l", long = "length", default_value = "6")]
        length: u32,
    },
    #[structopt(about = "Generate code for specified profile")]
    Show {
        name: String,
    },
    #[structopt(about = "Remove profile from store")]
    Remove {
        name: String,
    },
}

fn main() -> anyhow::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        println!("Uh oh! The program ran into a problem and crashed.\n\
                  Please submit an issue on GitHub (https://github.com/benstockil/totp-rs) \
                  or contact ben@stockil.co.uk to report this issue.\n\n\
                  Panic Info: \n{}", info);
    }));

    let path: PathBuf = "./profilestore.bin".into();
    let mut profiles = match path.is_file() {
        true => ProfileStore::load(path.clone())?,
        false => ProfileStore::new(path.clone())
    };

    let cmd = Command::from_args();
    match cmd {
        
        // Show OTP code for profile
        Command::Show { name } => {
            let profile = profiles.get(&name)
                .with_context(|| format!("Could not find profile with name {}", name))?;
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            println!("{:01$}", profile.get_otp(time), profile.digits as usize);
        }

        // Add profile to store
        Command::Add { name, key, time_step, length, } => {
            const ALPHABET: Alphabet = Alphabet::RFC4648 { padding: false };
            let secret = base32::decode(ALPHABET, &*key)
                .with_context(|| format!("{} is not a valid base32 string", key))?;
            profiles.add(TotpProfile {
                name: name.clone(),
                secret,
                time_step,
                digits: length,
            })?;
        }

        // Remove profile from store
        Command::Remove { name } => {
            profiles.remove(&name)
                .with_context(|| format!("Could not file profile with name {}", name))?;
        }
    }

    profiles.write_to_disk()?;

    Ok(())
}
