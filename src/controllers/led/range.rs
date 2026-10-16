use actix_web::{HttpResponse, error::HttpError, web};

use crate::led::engine::LedEngineHandle;
use crate::led::protocol;

#[derive(serde::Deserialize)]
pub struct Body {
    start: u16,
    end: u16,
    r: u8,
    g: u8,
    b: u8,
}
pub async fn controller(
    engine: web::Data<LedEngineHandle>,
    body: web::Json<Body>,
) -> Result<HttpResponse, HttpError> {
    engine.send(protocol::range(
        body.start as i32,
        body.end as i32,
        body.r as i32,
        body.g as i32,
        body.b as i32,
    ));

    Ok(HttpResponse::Ok().body("Color set"))
}
