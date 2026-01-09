use actix_web::{HttpResponse, error::HttpError, web};

use crate::ble_connector::led_manager::LedBleManager;
#[derive(serde::Deserialize)]
pub struct Body {
    start: u16,
    end: u16,
    r: u8,
    g: u8,
    b: u8,
}
pub async fn controller(
    ble: web::Data<LedBleManager>,
    body: web::Json<Body>,
) -> Result<HttpResponse, HttpError> {
    let frame = format!(
        "3 {} {} {} {} {}\n",
        body.start, body.end, body.r, body.g, body.b
    )
    .into_bytes();
    let _ = ble.write_chunks(&frame).await;

    Ok(HttpResponse::Ok().body("Color set"))
}
