use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "one_time_prekeys")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub device_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub prekey_id: i32,
    pub public_key: Vec<u8>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::devices::Entity",
        from = "Column::DeviceId",
        to = "super::devices::Column::DeviceId",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Devices,
}

impl Related<super::devices::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Devices.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
