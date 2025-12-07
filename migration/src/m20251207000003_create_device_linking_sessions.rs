use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DeviceLinkingSessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DeviceLinkingSessions::SessionId)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(
                        ColumnDef::new(DeviceLinkingSessions::PrimaryDeviceId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DeviceLinkingSessions::QrCodeToken)
                            .text()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(DeviceLinkingSessions::Status)
                            .small_integer()
                            .not_null()
                            .default(1), // 1 = pending, 2 = approved, 3 = expired, 4 = rejected
                    )
                    .col(
                        ColumnDef::new(DeviceLinkingSessions::NewDeviceUuid)
                            .uuid(),
                    )
                    .col(
                        ColumnDef::new(DeviceLinkingSessions::NewDeviceName)
                            .text(),
                    )
                    .col(
                        ColumnDef::new(DeviceLinkingSessions::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DeviceLinkingSessions::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(DeviceLinkingSessions::ApprovedAt)
                            .timestamp_with_time_zone(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_linking_sessions_primary_device")
                            .from(DeviceLinkingSessions::Table, DeviceLinkingSessions::PrimaryDeviceId)
                            .to(Devices::Table, Devices::DeviceId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index for finding sessions by token
        manager
            .create_index(
                Index::create()
                    .name("idx_linking_sessions_token")
                    .table(DeviceLinkingSessions::Table)
                    .col(DeviceLinkingSessions::QrCodeToken)
                    .to_owned(),
            )
            .await?;

        // Create index for finding pending sessions
        manager
            .create_index(
                Index::create()
                    .name("idx_linking_sessions_status_expires")
                    .table(DeviceLinkingSessions::Table)
                    .col(DeviceLinkingSessions::Status)
                    .col(DeviceLinkingSessions::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DeviceLinkingSessions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum DeviceLinkingSessions {
    Table,
    SessionId,
    PrimaryDeviceId,
    QrCodeToken,
    Status,
    NewDeviceUuid,
    NewDeviceName,
    ExpiresAt,
    CreatedAt,
    ApprovedAt,
}

#[derive(DeriveIden)]
enum Devices {
    Table,
    DeviceId,
}
