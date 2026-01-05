use actix_web::{HttpRequest, HttpResponse, error::HttpError, web};

use crate::ble_connector::led_manager::LedBleManager;

pub async fn controller(
    _req: HttpRequest,
    ble: web::Data<LedBleManager>,
) -> Result<HttpResponse, HttpError> {
    let is_connected = ble.is_connected().await;
    Ok(HttpResponse::Ok().body(is_connected.to_string()))
}
