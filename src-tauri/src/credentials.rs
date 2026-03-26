/// OS-native credential storage for sensitive values (GitHub PAT, agent tokens).
/// Uses Windows Credential Manager, macOS Keychain, or Linux Secret Service.

const SERVICE: &str = "com.cmg.umbra";

pub fn store_secret(key: &str, value: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE, key).map_err(|e| e.to_string())?;
    entry.set_password(value).map_err(|e| e.to_string())
}

pub fn get_secret(key: &str) -> Option<String> {
    let entry = keyring::Entry::new(SERVICE, key).ok()?;
    entry.get_password().ok()
}

pub fn delete_secret(key: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVICE, key).map_err(|e| e.to_string())?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Migrate a plaintext value from config into the credential store.
/// Returns the value (from credential store if available, otherwise from plaintext).
pub fn migrate_secret(key: &str, plaintext: Option<&str>) -> Option<String> {
    // Check if already stored in credential manager
    if let Some(stored) = get_secret(key) {
        if !stored.is_empty() {
            return Some(stored);
        }
    }

    // Migrate plaintext value to credential store
    if let Some(value) = plaintext {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            let _ = store_secret(key, trimmed);
            return Some(trimmed.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_credential_store() {
        let key = "umbra-test-credential";
        let _ = delete_secret(key);

        // Store
        store_secret(key, "test-value-123").expect("store should succeed");

        // Retrieve
        let value = get_secret(key);
        assert_eq!(value, Some("test-value-123".to_string()));

        // Delete
        delete_secret(key).expect("delete should succeed");
        assert_eq!(get_secret(key), None);
    }

    #[test]
    fn migrate_moves_plaintext_to_store() {
        let key = "umbra-test-migrate";
        let _ = delete_secret(key);

        let result = migrate_secret(key, Some("my-pat-token"));
        assert_eq!(result, Some("my-pat-token".to_string()));

        // Now should come from store even without plaintext
        let result2 = migrate_secret(key, None);
        assert_eq!(result2, Some("my-pat-token".to_string()));

        let _ = delete_secret(key);
    }
}
