use super::{AppState, CommandError, Result, with_service};
use gasd_core::{
    Task,
    jira::{JiraLink, JiraQuery},
};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use tauri::State;

fn error(message: impl Into<String>) -> CommandError {
    CommandError {
        code: "jira",
        message: message.into(),
    }
}

struct Connection {
    client: Client,
    site: String,
    email: String,
    token: String,
}

pub(super) fn site_origin(site: &str) -> Result<String> {
    let url =
        Url::parse(site.trim()).map_err(|_| error("Jira site must be an HTTPS Jira Cloud URL."))?;
    if url.scheme() != "https"
        || !url
            .host_str()
            .is_some_and(|host| host.ends_with(".atlassian.net"))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(error(
            "Jira site must be an origin such as https://your-team.atlassian.net (no path).",
        ));
    }
    Ok(url.origin().ascii_serialization())
}

impl Connection {
    async fn from_settings(state: State<'_, AppState>) -> Result<Self> {
        let (settings, token) = super::settings::jira_credentials(state).await?;
        Ok(Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|_| error("Could not initialize the Jira connection."))?,
            site: site_origin(&settings.site)?,
            email: settings.email,
            token,
        })
    }

    async fn send<T: serde::de::DeserializeOwned>(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<T> {
        let response = request
            .basic_auth(&self.email, Some(&self.token))
            .send()
            .await
            .map_err(|_| {
                error("Could not reach Jira. Check your network and Jira site, then retry.")
            })?;
        if !response.status().is_success() {
            let message = match response.status().as_u16() {
                400 => "Jira rejected the request. Check the JQL or ticket key.",
                401 | 403 => {
                    "Jira access denied. Check your email, API token, and Jira permissions."
                }
                404 => "Jira ticket or API endpoint was not found.",
                429 => "Jira rate limit reached. Wait a moment before retrying.",
                _ => "Jira returned an unexpected response. Please retry later.",
            };
            return Err(error(message));
        }
        response
            .json()
            .await
            .map_err(|_| error("Jira returned an unreadable response."))
    }
}

#[derive(Deserialize)]
struct IssueFields {
    summary: String,
    status: Status,
}
#[derive(Deserialize)]
struct Status {
    name: String,
}
#[derive(Deserialize)]
struct Issue {
    key: String,
    fields: IssueFields,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchResponse {
    issues: Vec<Issue>,
    next_page_token: Option<String>,
}
#[derive(Serialize)]
pub struct Ticket {
    key: String,
    summary: String,
    status: String,
    url: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPage {
    tickets: Vec<Ticket>,
    next_page_token: Option<String>,
}

#[tauri::command]
pub async fn jira_queries(state: State<'_, AppState>) -> Result<Vec<JiraQuery>> {
    with_service(state, |service| service.get_jira_queries()).await
}

#[tauri::command]
pub async fn jira_test_connection(state: State<'_, AppState>) -> Result<()> {
    let connection = Connection::from_settings(state).await?;
    let _: serde_json::Value = connection
        .send(
            connection
                .client
                .get(format!("{}/rest/api/3/myself", connection.site)),
        )
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn jira_query_save(
    state: State<'_, AppState>,
    id: Option<i64>,
    name: String,
    jql: String,
) -> Result<JiraQuery> {
    with_service(state, move |service| {
        service.save_jira_query(id, &name, &jql)
    })
    .await
}

#[tauri::command]
pub async fn jira_query_delete(state: State<'_, AppState>, id: i64) -> Result<()> {
    with_service(state, move |service| service.delete_jira_query(id)).await
}

#[tauri::command]
pub async fn jira_search(
    state: State<'_, AppState>,
    query_id: i64,
    next_page_token: Option<String>,
) -> Result<SearchPage> {
    let query = with_service(state.clone(), move |service| {
        service
            .get_jira_queries()?
            .into_iter()
            .find(|q| q.id == query_id)
            .ok_or(gasd_core::Error::NotFound {
                entity: "Jira query",
                id: query_id,
            })
    })
    .await?;
    let connection = Connection::from_settings(state).await?;
    let mut body =
        serde_json::json!({ "jql": query.jql, "fields": ["summary", "status"], "maxResults": 50 });
    if let Some(token) = next_page_token {
        body["nextPageToken"] = token.into();
    }
    let page: SearchResponse = connection
        .send(
            connection
                .client
                .post(format!("{}/rest/api/3/search/jql", connection.site))
                .json(&body),
        )
        .await?;
    Ok(SearchPage {
        tickets: page
            .issues
            .into_iter()
            .map(|issue| Ticket {
                url: format!("{}/browse/{}", connection.site, issue.key),
                key: issue.key,
                summary: issue.fields.summary,
                status: issue.fields.status.name,
            })
            .collect(),
        next_page_token: page.next_page_token,
    })
}

fn ticket_key(key: &str) -> Result<String> {
    let key = key.trim().to_ascii_uppercase();
    let valid = key.split_once('-').is_some_and(|(project, number)| {
        !project.is_empty()
            && project.starts_with(|c: char| c.is_ascii_alphabetic())
            && project
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
            && !number.is_empty()
            && number.chars().all(|c| c.is_ascii_digit())
    });
    if valid {
        Ok(key)
    } else {
        Err(error("Enter a Jira ticket key such as TEAM-123."))
    }
}

#[tauri::command]
pub async fn jira_task_link(
    state: State<'_, AppState>,
    task_id: i64,
    key: Option<String>,
) -> Result<Task> {
    let link = if let Some(key) = key {
        let key = ticket_key(&key)?;
        let connection = Connection::from_settings(state.clone()).await?;
        let issue: Issue = connection
            .send(
                connection
                    .client
                    .get(format!("{}/rest/api/3/issue/{key}", connection.site))
                    .query(&[("fields", "summary,status")]),
            )
            .await?;
        Some(JiraLink {
            url: format!("{}/browse/{}", connection.site, issue.key),
            key: issue.key,
        })
    } else {
        None
    };
    with_service(state, move |service| service.link_jira_task(task_id, link)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restricts_credentials_to_cloud_origin() {
        assert_eq!(
            site_origin(" https://team.atlassian.net/ ").unwrap(),
            "https://team.atlassian.net"
        );
        for site in [
            "http://team.atlassian.net",
            "https://evil.com",
            "https://team.atlassian.net.evil.com",
            "https://u:p@team.atlassian.net",
            "https://team.atlassian.net/path",
            "https://team.atlassian.net?x=1",
        ] {
            assert!(site_origin(site).is_err(), "{site}");
        }
    }

    #[test]
    fn validates_keys_before_using_in_path() {
        assert_eq!(ticket_key(" team-123 ").unwrap(), "TEAM-123");
        for key in ["", "ABC", "ABC-", "-123", "ABC-1/path", "ABC-1?x", "1ABC-1"] {
            assert!(ticket_key(key).is_err());
        }
    }

    #[test]
    fn reads_enhanced_search_pages_with_and_without_continuation() {
        let page: SearchResponse = serde_json::from_value(serde_json::json!({
            "issues": [{ "key": "TEAM-1", "fields": {
                "summary": "Review", "status": { "name": "In Progress" }
            }}],
            "nextPageToken": "page-2", "isLast": false
        }))
        .unwrap();
        assert_eq!(page.issues[0].key, "TEAM-1");
        assert_eq!(page.issues[0].fields.summary, "Review");
        assert_eq!(page.issues[0].fields.status.name, "In Progress");
        assert_eq!(page.next_page_token.as_deref(), Some("page-2"));
        let last: SearchResponse = serde_json::from_value(serde_json::json!({
            "issues": [], "isLast": true
        }))
        .unwrap();
        assert!(last.next_page_token.is_none());
    }
}
