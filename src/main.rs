use anyhow::{Context, Result};
mod controllers;
mod led;
mod service;
use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use led::engine;
use service::env::EnvSettings;

#[actix_web::main]
async fn main() -> Result<()> {
    let env_settings = EnvSettings::from_env();
    let device = led::device::open(
        &env_settings.led_spi_device,
        env_settings.led_spi_speed_hz,
        env_settings.led_color_order,
        env_settings.led_count,
    )
    .context(
        "failed to open the LED output — is SPI enabled (raspi-config) and the wire on GPIO 10?",
    )?;
    let engine_handle = engine::spawn(
        device,
        env_settings.led_count,
        env_settings.led_rainbow_speed,
    );

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allow_any_method()
                    .send_wildcard(),
            )
            .app_data(web::Data::new(engine_handle.clone()))
            .service(
                web::resource("/ble-status")
                    .route(web::get().to(controllers::ble_status::controller)),
            )
            .service(
                web::scope("/led")
                    .route(
                        "/color",
                        web::post().to(controllers::led::single_color::controller),
                    )
                    .route(
                        "/brightness",
                        web::post().to(controllers::led::brightness::controller),
                    )
                    .route(
                        "/debug",
                        web::post().to(controllers::led::send_text::controller),
                    )
                    .route(
                        "/rainbow",
                        web::post().to(controllers::led::rainbow::controller),
                    )
                    .route(
                        "/range",
                        web::post().to(controllers::led::range::controller),
                    )
                    .route("/off", web::post().to(controllers::led::off::controller))
                    .route(
                        "/events",
                        web::get().to(controllers::led::events::controller),
                    ),
            )
    })
    .bind((env_settings.web_addr.as_str(), env_settings.web_port))?
    .run()
    .await?;
    Ok(())
}
