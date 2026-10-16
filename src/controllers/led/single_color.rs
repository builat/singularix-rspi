use crate::led::engine::LedEngineHandle;
use crate::led::protocol;
use actix_web::{HttpRequest, HttpResponse, error::HttpError, web};

#[derive(serde::Deserialize)]
pub struct Body {
    pub r: i32,
    pub g: i32,
    pub b: i32,
}

pub async fn controller(
    _req: HttpRequest,
    payload: web::Json<Body>,
    engine: web::Data<LedEngineHandle>,
) -> Result<HttpResponse, HttpError> {
    let Body { r, g, b } = payload.into_inner();
    engine.send(protocol::color(r, g, b));
    Ok(HttpResponse::Ok().body("Color set"))
}
