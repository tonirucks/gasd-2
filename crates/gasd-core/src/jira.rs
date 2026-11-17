//! Local saved searches and single-ticket task links. Remote Jira access belongs to adapters.
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::{Result, Service, Task, sqlite::require_changed, validation::nonblank};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraQuery {
    pub id: i64,
    pub name: String,
    pub jql: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JiraLink {
    pub key: String,
    pub url: String,
}

impl Service {
    pub fn get_jira_queries(&self) -> Result<Vec<JiraQuery>> {
        Ok(self
            .repository
            .connection
            .prepare("SELECT id, name, jql FROM jira_queries ORDER BY id")?
            .query_map([], |row| {
                Ok(JiraQuery {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    jql: row.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?)
    }

    pub fn save_jira_query(&mut self, id: Option<i64>, name: &str, jql: &str) -> Result<JiraQuery> {
        let name = nonblank(name, "query name")?;
        let jql = nonblank(jql, "JQL")?;
        let connection = &self.repository.connection;
        let id = if let Some(id) = id {
            require_changed(
                connection.execute(
                    "UPDATE jira_queries SET name = ?1, jql = ?2 WHERE id = ?3",
                    params![name, jql, id],
                )?,
                "Jira query",
                id,
            )?;
            id
        } else {
            connection.execute(
                "INSERT INTO jira_queries(name, jql) VALUES (?1, ?2)",
                params![name, jql],
            )?;
            connection.last_insert_rowid()
        };
        Ok(JiraQuery {
            id,
            name: name.into(),
            jql: jql.into(),
        })
    }

    pub fn delete_jira_query(&mut self, id: i64) -> Result<()> {
        require_changed(
            self.repository
                .connection
                .execute("DELETE FROM jira_queries WHERE id = ?1", [id])?,
            "Jira query",
            id,
        )
    }

    pub fn link_jira_task(&mut self, id: i64, link: Option<JiraLink>) -> Result<Task> {
        if let Some(link) = &link {
            nonblank(&link.key, "Jira key")?;
            nonblank(&link.url, "Jira URL")?;
        }
        require_changed(
            self.repository.connection.execute(
                "UPDATE tasks SET jira_key = ?1, jira_url = ?2 WHERE id = ?3",
                params![
                    link.as_ref().map(|l| &l.key),
                    link.as_ref().map(|l| &l.url),
                    id
                ],
            )?,
            "task",
            id,
        )?;
        self.get_task(id)
    }
}
