use std::path::Path;

use argh::FromArgs;
use log::{error, info};
use std::fs;

mod network;
mod parser;
mod scan;

macro_rules! unwrap_or_show_error {
    ($m:tt, $p:expr, $f:stmt) => {{
        let tmp = { $f };
        if let Err(e) = tmp {
            error!($m, $p, e);
            return;
        }
        tmp.unwrap()
    }};
    ($m:tt, $p:expr, $x:ident) => {{
        if let Err(e) = $x {
            error!($m, $p, e);
            return;
        }
        $x.unwrap()
    }};
}

#[derive(FromArgs)]
/// Topics Manifest Generator
struct TopicManifest {
    /// specify the directory to debs root
    #[argh(option, short = 'd')]
    dir: String,
    /// specify the GitHub repository name (e.g. AOSC-Dev/aosc-os-abbs)
    #[argh(option, short = 'p')]
    repo: String,
}

fn main() {
    env_logger::init();
    let args: TopicManifest = argh::from_env();
    let dir = args.dir;
    let repo = args.repo;
    let output = Path::new(&dir).join("manifest/topics.json");
    info!("Preflight scanning for {}...", dir);
    let topics = unwrap_or_show_error!(
        "Unable to scan for topics: {}{}",
        "",
        scan::collect_topics(&repo, Path::new(&dir))
    );
    let manifests = unwrap_or_show_error!(
        "Unable to generate manifest file: {}{}",
        "",
        scan::generate_manifest(&topics)
    );
    unwrap_or_show_error!(
        "Unable to write manifest file: {}: {}",
        output.display(),
        fs::write(output.clone(), manifests)
    );
    info!("Topic manifest generated successfully.");
}
