use crate::otp::totp;
use base32::{self, Alphabet}; 
use serde::{
    de::{self, Visitor},
    Deserialize, Serialize, Serializer,
};

static B32_ALPHABET: Alphabet = Alphabet::RFC4648 { padding: false };

fn serialize_secret<S>(secret: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where S: Serializer {
    serializer.serialize_str(
        &base32::encode(B32_ALPHABET, secret)
    )
}

fn deserialize_secret<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where D: serde::Deserializer<'de> {
    struct SecretVisitor;

    impl<'de> Visitor<'de> for SecretVisitor {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a valid base32 string")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E> 
        where E: de::Error {
           match base32::decode(B32_ALPHABET, s) {
               Some(bytes) => Ok(bytes),
               None => Err(de::Error::custom("invalid base32-encoded string"))
            }
        }
    }

    deserializer.deserialize_str(SecretVisitor).into()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpProfile {
    pub name: String,
    #[serde(serialize_with = "serialize_secret")]
    #[serde(deserialize_with = "deserialize_secret")]
    pub secret: Vec<u8>,
    pub time_step: u64,
    pub digits: u32,
}

impl TotpProfile {
    pub fn get_otp(&self, time: u64) -> u32 {
        totp(&self.secret, time, self.time_step, self.digits)
    }
}
