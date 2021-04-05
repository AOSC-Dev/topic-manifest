use std::path::Path;

use clap::{crate_version, App, Arg};
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

fn main() {
    env_logger::init();
    let matches = App::new("Topics Manifest Generator")
        .arg(
            Arg::with_name("dir")
                .short("d")
                .required(true)
                .value_name("DIR")
                .help("Specify the directory to debs root"),
        )
        .arg(
            Arg::with_name("repo")
                .short("p")
                .required(true)
                .value_name("REPO")
                .help("Specify the GitHub repository name (e.g. AOSC-Dev/aosc-os-abbs)"),
        )
        .version(crate_version!())
        .get_matches();
    let dir = matches.value_of("dir").unwrap();
    let repo = matches.value_of("repo").unwrap();
    let output = Path::new(dir).join("manifest/topics.json");
    info!("Preflight scanning for {}...", dir);
    let topics = unwrap_or_show_error!(
        "Unable to scan for topics: {}{}",
        "",
        scan::collect_topics(repo, Path::new(dir))
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
