use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "conversations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub conv_id: Uuid,
    pub conv_type: i16,
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub creator_id: Option<Uuid>,
    pub metadata: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::CreatorId",
        to = "super::users::Column::UserId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Users,
    #[sea_orm(has_many = "super::conv_members::Entity")]
    ConvMembers,
    #[sea_orm(has_many = "super::messages::Entity")]
    Messages,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::conv_members::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ConvMembers.def()
    }
}

impl Related<super::messages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
