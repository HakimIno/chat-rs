use actix_web::{post, web, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use core::entities::{users, devices, one_time_prekeys};
use core::signal::wrapper::create_signal_keys;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub phone_number: String,
    pub device_name: Option<String>,
    pub platform: i16,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub device_id: i64,
    pub device_uuid: Uuid,
}

#[post("/api/v1/register")]
pub async fn register(
    db: web::Data<DatabaseConnection>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    let phone_hash = argon2::hash_encoded(
        req.phone_number.as_bytes(),
        b"somesalt",
        &argon2::Config::default(),
    ).unwrap_or_default();

    let signal_keys = match create_signal_keys() {
        Ok(keys) => keys,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to generate Signal keys: {}", e)
        })),
    };

    let user = users::ActiveModel {
        user_id: Set(Uuid::new_v4()),
        phone_number: Set(req.phone_number.clone()),
        phone_number_hash: Set(phone_hash.as_bytes().to_vec()),
        username: Set(None),
        display_name: Set(None),
        bio: Set(None),
        profile_picture: Set(None),
        last_seen_at: Set(None),
        is_online: Set(false),
        is_deleted: Set(false),
        deleted_at: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };

    let user = match user.insert(db.get_ref()).await {
        Ok(u) => u,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create user: {}", e)
        })),
    };

    let device = devices::ActiveModel {
        device_id: Set(0),
        user_id: Set(user.user_id),
        device_uuid: Set(Uuid::new_v4()),
        device_name: Set(req.device_name.clone()),
        platform: Set(req.platform),
        identity_key_public: Set(signal_keys.identity_key_pair.public_key.clone()),
        registration_id: Set(signal_keys.registration_id as i32),
        signed_prekey_id: Set(signal_keys.signed_prekey.id as i32),
        signed_prekey_public: Set(signal_keys.signed_prekey.public_key.clone()),
        signed_prekey_signature: Set(signal_keys.signed_prekey.signature.clone()),
        last_seen_at: Set(chrono::Utc::now().into()),
        created_at: Set(chrono::Utc::now().into()),
    };

    let device = match device.insert(db.get_ref()).await {
        Ok(d) => d,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create device: {}", e)
        })),
    };

    for prekey in signal_keys.one_time_prekeys.iter() {
        let otpk = one_time_prekeys::ActiveModel {
            device_id: Set(device.device_id),
            prekey_id: Set(prekey.id as i32),
            public_key: Set(prekey.public_key.clone()),
        };
        let _ = otpk.insert(db.get_ref()).await;
    }

    HttpResponse::Ok().json(RegisterResponse {
        user_id: user.user_id,
        device_id: device.device_id,
        device_uuid: device.device_uuid,
    })
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub phone_number: String,
    pub device_uuid: Uuid,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: Uuid,
}

#[post("/api/v1/auth/login")]
pub async fn login(
    db: web::Data<DatabaseConnection>,
    req: web::Json<LoginRequest>,
) -> impl Responder {
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;

    let user = match users::Entity::find()
        .filter(users::Column::PhoneNumber.eq(&req.phone_number))
        .one(db.get_ref())
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({
            "error": "User not found"
        })),
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database error: {}", e)
        })),
    };

    let device = match devices::Entity::find()
        .filter(devices::Column::DeviceUuid.eq(req.device_uuid))
        .filter(devices::Column::UserId.eq(user.user_id))
        .one(db.get_ref())
        .await
    {
        Ok(Some(d)) => d,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Device not found"
        })),
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database error: {}", e)
        })),
    };

    let access_token = format!("access_token_{}", user.user_id);
    let refresh_token = format!("refresh_token_{}", user.user_id);

    HttpResponse::Ok().json(LoginResponse {
        access_token,
        refresh_token,
        user_id: user.user_id,
    })
}
