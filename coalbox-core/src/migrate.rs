use std::path::Path;

use crate::entry::Entry;
use crate::error::CoalboxError;

mod kdbx;
mod bitwarden;

pub use kdbx::export_kdbx;
pub use bitwarden::export_bitwarden_encrypted;

pub fn migrate_entries(
    entries: &[Entry],
    target: &str,
    output: &Path,
    export_password: &str,
) -> Result<(), CoalboxError> {
    match target {
        "kdbx" => export_kdbx(entries, output, export_password),
        "bitwarden" => export_bitwarden_encrypted(entries, output, export_password),
        _ => Err(CoalboxError::Import(format!(
            "Unsupported target format: {}. Use 'kdbx' or 'bitwarden'.",
            target
        ))),
    }
}
