use sha1::{Digest, Sha1};

const HIBP_API: &str = "https://api.pwnedpasswords.com/range";

#[derive(Debug, Clone)]
pub struct BreachResult {
    pub breached: bool,
    pub count: u64,
}

#[derive(Debug, Clone)]
pub struct AuditResult {
    pub total_entries: usize,
    pub entries_with_passwords: usize,
    pub breached_entries: Vec<BreachAuditEntry>,
}

#[derive(Debug, Clone)]
pub struct BreachAuditEntry {
    pub entry_id: uuid::Uuid,
    pub title: String,
    pub breach_count: u64,
}

pub fn check_password(password: &str) -> Result<BreachResult, crate::error::CoalboxError> {
    let mut hasher = Sha1::new();
    hasher.update(password.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = format!("{:X}", hash);

    let prefix = &hash_hex[..5];
    let suffix = &hash_hex[5..];

    let url = format!("{}/{}", HIBP_API, prefix);

    let mut response = ureq::get(&url)
        .call()
        .map_err(|e| crate::error::CoalboxError::BreachCheck(e.to_string()))?;

    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| crate::error::CoalboxError::BreachCheck(e.to_string()))?;

    for line in body.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() == 2 {
            let hash_suffix = parts[0];
            let count: u64 = parts[1]
                .trim()
                .parse()
                .map_err(|_| crate::error::CoalboxError::BreachCheck("Invalid count".into()))?;

            if hash_suffix.eq_ignore_ascii_case(suffix) {
                return Ok(BreachResult {
                    breached: true,
                    count,
                });
            }
        }
    }

    Ok(BreachResult {
        breached: false,
        count: 0,
    })
}

pub fn check_passwords_batch(
    passwords: &[(uuid::Uuid, String, String)],
) -> Result<Vec<(uuid::Uuid, String, BreachResult)>, crate::error::CoalboxError> {
    let mut results = Vec::new();

    for (id, title, password) in passwords {
        let result = check_password(password)?;
        results.push((*id, title.clone(), result));
    }

    Ok(results)
}

pub fn audit_passwords(
    entries: &[crate::entry::Entry],
) -> Result<AuditResult, crate::error::CoalboxError> {
    let total_entries = entries.len();
    let mut entries_with_passwords = 0;
    let mut breached_entries = Vec::new();

    for entry in entries {
        if let Some(ref password) = entry.password {
            entries_with_passwords += 1;
            let result = check_password(password)?;
            if result.breached {
                breached_entries.push(BreachAuditEntry {
                    entry_id: entry.id,
                    title: entry.title.clone(),
                    breach_count: result.count,
                });
            }
        }
    }

    Ok(AuditResult {
        total_entries,
        entries_with_passwords,
        breached_entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_hash() {
        let mut hasher = Sha1::new();
        hasher.update(b"password");
        let hash = hasher.finalize();
        let hex = format!("{:X}", hash);
        assert_eq!(hex, "5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8");
    }

    #[test]
    fn test_password_not_breached_offline() {
        let unique = format!("test_password_{}", rand::random::<u64>());
        let result = check_password(&unique);
        assert!(result.is_ok());
        assert!(!result.unwrap().breached);
    }
}
