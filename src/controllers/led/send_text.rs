use actix_web::{HttpRequest, HttpResponse, error::HttpError, web};

use crate::led::engine::LedEngineHandle;
use crate::led::protocol;
/*
    Controller for debugging: accepts a raw firmware command line
    (e.g. `4 255 0 0`) and parses it exactly like the Arduino did.
*/
pub async fn controller(
    _req: HttpRequest,
    payload: String,
    engine: web::Data<LedEngineHandle>,
) -> Result<HttpResponse, HttpError> {
    engine.send(protocol::parse_ascii(payload.trim()));
    Ok(HttpResponse::Ok().body("Text sent"))
}
