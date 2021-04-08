use std::collections::HashMap;

use anyhow::{anyhow, Result};
use log::warn;
use reqwest::{
    blocking::{Client, ClientBuilder},
    header,
};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct GitHubBranch {
    #[serde(rename = "ref")]
    name: String,
}

#[derive(Deserialize, Clone)]
struct GitHubPullRequest {
    number: u64,
    title: String,
    head: GitHubBranch,
}

fn fetch_description(client: &Client, repo: &str, page: usize) -> Result<Vec<GitHubPullRequest>> {
    let url = format!(
        "https://api.github.com/repos/{}/pulls?base=stable&per_page=100&page={}",
        repo, page
    );
    let resp = client.get(&url).send()?;
    let payload: Vec<GitHubPullRequest> = resp.json()?;

    Ok(payload)
}

pub(crate) fn create_client() -> Result<Client> {
    let api_token = std::env::var("GITHUB_TOKEN");
    let mut client =
        ClientBuilder::new().user_agent(concat!("repo-manifest/", env!("CARGO_PKG_VERSION")));
    if let Ok(api_token) = api_token {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT, "application/vnd.github.v3+json".parse()?);
        headers.insert(
            header::AUTHORIZATION,
            format!("token {}", api_token).parse()?,
        );
        client = client.default_headers(headers);
    } else {
        warn!("Set GITHUB_TOKEN to increase the rate limits!");
    }

    Ok(client.build()?)
}

pub fn fetch_descriptions(client: &Client, repo: &str) -> Result<HashMap<String, String>> {
    let repo_name = repo.splitn(2, '/').nth(0);
    repo_name.ok_or_else(|| anyhow!("Invalid repo name: {}", repo))?;
    let mut results: HashMap<String, String> = HashMap::new();
    let mut page = 1usize;
    loop {
        let this_page = fetch_description(client, repo, page)?;
        let no_next_page = this_page.len() < 100;
        for entry in this_page {
            results.insert(entry.head.name, entry.title);
        }
        if no_next_page {
            break;
        }
        page += 1;
    }

    Ok(results)
}
