mod zchronod;
mod node_factory;
mod storage;
mod vlc;

use std::path::PathBuf;
use db_sql::pg::pg_client::set_up_db;
use tools::tokio_zhronod;
use node_api::config;
use structopt::StructOpt;
use node_api::error::{ZchronodConfigError, ZchronodConfigResult, ZchronodError, ZchronodResult};
use tracing::*;
use node_api::config::ZchronodConfig;
use crate::zchronod::ZchronodArc;
use crate::zchronod::Zchronod;

#[derive(StructOpt)]
struct ZchronodCli {
    #[structopt(short = "c", long = "config", parse(from_os_str), help = "Yaml file only")]
    config_path: Option<std::path::PathBuf>,

    #[structopt(long = "init_pg", help = "Init & refresh pg, caution: new db & new table")]
    init_pg: Option<String>,
}

fn main() {
    println!("start Zchronod");
    tokio_zhronod::block_forever_on(async_main());
}

async fn async_main() {
    let mut help_info = true;
    let args = ZchronodCli::from_args();

    // init pg db
    if let Some(pg_conn_str) = args.init_pg {
        help_info = false;
        println!("PostgreSQL connection addr: {}", pg_conn_str);
        // Use the PostgreSQL connection string here for initialization
        if !init_db(pg_conn_str).await {
            return;
        }
    }

    // setup node
    if let Some(config_path) = args.config_path {
        help_info = false;
        let zchronod_config = construct_node_config(config_path.clone());
        // let db_root_path = zchronod_config.storage_root_path.unwrap();
    
        //todo metrics init
    
        let zchronod = build_zchronod(zchronod_config.clone()).await;
    }

    if help_info {
        println!("\nPlease exec: Zchronod -h for help info.\n")
    }
    
    // shutdown Zchronod
    // tokio::signal::ctrl_c()
    //     .await
    //     .unwrap_or_else(|e| tracing::error!("Could not handle termination signal: {:?}", e));
    // tracing::info!("Gracefully shutting down Zchronod...");
    // let shutdown_result = zchronod.shutdown().await;
    // handle_shutdown(shutdown_result);
}

async fn init_db(pg_conn_str: String) -> bool {
    if let Ok(url) = url::Url::parse(&pg_conn_str) {
        let db_name = url.path().trim_start_matches('/');
        let base_url = url.as_str().trim_end_matches(db_name);
    
        println!("Base URL: {}", base_url);
        println!("Database Name: {}", db_name);
        if db_name == "" {
            println!("Database name is empty, exit");
            return false;
        }
        let database_connection = set_up_db(base_url, db_name).await;
        match database_connection {
            Err(err) => {
                println!("{}", err);
                return false;
            }
            Ok(conn) => {
                println!("Setup database success");
                let _ = conn.close().await;
                return true;
            },
        }
    } else {
        eprintln!("Invalid PostgreSQL connection string");
        return false;
    }
}

async fn build_zchronod(config: ZchronodConfig) -> ZchronodArc {
    Zchronod::zchronod_factory().set_config(config).produce().await.map_err(|e| {
        panic!("Failed to build Zchronod due to error [{:?}]", e);
    }).unwrap()
}

fn construct_node_config(config_path: PathBuf) -> config::ZchronodConfig {
    match config::ZchronodConfig::load_config(config_path) {
        Err(ZchronodConfigError::ConfigMissing(_)) => {
            println!("config path can't found.");
            std::process::exit(ERROR_CODE);
        }
        Err(ZchronodConfigError::SerializationError(err)) => {
            println!("config path can't be serialize");
            std::process::exit(ERROR_CODE);
        }
        result => {
            result.expect("failed to load zhronod config")
        }
    }
}

/// start Zchronod node error code for loading config
pub const ERROR_CODE: i32 = 42;