use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Local, NaiveDateTime};
use db_sql::pg::entities::merge_logs;
use node_api::config::ZchronodConfig;
use db_sql::api::{DbKindZchronod, DbWrite};
use db_sql::pg::entities::{clock_infos, prelude::{ClockInfos, MergeLogs}};
use sea_orm::{ActiveValue, Database, DatabaseConnection, EntityTrait};
use sha2::{Sha256, Digest};
use tools::helper::sha256_str_to_hex;
use crate::zchronod::ClockInfo;

pub struct Storage {
    // pub zchronod_db: DbWrite<DbKindZchronod>,
    pub pg_db: Arc<DatabaseConnection>
}

impl Storage {
    pub async fn new(config: ZchronodConfig) -> Self {
        // let zchronod_db = DbWrite::open(
        //     config.storage_root_path.as_ref().unwrap().as_path(),
        //     DbKindZchronod,
        // ).unwrap(); // todo error handling
        
        // connect to pg db
        let url = format!("{}/{}", config.pg_db_url, config.pg_db_name);
        let pg_db = Database::connect(&url).await.expect("failed to connect to database");
        let pg_db_arc = Arc::new(pg_db);
        Self {
            // zchronod_db,
            pg_db: pg_db_arc,
        }
        
        // println!("\nskip new db or storage\n");
        // Self{}
    }
    
    // postgre inner api
    pub async fn sinker_clock(&mut self, message_id: String, raw_message: String, clock_info: &ClockInfo) {
        let clock_str = serde_json::to_string(&clock_info.clock).unwrap();
        let hash_hex = sha256_str_to_hex(clock_str.clone());
        let naive_datetime = NaiveDateTime::from_timestamp_millis(clock_info.create_at.try_into().unwrap());
        let clock_info = clock_infos::ActiveModel {
            clock: ActiveValue::Set(clock_str.clone()),
            clock_hash: ActiveValue::Set(hash_hex),
            node_id: ActiveValue::Set(clock_info.node_id.to_owned()),
            message_id: ActiveValue::Set(message_id),
            raw_message: ActiveValue::Set(raw_message),
            event_count: ActiveValue::Set(clock_info.count.try_into().unwrap()),
            create_at: ActiveValue::Set(naive_datetime),
            ..Default::default()
        };
        let _res = ClockInfos::insert(clock_info).exec(self.pg_db.as_ref()).await.expect("insert clock_info error");
    }

    pub async fn sinker_merge_log(&mut self, fclock_info: &ClockInfo, tclock_info: &ClockInfo) {
        let fclock_str = serde_json::to_string(&fclock_info.clock).unwrap();
        let tclock_str = serde_json::to_string(&tclock_info.clock).unwrap();
        let f_hash_hex = sha256_str_to_hex(fclock_str);
        let e_hash_hex = sha256_str_to_hex(tclock_str);
        let now = Local::now().timestamp_millis();
        let naive_datetime = NaiveDateTime::from_timestamp_millis(now).unwrap();
        let merge_log = merge_logs::ActiveModel {
            from_id: ActiveValue::Set(fclock_info.node_id.to_owned()),
            to_id: ActiveValue::Set(tclock_info.node_id.to_owned()),
            start_count: ActiveValue::Set(fclock_info.count.to_owned().try_into().unwrap()),
            end_count: ActiveValue::Set(tclock_info.count.to_owned().try_into().unwrap()),
            s_clock_hash: ActiveValue::Set(f_hash_hex),
            e_clock_hash: ActiveValue::Set(e_hash_hex),
            merge_at: ActiveValue::Set(naive_datetime),
            ..Default::default()
        };
        let _res = MergeLogs::insert(merge_log).exec(self.pg_db.as_ref()).await.expect("insert merge_log error");
    }
}
