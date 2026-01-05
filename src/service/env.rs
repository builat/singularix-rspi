use dotenvy::dotenv;
use std::env;

#[derive(Clone)]
pub struct EnvSettings {
    pub web_addr: String,
    pub web_port: u16,
    pub ble_target_name: String,
    pub ble_chunk_size: usize,
    pub ble_svc_uuid: String,
    pub ble_rx_uuid_str: String,
    pub ble_tx_uuid_str: String,
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
            ble_target_name: env::var("BLE_TARGET_NAME").expect("BLE_TARGET_NAME not set"),
            ble_chunk_size: env::var("BLE_CHUNK_SIZE")
                .expect("BLE_CHUNK_SIZE not set")
                .parse::<usize>()
                .expect("BLE_CHUNK_SIZE must be a valid usize"),
            ble_svc_uuid: env::var("BLE_SVC_UUID").expect("BLE_SVC_UUID not set"),
            ble_rx_uuid_str: env::var("BLE_RX_UUID_STR").expect("BLE_RX_UUID_STR not set"),
            ble_tx_uuid_str: env::var("BLE_TX_UUID_STR").expect("BLE_TX_UUID_STR not set"),
        }
    }
}
