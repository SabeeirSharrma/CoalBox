use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::Sha256;

type HmacSha1 = Hmac<Sha1>;
type HmacSha256 = Hmac<Sha256>;

const DEFAULT_PERIOD: u64 = 30;
const DEFAULT_DIGITS: u32 = 6;

#[derive(Debug, Clone)]
pub struct TotpCode {
    pub code: String,
    pub remaining: u32,
}

#[derive(Debug, Clone)]
pub struct TotpConfig {
    pub secret: Vec<u8>,
    pub digits: u32,
    pub period: u64,
    pub algorithm: TotpAlgorithm,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TotpAlgorithm {
    Sha1,
    Sha256,
}

impl Default for TotpConfig {
    fn default() -> Self {
        Self {
            secret: Vec::new(),
            digits: DEFAULT_DIGITS,
            period: DEFAULT_PERIOD,
            algorithm: TotpAlgorithm::Sha1,
        }
    }
}

impl TotpConfig {
    pub fn from_secret(secret: &str) -> Result<Self, crate::error::CoalboxError> {
        let decoded = base32::decode(base32::Alphabet::RFC4648 { padding: true }, secret)
            .ok_or_else(|| crate::error::CoalboxError::Totp("Invalid base32 secret".into()))?;

        Ok(Self {
            secret: decoded,
            ..Default::default()
        })
    }

    pub fn generate_code(&self, time: u64) -> TotpCode {
        let time_step = time / self.period;
        let remaining = self.period - (time % self.period);

        let counter = time_step.to_be_bytes();
        let code = match self.algorithm {
            TotpAlgorithm::Sha1 => {
                let mut mac = HmacSha1::new_from_slice(&self.secret)
                    .expect("HMAC can take key of any size");
                mac.update(&counter);
                let result = mac.finalize().into_bytes();
                self.truncate(&result)
            }
            TotpAlgorithm::Sha256 => {
                let mut mac = HmacSha256::new_from_slice(&self.secret)
                    .expect("HMAC can take key of any size");
                mac.update(&counter);
                let result = mac.finalize().into_bytes();
                self.truncate(&result)
            }
        };

        TotpCode {
            code,
            remaining: remaining as u32,
        }
    }

    pub fn generate_current(&self) -> TotpCode {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.generate_code(now)
    }

    pub fn verify(&self, code: &str, time: u64) -> bool {
        let time_step = time / self.period;

        for offset in -1i64..=1 {
            let counter = (time_step as i64 + offset).to_be_bytes();
            let expected = match self.algorithm {
                TotpAlgorithm::Sha1 => {
                    let mut mac = HmacSha1::new_from_slice(&self.secret)
                        .expect("HMAC can take key of any size");
                    mac.update(&counter);
                    let result = mac.finalize().into_bytes();
                    self.truncate(&result)
                }
                TotpAlgorithm::Sha256 => {
                    let mut mac = HmacSha256::new_from_slice(&self.secret)
                        .expect("HMAC can take key of any size");
                    mac.update(&counter);
                    let result = mac.finalize().into_bytes();
                    self.truncate(&result)
                }
            };

            if expected == code {
                return true;
            }
        }
        false
    }

    fn truncate(&self, hash: &[u8]) -> String {
        let offset = (hash[hash.len() - 1] & 0x0f) as usize;
        let code_bytes = [
            hash[offset] & 0x7f,
            hash[offset + 1],
            hash[offset + 2],
            hash[offset + 3],
        ];
        let code_num = u32::from_be_bytes(code_bytes);
        let modulus = 10u32.pow(self.digits);
        let code = code_num % modulus;
        format!("{:0>width$}", code, width = self.digits as usize)
    }

    pub fn otpauth_uri(&self, issuer: &str, account: &str) -> String {
        let secret = base32::encode(
            base32::Alphabet::RFC4648 { padding: true },
            &self.secret,
        );
        format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&digits={}&period={}",
            issuer,
            account,
            secret,
            issuer,
            self.digits,
            self.period
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_code_length() {
        let config = TotpConfig {
            secret: vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
            digits: 6,
            period: 30,
            algorithm: TotpAlgorithm::Sha1,
        };
        let code = config.generate_current();
        assert_eq!(code.code.len(), 6);
    }

    #[test]
    fn test_totp_verify() {
        let secret = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let config = TotpConfig {
            secret: secret.clone(),
            digits: 6,
            period: 30,
            algorithm: TotpAlgorithm::Sha1,
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let code = config.generate_code(now);
        assert!(config.verify(&code.code, now));
    }

    #[test]
    fn test_base32_decode() {
        let config = TotpConfig::from_secret("JBSWY3DPEHPK3PXP");
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.secret.len(), 10);
    }
}
