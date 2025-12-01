use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "devices")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub device_id: i64,
    pub user_id: Uuid,
    #[sea_orm(unique)]
    pub device_uuid: Uuid,
    pub device_name: Option<String>,
    pub platform: i16,
    pub identity_key_public: Vec<u8>,
    pub registration_id: i32,
    pub signed_prekey_id: i32,
    pub signed_prekey_public: Vec<u8>,
    pub signed_prekey_signature: Vec<u8>,
    pub last_seen_at: DateTimeWithTimeZone,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::UserId",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Users,
    #[sea_orm(has_many = "super::one_time_prekeys::Entity")]
    OneTimePrekeys,
    #[sea_orm(has_many = "super::signal_sessions::Entity")]
    SignalSessions,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::one_time_prekeys::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OneTimePrekeys.def()
    }
}

impl Related<super::signal_sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SignalSessions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
