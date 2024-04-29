use sea_orm_migration::prelude::*;

mod m20240428_000001_create_clock_infos_table;
mod m20240428_000001_create_merge_logs_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240428_000001_create_clock_infos_table::Migration),
            Box::new(m20240428_000001_create_merge_logs_table::Migration),
        ]
    }
}