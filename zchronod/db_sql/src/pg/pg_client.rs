use futures::executor::block_on;
use sea_orm::*;
use sea_orm_migration::prelude::*;
use super::migrator::Migrator;
use super::entities::{prelude::*, *};

const DATABASE_PG_URL: &str = "postgres://postgres:hetu@0.0.0.0:5432";
const DB_NAME: &str = "vlc_inner_db";

pub async fn setup_db(request_url: &str, db_name: &str) -> Result<DatabaseConnection, DbErr>  {
    let db = Database::connect(request_url).await?;
    let db = match db.get_database_backend() {
       DbBackend::MySql => {
           db.execute(Statement::from_string(
               db.get_database_backend(),
               format!("CREATE DATABASE IF NOT EXISTS `{}`;", db_name),
           ))
           .await?;

           let url = format!("{}/{}", request_url, db_name);
           Database::connect(&url).await?
       }
       DbBackend::Postgres => {
           db.execute(Statement::from_string(
               db.get_database_backend(),
               format!("DROP DATABASE IF EXISTS \"{}\";", db_name),
           ))
           .await?;
           db.execute(Statement::from_string(
               db.get_database_backend(),
               format!("CREATE DATABASE \"{}\";", db_name),
           ))
           .await?;

           let url = format!("{}/{}", request_url, db_name);
           Database::connect(&url).await?
       }
       DbBackend::Sqlite => db,
    };

    let schema_manager = SchemaManager::new(&db); // To investigate the schema

    Migrator::up(&db.clone(), None).await?;
    assert!(schema_manager.has_table("clock_infos").await?);
    assert!(schema_manager.has_table("merge_logs").await?);

    Ok(db)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn set_up_db() {   // could add the function to server cli command
        match block_on(setup_db(DATABASE_PG_URL, DB_NAME)) {
            Err(err) => {
                panic!("{}", err);
            }
            Ok(db) => async {  }.await,
        }
    }

    #[tokio::test]
    async fn test_run() {
        let url = format!("{}/{}", DATABASE_PG_URL, DB_NAME);
        let db = Database::connect(&url).await.expect("failed to connect to database");
        {
            let mut clock = vlc::Clock::new();
            clock.inc(0);
            clock.inc(1);
            let clock_str = serde_json::to_string(&clock).unwrap();
            let clock_info = clock_infos::ActiveModel {
                clock: ActiveValue::Set(clock_str.clone()),
                clock_hash: ActiveValue::Set("todo".to_owned()),
                node_id: ActiveValue::Set("todo".to_owned()),
                message_id: ActiveValue::Set("todo".to_owned()),
                raw_message: ActiveValue::Set("todo".to_owned()),
                event_count: ActiveValue::Set(1),
                // create_at: ActiveValue::Set(current_time),
                ..Default::default()
            };
            let res = ClockInfos::insert(clock_info).exec(&db).await.expect("insert error");
            
            let clock_vec = ClockInfos::find().all(&db).await.expect("query error");
            println!("clock_vec-1 = {:?}", clock_vec);
            
            let clock_info2 = clock_infos::ActiveModel {
                id: ActiveValue::Set(res.last_insert_id),
                clock_hash: ActiveValue::Set("todo1".to_owned()),
                node_id: ActiveValue::Set("todo1".to_owned()),
                message_id: ActiveValue::Set("todo1".to_owned()),
                raw_message: ActiveValue::Set("todo1".to_owned()),
                event_count: ActiveValue::Set(2),
                ..Default::default()
            };
            clock_info2.clone().update(&db).await;

            let mut clock3 = clock_info2;
            clock3.id = ActiveValue::Set(2);
            clock3.event_count = ActiveValue::Set(2);
            clock3.clock_hash = ActiveValue::Set("todo2".to_owned());
            clock3.clock = ActiveValue::Set(clock_str);
            println!("clock3 = {:?}", clock3);
            ClockInfos::insert(clock3).exec(&db).await.expect("insert error");
            let clock_vec = ClockInfos::find().all(&db).await.expect("query error");
            println!("clock_vec-2 = {:?}", clock_vec);
        }
    }

    #[tokio::test]
    async fn test_get_clocks_by_msgid() {
        let url = format!("{}/{}", DATABASE_PG_URL, DB_NAME);
        let db = Database::connect(&url).await.expect("failed to connect to database");
        {
            let msg_id = "todo1";
            let clock_info = ClockInfos::find().filter(clock_infos::Column::MessageId.eq(msg_id)).all(&db).await.expect("query error");
            println!("pointed message_id's clocks = {:?}", clock_info);
        }
    }

    #[tokio::test]
    async fn test_get_clocks_from_start_id() {
        let url = format!("{}/{}", DATABASE_PG_URL, DB_NAME);
        let db = Database::connect(&url).await.expect("failed to connect to database");

        let start_id = 0;

        let clocks = ClockInfos::find()
            .filter(clock_infos::Column::Id.gt(start_id))
            .limit(1)
            .all(&db)
            .await
            .expect("query error");

        println!("clocks = {:?}", clocks);
    }
}