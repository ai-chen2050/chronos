use std::collections::HashMap;
use std::sync::Arc;
use node_api::config::ZchronodConfig;
use db_sql::api::{DbKindZchronod, DbWrite};
use sea_orm::{Database, DatabaseConnection};
use tokio::sync::RwLock;

pub struct Storage {
    // pub zchronod_db: DbWrite<DbKindZchronod>,
    // pub pg_db: Arc<RwLock<DatabaseConnection>>
}

impl Storage {
    pub async fn new(config: ZchronodConfig) -> Self {
        // let zchronod_db = DbWrite::open(
        //     config.storage_root_path.as_ref().unwrap().as_path(),
        //     DbKindZchronod,
        // ).unwrap(); // todo error handling
        
        // connect to pg db
        // let url = format!("{}/{}", config.pg_db_url, config.pg_db_name);
        // let pg_db = Database::connect(&url).await.expect("failed to connect to database");
        // let pg_db_arc = Arc::new(RwLock::new(pg_db));
        // Self {
        //     // zchronod_db,
        //     pg_db: pg_db_arc,
        // }
        
        println!("\nskip new db or storage\n");
        Self{}
    }
    pub async fn get(&self, key: String) -> Option<String> {
        todo!()
    }

    pub async fn set(&self, key: String, value: String) -> bool {
        todo!()
    }

    pub async fn delete(&self, key: String) -> bool {
        todo!()
    }

    pub async fn get_all(&self) -> HashMap<String, String> {
        todo!()
    }
}