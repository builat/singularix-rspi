use anyhow::Result;
mod ble_connector;
mod controllers;
mod service;
use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use ble_connector::led_manager::LedBleManager;
use service::env::EnvSettings;

#[actix_web::main]
async fn main() -> Result<()> {
    let env_settings = EnvSettings::from_env();
    let led_manager = LedBleManager::new(&env_settings).await?;
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
    .bind((env_settings.web_addr.as_str(), env_settings.web_port))?
    .run()
    .await?;
    Ok(())
}
