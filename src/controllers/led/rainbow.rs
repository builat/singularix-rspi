use actix_web::{HttpRequest, HttpResponse, error::HttpError, web};

use crate::ble_connector::led_manager::LedBleManager;

pub async fn controller(
    _req: HttpRequest,
    ble: web::Data<LedBleManager>,
) -> Result<HttpResponse, HttpError> {
    let _ = ble.write_chunks(format!("1").as_bytes()).await;
    Ok(HttpResponse::Ok().body("rainbow set"))
}
