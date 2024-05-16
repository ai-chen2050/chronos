use std::sync::Arc;
use chrono::{Local, NaiveDateTime};
use db_sql::pg::entities::merge_logs;
use node_api::config::ZchronodConfig;
// use db_sql::api::{DbKindZchronod, DbWrite};
use db_sql::pg::entities::{clock_infos, prelude::{ClockInfos, MergeLogs}};
use sea_orm::*;
use tools::helper::sha256_str_to_hex;
use crate::vlc::ClockInfo;
use crate::vlc::MergeLog;

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
    }
    
    // postgre inner api
    pub async fn sinker_clock(&self, message_id: String, raw_message: String, clock_info: &ClockInfo) {
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
        let res = ClockInfos::insert(clock_info).exec(self.pg_db.as_ref()).await;
        if let Err(err) = res {
            eprintln!("Insert clock_info error, err: {}", err);
        }
    }

    pub async fn sinker_merge_log(&self, fclock_info: &ClockInfo, tclock_info: &ClockInfo) {
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
        let res = MergeLogs::insert(merge_log).exec(self.pg_db.as_ref()).await;
        if let Err(err) = res {
            eprintln!("Insert merge_log error, err: {}", err);
        }
    }

    pub async fn get_clock_by_msgid(&self, msg_id: &str) -> Result<ClockInfo, DbErr> {
        let clock_info = ClockInfos::find().filter(clock_infos::Column::MessageId.eq(msg_id)).one(self.pg_db.as_ref()).await;
        match clock_info {
            Err(err) => {
                eprintln!("Query clockinfos by msg_id error, err: {}", err);
                Err(err)
            }
            Ok(None) => {
                let err = DbErr::RecordNotFound(format!("when msg_id is {}", msg_id));
                eprintln!("RecordNotFound: Clock not found for msg_id: {}", msg_id);
                Err(err)
            }
            Ok(Some(clock)) => {
                let clock_ret: ClockInfo = clock.into();
                return Ok(clock_ret);
            }
        }
    }

    pub async fn get_clocks_by_keyid(&self, start_id: u64, number: u64) -> Result<Vec<ClockInfo>, DbErr> {
        let clock_infos= ClockInfos::find()
            .filter(clock_infos::Column::Id.gt(start_id))
            .limit(number)
            .all(self.pg_db.as_ref()).await;

        match clock_infos {
            Err(err) => {
                eprintln!("Query clockinfos by start_id error, err: {}", err);
                Err(err)
            }
            Ok(clocks) => {
                let clock_rets = clocks.iter().map(|clock| (clock.clone().into())).collect();
                return Ok(clock_rets);
            }
        }
    }

    pub async fn get_mergelogs_by_keyid(&self, start_id: u64, number: u64) -> Result<Vec<MergeLog>, DbErr> {
        let merge_logs= MergeLogs::find()
            .filter(merge_logs::Column::Id.gt(start_id))
            .limit(number)
            .all(self.pg_db.as_ref()).await;

        match merge_logs {
            Err(err) => {
                eprintln!("Query merge_logs by start_id error, err: {}", err);
                Err(err)
            }
            Ok(logs) => {
                let mergelog_rets = logs.iter().map(|log| (log.clone().into())).collect();
                return Ok(mergelog_rets);
            }
        }
    }
}
