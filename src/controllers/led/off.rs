use actix_web::{HttpRequest, HttpResponse, error::HttpError, web};

use crate::led::engine::LedEngineHandle;
use crate::led::protocol;

pub async fn controller(
    _req: HttpRequest,
    engine: web::Data<LedEngineHandle>,
) -> Result<HttpResponse, HttpError> {
    engine.send(protocol::off());
    Ok(HttpResponse::Ok().body("LEDs off"))
}
