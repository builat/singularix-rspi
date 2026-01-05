use actix_web::{HttpRequest, HttpResponse, error::HttpError, web};

use crate::ble_connector::led_manager::LedBleManager;

#[derive(serde::Deserialize)]
pub struct Body {
    pub r: i32,
    pub g: i32,
    pub b: i32,
}

pub async fn controller(
    _req: HttpRequest,
    payload: web::Json<Body>,
    ble: web::Data<LedBleManager>,
) -> Result<HttpResponse, HttpError> {
    let Body { r, g, b } = payload.into_inner();
    let color_setter = format!("4 {} {} {}\n", r, g, b);
    let _ = ble.write_chunks(color_setter.as_bytes()).await;
    Ok(HttpResponse::Ok().body("Color set"))
}
