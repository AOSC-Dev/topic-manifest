use crate::network::{create_client, fetch_descriptions};
use crate::parser;
use anyhow::{anyhow, Result};
use fs::DirEntry;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, SystemTime},
};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicManifest {
    name: String,
    description: Option<String>,
    date: u64,
    arch: Vec<String>,
    packages: Vec<String>,
}

#[inline]
pub fn generate_manifest(manifest: &[TopicManifest]) -> Result<String> {
    Ok(to_string(manifest)?)
}

/// Scan the topic under the given path
fn scan_topic(topic_path: DirEntry) -> Result<TopicManifest> {
    let created = topic_path
        .metadata()?
        .created()
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::new(0, 0))
        .as_secs();
    let topic_name = topic_path.file_name();
    info!("Scanning topic {:?}", topic_name);
    // do not include "stable" as a topic
    if topic_name.to_string_lossy() == "stable" {
        return Err(anyhow!("'stable' is not a topic"));
    }
    let topic_dir = topic_path.path().join("main");
    // HashSet is used here since it's nearly O(1) in time in contrast to Vec which is O(n) in time when searching
    let mut all_names: HashSet<String> = HashSet::new();
    let mut all_arch: Vec<String> = Vec::new();
    for arch in fs::read_dir(topic_dir)? {
        let entry = arch?;
        if entry.file_type()?.is_dir() && entry.file_name().to_string_lossy().starts_with("binary-")
        {
            let name = entry.file_name();
            let arch_name = &name.to_string_lossy()[7..]; // strip "binary-" prefix
            let packages = entry.path().join("Packages");
            let contents = fs::read(packages)?;
            let names = parser::extract_all_names(&contents);
            if let Ok((left, names)) = names {
                if !left.is_empty() {
                    warn!(
                        "Parser encountered issues, {} bytes remain unparsed",
                        left.len()
                    );
                }
                // since the parser works with &[u8], we need to convert to String before serializing to JSON
                for name in names {
                    all_names.insert(std::str::from_utf8(name)?.to_owned());
                }
                all_arch.push(arch_name.to_owned());
            }
        }
    }

    Ok(TopicManifest {
        name: topic_name.to_string_lossy().to_string(),
        description: None,
        date: created,
        arch: all_arch,
        packages: all_names.into_iter().collect::<Vec<String>>(),
    })
}

/// Returns all the topics under the given path
pub fn collect_topics(repo: &str, path: &Path) -> Result<Vec<TopicManifest>> {
    let dist_dir = path.join("dists");
    let mut manifests: Vec<TopicManifest> = Vec::new();
    let mut descriptions = None;
    let client = create_client();
    info!("Fetching topic descriptions from GitHub ...");
    match client {
        Ok(client) => match fetch_descriptions(&client, repo) {
            Ok(d) => descriptions = Some(d),
            Err(e) => error!("Failed to fetch descriptions: {}", e),
        },
        Err(e) => error!("Failed to create a HTTP client: {}", e),
    }
    let descriptions = descriptions.unwrap_or_else(|| {
        warn!("Descriptions unavailable");
        HashMap::new()
    });

    let topics = fs::read_dir(dist_dir)?;
    for topic in topics {
        let manifest = scan_topic(topic?);
        if let Err(e) = manifest {
            warn!("Error scanning topic: {:?}. Topic ignored.", e);
            continue;
        }
        let mut manifest = manifest.unwrap();
        if let Some(desc) = descriptions.get(&manifest.name) {
            manifest.description = Some(desc.clone());
        } else {
            warn!("{}: No description available.", manifest.name);
        }
        manifests.push(manifest);
    }

    Ok(manifests)
}
