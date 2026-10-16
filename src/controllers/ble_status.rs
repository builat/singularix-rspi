use actix_web::{HttpRequest, HttpResponse, error::HttpError, web};

use crate::led::engine::LedEngineHandle;

/*
    Kept under the historical /ble-status route: now reports whether the
    LED engine thread is running.
*/
pub async fn controller(
    _req: HttpRequest,
    engine: web::Data<LedEngineHandle>,
) -> Result<HttpResponse, HttpError> {
    let is_alive = engine.is_alive();
    Ok(HttpResponse::Ok().body(is_alive.to_string()))
}
