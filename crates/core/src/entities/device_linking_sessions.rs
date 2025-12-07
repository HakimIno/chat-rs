use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Status of a device linking session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkingStatus {
    Pending = 1,
    Approved = 2,
    Expired = 3,
    Rejected = 4,
}

impl From<i16> for LinkingStatus {
    fn from(v: i16) -> Self {
        match v {
            1 => LinkingStatus::Pending,
            2 => LinkingStatus::Approved,
            3 => LinkingStatus::Expired,
            4 => LinkingStatus::Rejected,
            _ => LinkingStatus::Pending,
        }
    }
}

impl From<LinkingStatus> for i16 {
    fn from(s: LinkingStatus) -> Self {
        s as i16
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "device_linking_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub session_id: Uuid,
    pub primary_device_id: i64,
    #[sea_orm(unique)]
    pub qr_code_token: String,
    pub status: i16, // 1 = pending, 2 = approved, 3 = expired, 4 = rejected
    pub new_device_uuid: Option<Uuid>,
    pub new_device_name: Option<String>,
    pub expires_at: DateTimeWithTimeZone,
    pub created_at: DateTimeWithTimeZone,
    pub approved_at: Option<DateTimeWithTimeZone>,
}

impl Model {
    pub fn linking_status(&self) -> LinkingStatus {
        LinkingStatus::from(self.status)
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }

    pub fn is_pending(&self) -> bool {
        self.status == LinkingStatus::Pending as i16 && !self.is_expired()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::devices::Entity",
        from = "Column::PrimaryDeviceId",
        to = "super::devices::Column::DeviceId",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    PrimaryDevice,
}

impl Related<super::devices::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PrimaryDevice.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
