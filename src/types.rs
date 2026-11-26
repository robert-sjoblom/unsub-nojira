use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,
    pub token: String,
    pub excluded_repos: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub slug: String,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "displayName", default)]
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub slug: String,
    #[allow(dead_code)]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub id: u64,
    pub title: String,
    #[serde(default)]
    pub reviewers: Vec<Participant>,
    #[serde(default)]
    pub participants: Vec<Participant>,
}

#[derive(Debug, Deserialize)]
pub struct Participant {
    pub user: User,
    #[allow(dead_code)]
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub approved: bool,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaginatedResponse<T> {
    pub values: Vec<T>,
    #[serde(rename = "isLastPage")]
    pub is_last_page: bool,
    #[serde(rename = "nextPageStart")]
    pub next_page_start: Option<u64>,
}
