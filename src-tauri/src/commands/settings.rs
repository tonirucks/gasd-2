use super::{AppState, CommandError, Result, with_native_service};
use gasd_core::{Service, settings::JiraSettings};
use serde::Serialize;
use tauri::State;

fn error(message: &str) -> CommandError {
    CommandError {
        code: "settings",
        message: message.into(),
    }
}

fn credential_error() -> CommandError {
    error(
        "Could not access the OS credential store. Unlock your keychain. On Linux/WSL, start a Secret Service provider and create/unlock a default Passwords keyring in Seahorse (Passwords and Keys); a running daemon alone is not enough. No token is stored in plain text.",
    )
}

fn keyring_error(source: keyring::Error) -> CommandError {
    // Classify failures without displaying backend details that could contain
    // credential data (for example, BadEncoding contains the stored bytes).
    match source {
        keyring::Error::NoStorageAccess(_) => error(
            "The OS credential store is locked or access was denied. Unlock the keyring in Passwords and Keys (Seahorse) on Linux/WSL, or your OS credential manager, and retry.",
        ),
        keyring::Error::BadEncoding(_) => error(
            "The saved Jira credential could not be decoded. Enter a replacement API token in Settings.",
        ),
        _ => credential_error(),
    }
}

// Abstracted so tests never read or change the user's real credential store.
trait TokenStore {
    fn get(&self, settings: &JiraSettings) -> Result<Option<String>>;
    fn set(&self, settings: &JiraSettings, token: &str) -> Result<()>;
    fn delete(&self, settings: &JiraSettings) -> Result<()>;
}

struct OsTokenStore;

fn entry(settings: &JiraSettings) -> Result<keyring::Entry> {
    // Scope credentials to the connection identity: changing site or email must
    // never accidentally send the previous account's token to the new site.
    keyring::Entry::new(
        "gasd.jira",
        &format!("{}|{}", settings.site, settings.email),
    )
    .map_err(keyring_error)
}

impl TokenStore for OsTokenStore {
    fn get(&self, settings: &JiraSettings) -> Result<Option<String>> {
        match entry(settings)?.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(source) => Err(keyring_error(source)),
        }
    }

    fn set(&self, settings: &JiraSettings, token: &str) -> Result<()> {
        entry(settings)?.set_password(token).map_err(keyring_error)
    }

    fn delete(&self, settings: &JiraSettings) -> Result<()> {
        match entry(settings)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(source) => Err(keyring_error(source)),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraConnectionSettings {
    site: String,
    email: String,
    has_token: bool,
}

#[derive(Serialize)]
pub struct AppSettings {
    jira: JiraConnectionSettings,
}

fn get_settings(service: &Service, tokens: &impl TokenStore) -> Result<AppSettings> {
    let jira = match service.get_jira_settings()? {
        Some(settings) => JiraConnectionSettings {
            has_token: tokens.get(&settings)?.is_some(),
            site: settings.site,
            email: settings.email,
        },
        None => JiraConnectionSettings {
            site: String::new(),
            email: String::new(),
            has_token: false,
        },
    };
    Ok(AppSettings { jira })
}

fn save_settings(
    service: &mut Service,
    tokens: &impl TokenStore,
    site: String,
    email: String,
    token: Option<String>,
) -> Result<AppSettings> {
    let settings = JiraSettings {
        site: super::jira::site_origin(&site)?,
        email: email.trim().into(),
    };
    if settings.email.is_empty() || !settings.email.contains('@') {
        return Err(error("Enter your Atlassian account email."));
    }
    if let Some(token) = token {
        if token.trim().is_empty() {
            return Err(error("The Jira API token must not be blank."));
        }
        // Persist the secret first: keychain failure must leave the active
        // database connection unchanged. No plaintext fallback is allowed.
        tokens.set(&settings, token.trim())?;
    } else if tokens.get(&settings)?.is_none() {
        return Err(error(
            "Enter an API token for this site and email. Existing tokens are only reused for the same connection.",
        ));
    }
    service.save_jira_settings(&settings)?;
    Ok(AppSettings {
        jira: JiraConnectionSettings {
            site: settings.site,
            email: settings.email,
            has_token: true,
        },
    })
}

fn clear_settings(service: &mut Service, tokens: &impl TokenStore) -> Result<()> {
    if let Some(settings) = service.get_jira_settings()? {
        tokens.delete(&settings)?;
    }
    service.clear_jira_settings()?;
    Ok(())
}

pub(super) async fn jira_credentials(state: State<'_, AppState>) -> Result<(JiraSettings, String)> {
    with_native_service(state, |service| {
        let settings = service.get_jira_settings()?.ok_or_else(|| {
            error("Configure Jira in Settings before running a query or linking a ticket.")
        })?;
        let token = OsTokenStore
            .get(&settings)?
            .ok_or_else(|| error("Jira API token is missing. Add one in Settings."))?;
        Ok((settings, token))
    })
    .await
}

#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>) -> Result<AppSettings> {
    with_native_service(state, |service| get_settings(service, &OsTokenStore)).await
}

#[tauri::command]
pub async fn settings_save_jira(
    state: State<'_, AppState>,
    site: String,
    email: String,
    token: Option<String>,
) -> Result<AppSettings> {
    with_native_service(state, move |service| {
        save_settings(service, &OsTokenStore, site, email, token)
    })
    .await
}

#[tauri::command]
pub async fn settings_clear_jira(state: State<'_, AppState>) -> Result<()> {
    with_native_service(state, |service| clear_settings(service, &OsTokenStore)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::{Cell, RefCell},
        collections::HashMap,
    };

    #[derive(Default)]
    struct MemoryTokens {
        values: RefCell<HashMap<(String, String), String>>,
        unavailable: Cell<bool>,
    }

    impl TokenStore for MemoryTokens {
        fn get(&self, settings: &JiraSettings) -> Result<Option<String>> {
            if self.unavailable.get() {
                return Err(credential_error());
            }
            Ok(self
                .values
                .borrow()
                .get(&(settings.site.clone(), settings.email.clone()))
                .cloned())
        }
        fn set(&self, settings: &JiraSettings, token: &str) -> Result<()> {
            if self.unavailable.get() {
                return Err(credential_error());
            }
            self.values.borrow_mut().insert(
                (settings.site.clone(), settings.email.clone()),
                token.into(),
            );
            Ok(())
        }
        fn delete(&self, settings: &JiraSettings) -> Result<()> {
            if self.unavailable.get() {
                return Err(credential_error());
            }
            self.values
                .borrow_mut()
                .remove(&(settings.site.clone(), settings.email.clone()));
            Ok(())
        }
    }

    fn service() -> Service {
        Service::new(gasd_core::SqliteRepository::in_memory().unwrap())
    }

    #[test]
    fn credential_errors_explain_recovery_without_exposing_stored_bytes() {
        let encoded = keyring_error(keyring::Error::BadEncoding(b"secret-token".to_vec()));
        assert!(encoded.message.contains("replacement API token"));
        assert!(!encoded.message.contains("secret-token"));
        let locked = keyring_error(keyring::Error::NoStorageAccess(Box::new(
            std::io::Error::other("sensitive backend detail"),
        )));
        assert!(locked.message.contains("locked"));
        assert!(!locked.message.contains("sensitive backend detail"));
        assert!(
            credential_error()
                .message
                .contains("default Passwords keyring")
        );
    }

    fn save(
        service: &mut Service,
        tokens: &MemoryTokens,
        site: &str,
        email: &str,
        token: Option<&str>,
    ) -> Result<AppSettings> {
        save_settings(
            service,
            tokens,
            site.into(),
            email.into(),
            token.map(Into::into),
        )
    }

    #[test]
    fn stores_only_metadata_and_token_presence_in_api_responses() {
        let mut service = service();
        let tokens = MemoryTokens::default();
        assert!(!get_settings(&service, &tokens).unwrap().jira.has_token);
        let response = save(
            &mut service,
            &tokens,
            " https://team.atlassian.net/ ",
            " me@example.com ",
            Some("secret-token"),
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(response).unwrap(),
            serde_json::json!({
                "jira": { "site": "https://team.atlassian.net", "email": "me@example.com", "hasToken": true }
            })
        );
        let settings = service.get_jira_settings().unwrap().unwrap();
        assert!(
            !serde_json::to_string(&settings)
                .unwrap()
                .contains("secret-token")
        );
        assert_eq!(
            tokens.get(&settings).unwrap().as_deref(),
            Some("secret-token")
        );
        save(&mut service, &tokens, &settings.site, &settings.email, None).unwrap();
        assert_eq!(
            tokens.get(&settings).unwrap().as_deref(),
            Some("secret-token")
        );
        save(
            &mut service,
            &tokens,
            &settings.site,
            &settings.email,
            Some("replacement"),
        )
        .unwrap();
        assert_eq!(
            tokens.get(&settings).unwrap().as_deref(),
            Some("replacement")
        );
    }

    #[test]
    fn never_reuses_a_different_sites_or_accounts_token() {
        let mut service = service();
        let tokens = MemoryTokens::default();
        save(
            &mut service,
            &tokens,
            "https://team.atlassian.net",
            "me@example.com",
            Some("secret"),
        )
        .unwrap();
        let original = service.get_jira_settings().unwrap();
        for (site, email) in [
            ("https://other.atlassian.net", "me@example.com"),
            ("https://team.atlassian.net", "other@example.com"),
        ] {
            assert!(save(&mut service, &tokens, site, email, None).is_err());
            assert_eq!(service.get_jira_settings().unwrap(), original);
        }
        assert!(
            save(
                &mut service,
                &tokens,
                "https://evil.com",
                "me@example.com",
                Some("secret")
            )
            .is_err()
        );
        assert!(
            save(
                &mut service,
                &tokens,
                "https://team.atlassian.net",
                "",
                Some("secret")
            )
            .is_err()
        );
        assert!(
            save(
                &mut service,
                &tokens,
                "https://team.atlassian.net",
                "me@example.com",
                Some(" ")
            )
            .is_err()
        );
    }

    #[test]
    fn keychain_failure_never_falls_back_to_database_storage() {
        let mut service = service();
        let tokens = MemoryTokens::default();
        tokens.unavailable.set(true);
        assert!(
            save(
                &mut service,
                &tokens,
                "https://team.atlassian.net",
                "me@example.com",
                Some("secret")
            )
            .is_err()
        );
        assert!(service.get_jira_settings().unwrap().is_none());
        tokens.unavailable.set(false);
        save(
            &mut service,
            &tokens,
            "https://team.atlassian.net",
            "me@example.com",
            Some("secret"),
        )
        .unwrap();
        let original = service.get_jira_settings().unwrap();
        tokens.unavailable.set(true);
        assert!(
            save(
                &mut service,
                &tokens,
                "https://other.atlassian.net",
                "me@example.com",
                Some("new-secret")
            )
            .is_err()
        );
        assert!(clear_settings(&mut service, &tokens).is_err());
        assert_eq!(service.get_jira_settings().unwrap(), original);
    }

    #[test]
    fn disconnect_removes_active_token_but_preserves_queries_and_tasks() {
        let mut service = service();
        let tokens = MemoryTokens::default();
        let section = service.create_section("Work").unwrap();
        let task = service.create_task(section.id, "Review").unwrap();
        let query = service
            .save_jira_query(None, "Mine", "assignee = currentUser()")
            .unwrap();
        save(
            &mut service,
            &tokens,
            "https://team.atlassian.net",
            "me@example.com",
            Some("secret"),
        )
        .unwrap();
        clear_settings(&mut service, &tokens).unwrap();
        clear_settings(&mut service, &tokens).unwrap();
        assert!(tokens.values.borrow().is_empty());
        assert!(service.get_jira_settings().unwrap().is_none());
        assert_eq!(service.get_task(task.id).unwrap(), task);
        assert_eq!(service.get_jira_queries().unwrap(), [query]);
    }
}
