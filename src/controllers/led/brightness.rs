use actix_web::{HttpRequest, HttpResponse, error::HttpError, web};

use crate::led::engine::LedEngineHandle;
use crate::led::protocol;

pub async fn controller(
    _req: HttpRequest,
    payload: web::Json<i32>,
    engine: web::Data<LedEngineHandle>,
) -> Result<HttpResponse, HttpError> {
    let mut brightness = *payload;
    if brightness < 0 {
        brightness = 0;
    } else if brightness > 255 {
        brightness = 255;
    }
    engine.send(protocol::brightness(brightness));
    Ok(HttpResponse::Ok().body("Brightness set"))
}
