#![allow(missing_docs)]
mod config;
mod debian;
mod destination;
mod fetcher;
mod packages;
mod redhat;
mod server;
mod state;
mod sync;
mod utils;

use crate::config::Config;
use crate::sync::SyncManager;
use clap::{App, Arg};
use std::process::exit;

fn main() {
    env_logger::init();

    let action_validator = |x: String| -> Result<(), String> {
        if vec!["check", "sync", "server"].contains(&x.as_str()) {
            Ok(())
        } else {
            Err("only check, sync, server are valid actions".into())
        }
    };

    let matches = App::new("RepoSync")
        .version("0.1")
        .author("Davide Baldo <davide.baldo@zextras.com>")
        .about("Keep a repository synchronized to an S3 bucket")
        .args(&[
            Arg::with_name("config")
                .long("config")
                .value_name("CONFIG_FILE")
                .help("location of config file")
                .takes_value(true)
                .required(true)
                .index(1),
            Arg::with_name("action")
                .long("action")
                .value_name("ACTION")
                .help("which action to perform")
                .takes_value(true)
                .required(true)
                .validator(action_validator)
                .index(2),
            Arg::with_name("repository")
                .long("repo")
                .value_name("REPO")
                .help("which repo to synchronize, check, sync, or server")
                .takes_value(true)
                .required(false),
        ])
        .get_matches();

    let config_file = matches.value_of("config").unwrap();

    let result = config::load_config(config_file);
    if result.is_err() {
        println!("{}", result.err().unwrap().to_string());
        exit(1);
    }
    let config = result.unwrap();

    let action = matches.value_of("action").unwrap();
    match action {
        "check" => {
            println!("config file is correct");
            exit(0);
        }
        "sync" => {
            if let Some(repo_name) = matches.value_of("repository") {
                let mut repo_names: Vec<String>;
                if repo_name == "all" {
                    repo_names = config.repo.iter().map(|r| r.name.clone()).collect();
                } else {
                    repo_names = vec![repo_name.into()]
                }
                let mut sync_manager = SyncManager::new(config);
                for repo_name in repo_names {
                    println!("starting synchronization of {}", repo_name);
                    let result = sync_manager.sync_repo(&repo_name);
                    if result.is_err() {
                        println!(
                            "failed to synchronize: {}",
                            result.err().unwrap().to_string()
                        );
                        exit(1);
                    }
                    println!("repo fully synchronized");
                }
                exit(0);
            } else {
                println!("missing argument repo");
                exit(1);
            }
        }
        "server" => {
            start_server(SyncManager::new(config));
            exit(0);
        }
        _ => {
            panic!("unknown action {}", action);
        }
    }
}

#[tokio::main]
async fn start_server(sync_manager: SyncManager) {
    server::create(sync_manager, "127.0.0.1:8080").await;
}
