use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures::StreamExt;
use uuid::Uuid;
use super::connection::{ConnectionManager, WsConnection};

#[get("/ws/")]
pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    manager: web::Data<ConnectionManager>,
) -> Result<HttpResponse, Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    let conn_id = Uuid::new_v4();
    let user_id = Uuid::new_v4(); // TODO: Extract from JWT
    let device_id = 1; // TODO: Extract from JWT

    let ws_conn = WsConnection {
        user_id,
        device_id,
        conn_id,
    };

    manager.add_connection(ws_conn.clone()).await;

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Text(text) => {
                    tracing::debug!("Received text message: {}", text);
                    let _ = session.text(format!("Echo: {}", text)).await;
                }
                Message::Binary(bin) => {
                    tracing::debug!("Received binary message: {} bytes", bin.len());
                    let _ = session.binary(bin).await;
                }
                Message::Ping(bytes) => {
                    let _ = session.pong(&bytes).await;
                }
                Message::Close(reason) => {
                    tracing::info!("WebSocket closed: {:?}", reason);
                    break;
                }
                _ => {}
            }
        }

        manager.remove_connection(&conn_id).await;
        tracing::info!("Connection {} closed", conn_id);
    });

    Ok(response)
}
