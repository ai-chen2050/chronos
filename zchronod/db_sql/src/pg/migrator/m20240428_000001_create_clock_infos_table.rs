use sea_orm_migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240428_000001_create_clock_infos_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the clock_infos table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ClockInfos::Table)
                    .col(
                        ColumnDef::new(ClockInfos::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ClockInfos::Clock).string().not_null())
                    .col(ColumnDef::new(ClockInfos::ClockHash).char_len(64).unique_key().not_null())
                    .col(ColumnDef::new(ClockInfos::NodeId).char_len(32).not_null())
                    .col(ColumnDef::new(ClockInfos::MessageId).text().not_null())
                    .col(ColumnDef::new(ClockInfos::RawMessage).text().not_null())
                    .col(ColumnDef::new(ClockInfos::EventCount).integer().not_null())
                    .col(ColumnDef::new(ClockInfos::CreateAt).timestamp())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the ClockInfo table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ClockInfos::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum ClockInfos {
    Table,
    Id,
    Clock,
    ClockHash,
    NodeId,
    MessageId,
    RawMessage,
    EventCount,
    CreateAt
}