//! Typed application settings backed by local storage. Secrets must not be stored here.
use rusqlite::{OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};

use crate::{Result, Service, validation::nonblank};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JiraSettings {
    pub site: String,
    pub email: String,
}

impl Service {
    pub fn get_jira_settings(&self) -> Result<Option<JiraSettings>> {
        Ok(self
            .repository
            .connection
            .query_row(
                "SELECT site.value, email.value FROM app_settings site
             JOIN app_settings email ON email.key = 'jira.email'
             WHERE site.key = 'jira.site'",
                [],
                |row| {
                    Ok(JiraSettings {
                        site: row.get(0)?,
                        email: row.get(1)?,
                    })
                },
            )
            .optional()?)
    }

    pub fn save_jira_settings(&mut self, settings: &JiraSettings) -> Result<()> {
        let site = nonblank(&settings.site, "Jira site")?;
        let email = nonblank(&settings.email, "Jira email")?;
        let tx = self
            .repository
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        for (key, value) in [("jira.site", site), ("jira.email", email)] {
            tx.execute(
                "INSERT INTO app_settings(key, value) VALUES (?1, ?2)
                        ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn clear_jira_settings(&mut self) -> Result<()> {
        self.repository.connection.execute(
            "DELETE FROM app_settings WHERE key IN ('jira.site', 'jira.email')",
            [],
        )?;
        Ok(())
    }
}
