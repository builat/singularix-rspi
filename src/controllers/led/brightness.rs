use actix_web::{HttpRequest, HttpResponse, error::HttpError, web};

use crate::ble_connector::led_manager::LedBleManager;

pub async fn controller(
    _req: HttpRequest,
    payload: web::Json<i32>,
    ble: web::Data<LedBleManager>,
) -> Result<HttpResponse, HttpError> {
    let mut brightness = *payload;
    if brightness < 0 {
        brightness = 0;
    } else if brightness > 255 {
        brightness = 255;
    }
    let _ = ble
        .write_chunks(format!("5 {}", brightness).as_bytes())
        .await;
    Ok(HttpResponse::Ok().body("Brightness set"))
}
