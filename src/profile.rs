use crate::otp::totp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TotpProfile {
    pub name: String,
    pub secret: Vec<u8>,
    pub time_step: u64,
    pub digits: u32,
}

impl TotpProfile {
    pub fn get_otp(&self, time: u64) -> u32 {
        totp(&self.secret, time, self.time_step, self.digits)
    }
}
