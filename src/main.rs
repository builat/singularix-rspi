use anyhow::Result;
mod ble_connector;
mod controllers;
mod service;
use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use ble_connector::led_manager::LedBleManager;
use service::utils::{get_env, get_parsed_env};
#[actix_web::main]
async fn main() -> Result<()> {
    let led_manager = LedBleManager::new().await?;

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allow_any_method()
                    .send_wildcard(),
            )
            .app_data(web::Data::new(led_manager.clone()))
            .service(
                web::resource("/ble-status")
                    .route(web::get().to(controllers::ble_status::controller)),
            )
            .service(
                web::resource("/set-full-color")
                    .route(web::post().to(controllers::set_single_color::controller)),
            )
    })
    .bind((
        get_env("WEB_ADDR").as_str(),
        get_parsed_env::<u16>("WEB_PORT"),
    ))?
    .run()
    .await?;
    Ok(())
}
