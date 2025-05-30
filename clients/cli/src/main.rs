// Copyright (c) 2024 Nexus. All rights reserved.

mod analytics;
mod config;
mod environment;
#[path = "proto/nexus.orchestrator.rs"]
mod nexus_orchestrator;
mod orchestrator_client;
mod prover;
mod setup;
mod utils;
use std::sync::{Arc, Mutex};

use crate::prover::start_prover;
use crate::setup::{clear_node_config, SetupResult};
use crate::utils::system_stats::measure_gflops;
use clap::{Parser, Subcommand};
use colored::Colorize;
use log::error;
use std::error::Error;
use std::path::PathBuf;
use std::thread;
use tokio::runtime::Runtime;

#[derive(clap::ValueEnum, Clone, Debug)]
enum Environment {
    Local,
    Dev,
    Staging,
    Beta,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Command to execute
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the prover
    Start {
        /// Environment to connect to.
        #[arg(long, value_enum)]
        env: Option<Environment>,

        /// Maximum number of threads to use for proving.
        #[arg(long)]
        max_threads: Option<u32>,

        // 新增：支持多个 node id
        #[arg(long, value_delimiter = ',')]
        node_ids: Option<Vec<u64>>,
    },
    /// Logout from the current session
    Logout,
}

/// Get the path to the Nexus config file, typically located at ~/.nexus/config.json.
fn get_config_path() -> Result<PathBuf, ()> {
    let home_path = home::home_dir().expect("Failed to get home directory");
    let config_path = home_path.join(".nexus").join("config.json");
    Ok(config_path)
}

/// Displays the splash screen with branding and system information.
fn display_splash_screen(environment: &environment::Environment) {
    utils::banner::print_banner();
    println!();
    println!(
        "{}: {}",
        "Computational capacity of this node".bold(),
        format!("{:.2} GFLOPS", measure_gflops()).bright_cyan()
    );
    println!(
        "{}: {}",
        "Environment".bold(),
        environment.to_string().bright_cyan()
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize default log level, but can be overridden by the RUST_LOG environment variable.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();

    match cli.command {
        Command::Start { env, max_threads,node_ids } => {
            let environment = environment::Environment::from(env);
            display_splash_screen(&environment);
            let config_path = get_config_path().expect("Failed to get config path");
            match setup::run_initial_setup(&config_path).await? {
                // == CLI is not registered yet. Perform local proving ==
                SetupResult::Anonymous => {
                    println!("Proving anonymously...");
                    prove_parallel(environment, None, max_threads).await.expect("prove_parallel panic");
                }

                // == CLI is registered and connected ==
                SetupResult::Connected(node_ids) => {
                    println!("Proving with existing node id: {:?}", node_ids);
                    // let node_id: u64 = node_id
                    //     .parse()
                    //     .unwrap_or_else(|_| panic!("invalid node id {}", node_id));
                    // let node_id: u64 = 3867576;
                    loop {
                        let mut error_count = 0;
                        // 多 node id 轮询
                        for node_id in &node_ids {
                            let node_id: u64 = node_id
                                .parse()
                                .unwrap_or_else(|_| panic!("invalid node id {}", node_id));
                            match prove_parallel(environment, Some(node_id), max_threads).await {
                                Err(e) => {
                                    error!("节点 {} 证明失败: {}", node_id, e);
                                    error_count += 1;
                                }
                                Ok(_) => {}
                            }
                        }
                    }
                }

                // == Something went wrong during setup ==
                SetupResult::Invalid => {
                    error!("Invalid setup option selected.");
                    return Err("Invalid setup option selected".into());
                }
            }
        }
        Command::Logout => {
            let config_path = get_config_path().expect("Failed to get config path");
            println!(
                "\n===== {} =====\n",
                "Logging out of the Nexus CLI"
                    .bold()
                    .underline()
                    .bright_cyan()
            );
            clear_node_config(&config_path)?;
        }
    }

    Ok(())
}

/// Proves in parallel using multiple threads.
///
/// # Arguments
/// * `environment` - The environment to connect to.
/// * `node_id` - The node ID to connect to, if specified.
/// * `max_threads` - The maximum number of threads to use, if specified.
async fn prove_parallel(
    environment: environment::Environment,
    node_id: Option<u64>,
    max_threads: Option<u32>,
)-> Result<(), String> {
    if node_id.is_some() {
        println!(
            "\n===== {} =====\n",
            "Starting proof generation".bold().underline().bright_cyan()
        );
    } else {
        println!(
            "\n===== {} =====\n",
            "Starting Anonymous proof generation for programs"
                .bold()
                .underline()
                .bright_cyan()
        );
    }

    // Choose a reasonable number of threads.
    let num_threads = max_threads.unwrap_or(1).clamp(1, 8);
    let mut handles = Vec::new();
    let error_count = Arc::new(Mutex::new(0usize));
    for i in 0..num_threads {
        let node_id_clone = node_id;
        
        let error_count_clone = Arc::clone(&error_count);
        let handle = thread::spawn(move || {
            // Create a new runtime for each thread
            let rt = Runtime::new().expect("Failed to create Tokio runtime");
           
            rt.block_on(async {
                match start_prover(environment, node_id_clone).await {
                    Ok(()) => println!("Thread {} completed successfully", i),
                    Err(e) => {
                        eprintln!("Thread {} failed: {:?}", i, e);
                        let mut count = error_count_clone.lock().unwrap();
                        *count += 1;
                        return;
                    }
                }
            });
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    let failed = *error_count.lock().unwrap();

    if failed == num_threads as usize {
        return Err("所有线程均失败，证明任务未完成".to_string());
    }
    println!("All provers finished.");
    Ok(())
}
