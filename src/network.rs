use std::collections::HashMap;

use anyhow::{anyhow, Result};
use attohttpc::{header, Method, RequestBuilder};
use log::warn;
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

fn fetch_description(request: RequestBuilder, page: usize) -> Result<Vec<GitHubPullRequest>> {
    let request = request.params(&[
        ("base", "stable"),
        ("per_page", "100"),
        ("page", &page.to_string()),
    ]);
    let resp = request.try_prepare()?.send()?;
    let payload: Vec<GitHubPullRequest> = resp.json()?;

    Ok(payload)
}

fn create_request(repo: &str) -> Result<RequestBuilder> {
    let api_token = std::env::var("GITHUB_TOKEN");
    let mut request = RequestBuilder::try_new(
        Method::GET,
        format!("https://api.github.com/repos/{}/pulls", repo),
    )?
    .try_header_append(
        header::USER_AGENT,
        concat!("repo-manifest/", env!("CARGO_PKG_VERSION")),
    )?
    .try_header_append(header::ACCEPT, "application/vnd.github.v3+json")?;
    if let Ok(api_token) = api_token {
        request =
            request.try_header_append(header::AUTHORIZATION, format!("token {}", api_token))?;
    } else {
        warn!("Set GITHUB_TOKEN to increase the rate limits!");
    }

    Ok(request)
}

pub fn fetch_descriptions(repo: &str) -> Result<HashMap<String, String>> {
    let repo_name = repo.splitn(2, '/').nth(0);
    repo_name.ok_or_else(|| anyhow!("Invalid repo name: {}", repo))?;
    let mut results: HashMap<String, String> = HashMap::new();
    let mut page = 1usize;
    loop {
        let request = create_request(repo)?;
        let this_page = fetch_description(request, page)?;
        let no_next_page = this_page.len() < 100;
        for entry in this_page {
            results.insert(entry.head.name, entry.title.trim().to_string());
        }
        if no_next_page {
            break;
        }
        page += 1;
    }

    Ok(results)
}
