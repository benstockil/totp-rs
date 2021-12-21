use hmac::{Hmac, Mac};
use sha1::Sha1;

pub fn totp(secret: &[u8], time: u64, time_step: u64, digits: u32) -> u32 {
    hotp(secret, time / time_step, digits)
}

pub fn hotp(secret: &[u8], counter: u64, digits: u32) -> u32 {
    let mut hmac = Hmac::<Sha1>::new_from_slice(secret).unwrap();
    hmac.update(&counter.to_be_bytes());
    let result = hmac.finalize().into_bytes();
    let truncated = truncate(&result);
    truncated % 10u32.pow(digits)
}

fn truncate(input: &[u8]) -> u32 {
    let offset = (input[19] & 0xf) as usize;
    let mut bytes = [0; 4];
    bytes.copy_from_slice(&input[offset..offset + 4]);
    bytes[0] &= 0x7f;
    u32::from_be_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotp_generation() {
        let secret = "12345678901234567890".as_bytes();
        #[rustfmt::skip]
        let results = [
            755224, 287082, 359152, 969429, 338314, 
            254676, 287922, 162583, 399871, 520489,
        ];
        for i in 0..10 {
            assert_eq!(hotp(secret, i as u64, 6), results[i]);
        }
    }

    #[test]
    fn totp_generation() {
        let secret1 = "12345678901234567890".as_bytes();
        assert_eq!(totp(secret1, 59, 30, 8), 94287082);
        assert_eq!(totp(secret1, 1111111109, 30, 8), 07081804);
    }
}
