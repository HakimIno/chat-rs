use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add device type column (1 = Primary, 2 = Linked)
        manager
            .alter_table(
                Table::alter()
                    .table(Devices::Table)
                    .add_column(
                        ColumnDef::new(Devices::DeviceType)
                            .small_integer()
                            .not_null()
                            .default(1), // Default to Primary
                    )
                    .to_owned(),
            )
            .await?;

        // Add is_active column
        manager
            .alter_table(
                Table::alter()
                    .table(Devices::Table)
                    .add_column(
                        ColumnDef::new(Devices::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .to_owned(),
            )
            .await?;

        // Add linked_at column
        manager
            .alter_table(
                Table::alter()
                    .table(Devices::Table)
                    .add_column(ColumnDef::new(Devices::LinkedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await?;

        // Add linked_by_device_id column (FK to devices for Linked devices)
        manager
            .alter_table(
                Table::alter()
                    .table(Devices::Table)
                    .add_column(ColumnDef::new(Devices::LinkedByDeviceId).big_integer())
                    .to_owned(),
            )
            .await?;

        // Create index for faster lookup of user's devices
        manager
            .create_index(
                Index::create()
                    .name("idx_devices_user_id_active")
                    .table(Devices::Table)
                    .col(Devices::UserId)
                    .col(Devices::IsActive)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_devices_user_id_active")
                    .table(Devices::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Devices::Table)
                    .drop_column(Devices::LinkedByDeviceId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Devices::Table)
                    .drop_column(Devices::LinkedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Devices::Table)
                    .drop_column(Devices::IsActive)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Devices::Table)
                    .drop_column(Devices::DeviceType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Devices {
    Table,
    UserId,
    DeviceType,
    IsActive,
    LinkedAt,
    LinkedByDeviceId,
}
