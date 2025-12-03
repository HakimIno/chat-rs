use actix_web::{post, web, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use core::entities::{users, devices, one_time_prekeys};
use infrastructure::crypto::signal::{
    generate_identity_keypair, generate_registration_id, generate_signed_prekey, generate_prekeys
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::config::Config;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use rand::Rng;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub device_id: i64,
    pub exp: i64,
    pub iat: i64,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestOtpRequest {
    pub phone_number: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyOtpRequest {
    pub phone_number: String,
    pub otp: String,
    pub device_uuid: Uuid,
    pub device_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyOtpResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: Uuid,
    pub device_id: i64,
    pub is_new_user: bool,
}

#[post("/api/v1/auth/request-otp")]
pub async fn request_otp(
    redis_conn: web::Data<MultiplexedConnection>,
    req: web::Json<RequestOtpRequest>,
) -> impl Responder {
    let mut conn = redis_conn.get_ref().clone();

    let otp: String = (0..6).map(|_| rand::thread_rng().gen_range(0..10).to_string()).collect();
    let key = format!("otp:{}", req.phone_number);

    // Store in Redis with 180s expiration
    if let Err(e) = conn.set_ex::<_, _, ()>(&key, &otp, 180).await {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Redis set error: {}", e)
        }));
    }

    println!("OTP for {} is {}", req.phone_number, otp);

    HttpResponse::Ok().json(serde_json::json!({
        "message": "OTP sent successfully"
    }))
}

#[post("/api/v1/auth/verify-otp")]
pub async fn verify_otp(
    db: web::Data<DatabaseConnection>,
    redis_conn: web::Data<MultiplexedConnection>,
    config: web::Data<Config>,
    req: web::Json<VerifyOtpRequest>,
) -> impl Responder {
    let mut conn = redis_conn.get_ref().clone();

    let key = format!("otp:{}", req.phone_number);
    let stored_otp: Option<String> = match conn.get(&key).await {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Redis get error: {}", e)
            }));
        }
    };

    if stored_otp.is_none() || stored_otp.unwrap() != req.otp {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid or expired OTP"
        }));
    }

    // Start transaction
    let txn = match db.begin().await {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database transaction error: {}", e)
        })),
    };

    // Check if user exists
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;
    
    let (user, is_new_user) = match users::Entity::find()
        .filter(users::Column::PhoneNumber.eq(&req.phone_number))
        .one(&txn)
        .await
    {
        Ok(Some(u)) => (u, false),
        Ok(None) => {
            // Create new user
            let new_user = users::ActiveModel {
                user_id: Set(Uuid::new_v4()),
                phone_number: Set(req.phone_number.clone()),
                phone_number_hash: Set(Vec::new()), // Placeholder or hash logic
                username: Set(None),
                display_name: Set(None),
                bio: Set(None),
                profile_picture: Set(None),
                last_seen_at: Set(None),
                is_online: Set(false),
                is_deleted: Set(false),
                deleted_at: Set(None),
                created_at: Set(Utc::now().into()),
                updated_at: Set(Utc::now().into()),
            };
            match new_user.insert(&txn).await {
                Ok(u) => (u, true),
                Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to create user: {}", e)
                })),
            }
        },
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database error: {}", e)
        })),
    };

    // Generate Signal Keys
    let (identity_key_pair, _) = match generate_identity_keypair() {
        Ok(k) => k,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to generate identity keys: {}", e)
        })),
    };
    let registration_id = generate_registration_id();
    let signed_prekey = match generate_signed_prekey(&identity_key_pair, 1) {
        Ok(k) => k,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to generate signed prekey: {}", e)
        })),
    };
    let one_time_prekeys_list = match generate_prekeys(1, 100) {
        Ok(k) => k,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to generate prekeys: {}", e)
        })),
    };

    // Create Device
    let device = devices::ActiveModel {
        // device_id is auto increment, so we don't set it (or set to NotSet if using ActiveModel default)
        // But wait, in the previous code it was Set(0) which implies it might not be auto increment or SeaORM handles 0 as auto?
        // Let's check the migration. m20251201000002_create_devices.rs.
        // It says .big_integer().not_null().auto_increment().primary_key()
        // So we should NOT set device_id.
        user_id: Set(user.user_id),
        device_uuid: Set(req.device_uuid),
        device_name: Set(req.device_name.clone()),
        platform: Set(1), // Default platform
        identity_key_public: Set(identity_key_pair.public_key),
        registration_id: Set(registration_id as i32),
        signed_prekey_id: Set(signed_prekey.id as i32),
        signed_prekey_public: Set(signed_prekey.public_key),
        signed_prekey_signature: Set(signed_prekey.signature),
        last_seen_at: Set(Utc::now().into()),
        created_at: Set(Utc::now().into()),
        ..Default::default() // To handle device_id and others
    };

    let device = match device.insert(&txn).await {
        Ok(d) => d,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create device: {}", e)
        })),
    };

    // Insert One Time Prekeys
    for prekey in one_time_prekeys_list {
        let otpk = one_time_prekeys::ActiveModel {
            device_id: Set(device.device_id),
            prekey_id: Set(prekey.id as i32),
            public_key: Set(prekey.public_key),
        };
        if let Err(e) = otpk.insert(&txn).await {
             return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to insert prekey: {}", e)
            }));
        }
    }

    if let Err(e) = txn.commit().await {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Transaction commit error: {}", e)
        }));
    }

    // Generate JWT
    let now = Utc::now();
    let access_claims = Claims {
        sub: user.user_id.to_string(),
        device_id: device.device_id,
        iat: now.timestamp(),
        exp: (now + Duration::minutes(15)).timestamp(),
        token_type: "access".to_string(),
    };

    let refresh_claims = Claims {
        sub: user.user_id.to_string(),
        device_id: device.device_id,
        iat: now.timestamp(),
        exp: (now + Duration::days(30)).timestamp(),
        token_type: "refresh".to_string(),
    };

    let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
    let access_token = encode(&Header::default(), &access_claims, &encoding_key).unwrap();
    let refresh_token = encode(&Header::default(), &refresh_claims, &encoding_key).unwrap();

    HttpResponse::Ok().json(VerifyOtpResponse {
        access_token,
        refresh_token,
        user_id: user.user_id,
        device_id: device.device_id,
        is_new_user,
    })
}
