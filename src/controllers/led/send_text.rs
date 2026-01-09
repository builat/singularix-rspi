use actix_web::{HttpRequest, HttpResponse, error::HttpError, web};

use crate::ble_connector::led_manager::LedBleManager;
/*
    Controller for debugging sending raw text and translate it directly to the LED strip over BLE interface.
*/
pub async fn controller(
    _req: HttpRequest,
    payload: String,
    ble: web::Data<LedBleManager>,
) -> Result<HttpResponse, HttpError> {
    let _ = ble.write_chunks(payload.as_bytes()).await;
    Ok(HttpResponse::Ok().body("Text sent"))
}
