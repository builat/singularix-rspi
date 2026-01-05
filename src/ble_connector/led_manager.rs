use crate::service::utils::{get_env, get_parsed_env};
use anyhow::{Result, anyhow};
use bluer::Device;
use bluer::gatt::remote::{Characteristic, Service};
use bluer::{Adapter, AdapterEvent, Address, Uuid};
use futures::StreamExt;
use tokio::time::{Duration, sleep};

#[derive(Clone)]
pub struct LedBleManager {
    pub dev: Device,
    pub rx: Characteristic,
    pub _tx: Characteristic,
}

impl LedBleManager {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let target_name = get_env("BLE_TARGET_NAME");
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;
        let svc_uuid = Uuid::parse_str(&get_env("BLE_SVC_UUID"))?;
        adapter.set_powered(true).await?;
        println!("[BLE] Powering device: {}", adapter.name());
        let dev = Self::connect_with_retry(&adapter, &target_name, svc_uuid).await?;
        println!("[BLE] Connected to {}", dev.address());

        let svc = Self::find_service_by_uuid(&dev, svc_uuid).await?;
        let rx =
            Self::find_char_by_uuid(&svc, Uuid::parse_str(&get_env("BLE_RX_UUID_STR"))?).await?;
        let _tx =
            Self::find_char_by_uuid(&svc, Uuid::parse_str(&get_env("BLE_TX_UUID_STR"))?).await?;

        Ok(LedBleManager { dev, rx, _tx })
    }

    pub async fn is_connected(&self) -> bool {
        self.dev.is_connected().await.unwrap_or(false)
    }

    pub async fn connect_with_retry(
        adapter: &Adapter,
        target_name: &str,
        svc_uuid: Uuid,
    ) -> Result<Device, anyhow::Error> {
        let mut attempts: usize = 0;

        loop {
            attempts += 1;

            let mut events = adapter.discover_devices().await?;
            println!(
                "[BLE] Scanning for '{}' (or service {})...",
                target_name, svc_uuid,
            );

            let addr: Address = loop {
                if let Some(ev) = events.next().await {
                    if let AdapterEvent::DeviceAdded(a) = ev {
                        if let Ok(d) = adapter.device(a) {
                            let name = d.name().await.ok().flatten().unwrap_or_default();
                            let uuids = d.uuids().await.ok().flatten().unwrap_or_default();
                            let svc_ok = uuids.contains(&svc_uuid);
                            let name_ok = name == target_name;

                            if name_ok || svc_ok {
                                println!("[BLE] Found candidate: {:?} name='{name}'", a);
                                break a;
                            } else {
                                println!("[BLE] Ignoring device: {:?} name='{name}'", a);
                            }
                        }
                    }
                }
            };

            drop(events);
            sleep(Duration::from_millis(800)).await;

            let dev = adapter.device(addr)?;
            println!(
                "[BLE] Connecting to {} (attempt {attempts})...",
                dev.address()
            );

            match dev.connect().await {
                Ok(_) => {
                    println!("[BLE] Waiting for connection to be fully established...");
                    for _ in 0..100 {
                        let is_connected = dev.is_connected().await.unwrap_or(false);
                        if is_connected {
                            return Ok(dev);
                        }
                        sleep(Duration::from_millis(800)).await;
                    }
                    eprintln!("[BLE] Connection are not successful after connect(); retrying…");
                }
                Err(e) => {
                    eprintln!("[BLE] connect() failed: {e}");
                }
            }

            let rmd = adapter.remove_device(addr).await;
            if let Err(e) = rmd {
                eprintln!("[BLE] remove_device() failed: {e}");
            }
            sleep(Duration::from_millis(900)).await;

            if attempts >= 100 {
                return Err(anyhow!("unable to connect after {attempts} attempts"));
            }
        }
    }

    async fn find_service_by_uuid(dev: &Device, target: Uuid) -> Result<Service> {
        for s in dev.services().await? {
            if s.uuid().await? == target {
                return Ok(s);
            }
        }
        Err(anyhow!("[BLE] Service {} not found", target))
    }
    async fn find_char_by_uuid(svc: &Service, target: Uuid) -> Result<Characteristic> {
        for c in svc.characteristics().await? {
            if c.uuid().await? == target {
                return Ok(c);
            }
        }
        Err(anyhow!("[BLE] Characteristic {} not found", target))
    }
    pub async fn write_chunks(&self, data: &[u8]) -> Result<()> {
        for chunk in data.chunks(get_parsed_env::<usize>("BLE_CHUNK_SIZE")) {
            let _ = &self.rx.write(chunk).await?;
            sleep(Duration::from_millis(10)).await;
        }
        Ok(())
    }
}
