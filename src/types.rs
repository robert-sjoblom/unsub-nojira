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
    pub name: String,
    #[serde(rename = "displayName")]
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
}

#[derive(Debug, Deserialize)]
pub struct Participant {
    pub user: User,
    #[allow(dead_code)]
    pub role: String,
    #[allow(dead_code)]
    pub approved: bool,
}

#[derive(Debug, Deserialize)]
pub struct PaginatedResponse<T> {
    pub values: Vec<T>,
    #[serde(rename = "isLastPage")]
    pub is_last_page: bool,
    #[serde(rename = "nextPageStart")]
    pub next_page_start: Option<u64>,
}
