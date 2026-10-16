use dotenvy::dotenv;
use std::env;

use crate::led::device::ColorOrder;

#[derive(Clone)]
pub struct EnvSettings {
    pub web_addr: String,
    pub web_port: u16,
    pub led_count: usize,
    pub led_color_order: ColorOrder,
    pub led_spi_device: String,
    pub led_spi_speed_hz: u32,
    pub led_rainbow_speed: u16,
}

impl EnvSettings {
    pub fn from_env() -> Self {
        dotenv().ok();
        EnvSettings {
            web_addr: env::var("WEB_ADDR").expect("WEB_ADDR not set"),
            web_port: env::var("WEB_PORT")
                .expect("WEB_PORT not set")
                .parse::<u16>()
                .expect("WEB_PORT must be a valid u16"),
            // LED_* settings default to the values the Arduino firmware used.
            led_count: env::var("LED_COUNT")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .filter(|&n| n > 0)
                .unwrap_or(274),
            led_color_order: env::var("LED_COLOR_ORDER")
                .ok()
                .and_then(|v| ColorOrder::parse(&v))
                .unwrap_or_default(),
            led_spi_device: env::var("LED_SPI_DEVICE").unwrap_or_else(|_| "/dev/spidev0.0".into()),
            led_spi_speed_hz: env::var("LED_SPI_SPEED_HZ")
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(3_200_000),
            led_rainbow_speed: env::var("LED_RAINBOW_SPEED")
                .ok()
                .and_then(|v| v.parse::<u16>().ok())
                .unwrap_or(180),
        }
    }
}
