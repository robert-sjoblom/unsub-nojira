use anyhow::{Context, Result};
use reqwest::Client;

use crate::types::{PaginatedResponse, Participant, PullRequest, Repository};

pub struct BitbucketClient {
    base_url: String,
    token: String,
    client: Client,
}

impl BitbucketClient {
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            base_url,
            token,
            client: Client::new(),
        }
    }

    pub async fn list_repos(&self, project: &str) -> Result<Vec<Repository>> {
        let mut all_repos = Vec::new();
        let mut start = 0;
        let limit = 100;

        loop {
            let url = format!(
                "{}/rest/api/1.0/projects/{}/repos?start={}&limit={}",
                self.base_url, project, start, limit
            );

            let response: PaginatedResponse<Repository> = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .send()
                .await
                .context("Failed to fetch repositories")?
                .error_for_status()
                .context("API returned error status")?
                .json()
                .await
                .context("Failed to parse repositories response")?;

            all_repos.extend(response.values);

            if response.is_last_page {
                break;
            }

            start = response.next_page_start.unwrap_or(start + limit);
        }

        Ok(all_repos)
    }

    pub async fn list_open_prs(&self, project: &str, repo: &str) -> Result<Vec<PullRequest>> {
        let mut all_prs = Vec::new();
        let mut start = 0;
        let limit = 100;

        loop {
            let url = format!(
                "{}/rest/api/1.0/projects/{}/repos/{}/pull-requests?state=OPEN&start={}&limit={}",
                self.base_url, project, repo, start, limit
            );

            let response: PaginatedResponse<PullRequest> = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .send()
                .await
                .context("Failed to fetch pull requests")?
                .error_for_status()
                .context("API returned error status")?
                .json()
                .await
                .context("Failed to parse pull requests response")?;

            all_prs.extend(response.values);

            if response.is_last_page {
                break;
            }

            start = response.next_page_start.unwrap_or(start + limit);
        }

        Ok(all_prs)
    }

    pub async fn get_pr_participants(
        &self,
        project: &str,
        repo: &str,
        pr_id: u64,
    ) -> Result<Vec<Participant>> {
        let url = format!(
            "{}/rest/api/1.0/projects/{}/repos/{}/pull-requests/{}/participants",
            self.base_url, project, repo, pr_id
        );

        let response: PaginatedResponse<Participant> = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .context(format!("Failed to fetch participants for PR {}", pr_id))?
            .error_for_status()
            .context("API returned error status")?
            .json()
            .await
            .context("Failed to parse participants response")?;

        Ok(response.values)
    }

    pub async fn remove_reviewer(
        &self,
        project: &str,
        repo: &str,
        pr_id: u64,
        user_slug: &str,
    ) -> Result<()> {
        let url = format!(
            "{}/rest/api/1.0/projects/{}/repos/{}/pull-requests/{}/participants/{}",
            self.base_url, project, repo, pr_id, user_slug
        );

        self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .context(format!("Failed to remove reviewer from PR {}", pr_id))?
            .error_for_status()
            .context("API returned error status when removing reviewer")?;

        Ok(())
    }
}
