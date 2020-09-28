use crate::parser;
use anyhow::Result;
use log::{info, warn};
use serde_derive::{Deserialize, Serialize};
use serde_json::to_string;
use std::collections::HashSet;
use std::{fs, path::Path};

#[derive(Serialize, Deserialize, Debug)]
pub struct TopicManifest {
    name: String,
    description: Option<String>,
    arch: Vec<String>,
    packages: Vec<String>,
}

#[inline]
pub fn generate_manifest(manifest: &[TopicManifest]) -> Result<String> {
    Ok(to_string(manifest)?)
}

pub fn collect_topics(path: &Path) -> Result<Vec<TopicManifest>> {
    let dist_dir = path.join("dists");
    let mut manifests: Vec<TopicManifest> = Vec::new();
    let topics = fs::read_dir(dist_dir)?;
    for topic in topics {
        let topic_path = topic?;
        let topic_name = topic_path.file_name();
        if topic_name.to_string_lossy() == "stable" {
            continue;
        }
        info!("Scanning topic {:?}", topic_name);
        let topic_dir = topic_path.path().join("main");
        let mut all_names: HashSet<String> = HashSet::new();
        let mut all_arch: Vec<String> = Vec::new();
        for arch in fs::read_dir(topic_dir)? {
            let entry = arch?;
            if entry.file_type()?.is_dir()
                && entry.file_name().to_string_lossy().starts_with("binary-")
            {
                let name = entry.file_name();
                let arch_name = &name.to_string_lossy()[7..];
                let packages = entry.path().join("Packages");
                let contents = fs::read(packages)?;
                let names = parser::extract_all_names(&contents);
                if let Ok((left, names)) = names {
                    if left.is_empty() {
                        warn!(
                            "Parser encountered issues, {} bytes remain unparsed",
                            left.len()
                        );
                    }
                    for name in names {
                        all_names.insert(std::str::from_utf8(name)?.to_owned());
                    }
                    all_arch.push(arch_name.to_owned());
                }
            }
        }
        manifests.push(TopicManifest {
            name: topic_name.to_string_lossy().to_string(),
            description: None,
            arch: all_arch,
            packages: all_names.into_iter().collect::<Vec<String>>(),
        });
    }

    Ok(manifests)
}
