use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Emitter};

const PIPE_NAME: &str = r"\\.\pipe\HardwareMonitor_31337";
const NAME_SIZE: usize = 128;

#[cfg(target_os = "windows")]
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(target_os = "windows")]
use tokio::net::windows::named_pipe::ClientOptions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hardware {
    pub name: String,
    pub identifier: String,
    pub hardware_type: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensor {
    pub name: String,
    pub identifier: String,
    pub hardware_identifier: String,
    pub sensor_type: i32,
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareMonitorData {
    pub last_poll_time: u64,
    pub hardwares: Vec<Hardware>,
    pub sensors: Vec<Sensor>,
    pub present_mon_apps: Vec<String>,
}

#[derive(Clone)]
pub struct PipeClient {
    #[cfg(target_os = "windows")]
    write_stream: Arc<Mutex<Option<tokio::net::windows::named_pipe::NamedPipeClient>>>,
    #[cfg(not(target_os = "windows"))]
    write_stream: Arc<Mutex<Option<isize>>>,
}

impl PipeClient {
    pub fn new() -> Self {
        Self {
            write_stream: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&self, app_handle: AppHandle) {
        #[cfg(not(target_os = "windows"))]
        {
            // On non-windows, run a mock loop emitting stub metrics so the Svelte frontend can render during dev!
            let _ = app_handle.emit("pipe-status", "mock-connected");
            tokio::spawn(async move {
                let mut mock_time = 0u64;
                loop {
                    mock_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;

                    let mock_data = HardwareMonitorData {
                        last_poll_time: mock_time,
                        hardwares: vec![
                            Hardware { name: "Mock CPU".to_string(), identifier: "/intelcpu/0".to_string(), hardware_type: 2 },
                            Hardware { name: "Mock GPU".to_string(), identifier: "/nvidiagpu/0".to_string(), hardware_type: 4 },
                        ],
                        sensors: vec![
                            Sensor { name: "CPU Core".to_string(), identifier: "/intelcpu/0/temperature/0".to_string(), hardware_identifier: "/intelcpu/0".to_string(), sensor_type: 4, value: 45.5 },
                            Sensor { name: "CPU Total Load".to_string(), identifier: "/intelcpu/0/load/0".to_string(), hardware_identifier: "/intelcpu/0".to_string(), sensor_type: 5, value: 12.0 },
                            Sensor { name: "CPU Package Power".to_string(), identifier: "/intelcpu/0/power/0".to_string(), hardware_identifier: "/intelcpu/0".to_string(), sensor_type: 2, value: 35.0 },
                            Sensor { name: "GPU Core".to_string(), identifier: "/nvidiagpu/0/temperature/0".to_string(), hardware_identifier: "/nvidiagpu/0".to_string(), sensor_type: 4, value: 55.2 },
                            Sensor { name: "GPU Core Load".to_string(), identifier: "/nvidiagpu/0/load/0".to_string(), hardware_identifier: "/nvidiagpu/0".to_string(), sensor_type: 5, value: 38.0 },
                            Sensor { name: "GPU Memory Used".to_string(), identifier: "/nvidiagpu/0/load/3".to_string(), hardware_identifier: "/nvidiagpu/0".to_string(), sensor_type: 5, value: 24.5 },
                            Sensor { name: "GPU D3D Usage".to_string(), identifier: "/nvidiagpu/0/sm/0".to_string(), hardware_identifier: "/nvidiagpu/0".to_string(), sensor_type: 5, value: 1024.0 },
                            Sensor { name: "GPU Package Power".to_string(), identifier: "/nvidiagpu/0/power/0".to_string(), hardware_identifier: "/nvidiagpu/0".to_string(), sensor_type: 2, value: 95.0 },
                            Sensor { name: "Memory Used".to_string(), identifier: "/ram/load/0".to_string(), hardware_identifier: "/ram".to_string(), sensor_type: 5, value: 6.8 },
                            Sensor { name: "Memory Available".to_string(), identifier: "/ram/load/1".to_string(), hardware_identifier: "/ram".to_string(), sensor_type: 5, value: 9.2 },
                            Sensor { name: "Frame Time".to_string(), identifier: "/presentmon/frametime".to_string(), hardware_identifier: "/presentmon".to_string(), sensor_type: 5, value: 16.6 },
                            Sensor { name: "Network Download".to_string(), identifier: "/nic/0/rx".to_string(), hardware_identifier: "/nic/0".to_string(), sensor_type: 5, value: 1024000.0 },
                            Sensor { name: "Network Upload".to_string(), identifier: "/nic/0/tx".to_string(), hardware_identifier: "/nic/0".to_string(), sensor_type: 5, value: 128000.0 },
                        ],
                        present_mon_apps: vec!["Auto".to_string(), "cyberpunk2077.exe".to_string(), "eldenring.exe".to_string()],
                    };
                    
                    let _ = app_handle.emit("hardware-data", &mock_data);
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            });
        }

        #[cfg(target_os = "windows")]
        {
            let write_stream_clone = self.write_stream.clone();
            
            tokio::spawn(async move {
                loop {
                    println!("Trying to connect to Named Pipe: {}", PIPE_NAME);
                    match ClientOptions::new().open(PIPE_NAME) {
                        Ok(mut client) => {
                            println!("Connected to Named Pipe for reading!");
                            
                            let shared_client = Arc::new(Mutex::new(client));
                            {
                                let mut write_guard = write_stream_clone.lock().await;
                                *write_guard = Some(match Arc::try_unwrap(shared_client.clone()) {
                                    Ok(c) => c.into_inner(),
                                    Err(_) => continue,
                                });
                            }
                            
                            let mut client = write_stream_clone.lock().await.take().unwrap();
                            let mut data_state = HardwareMonitorData {
                                last_poll_time: 0,
                                hardwares: Vec::new(),
                                sensors: Vec::new(),
                                present_mon_apps: Vec::new(),
                            };

                            let _ = app_handle.emit("pipe-status", "connected");

                            loop {
                                let mut command_bytes = [0u8; 2];
                                if let Err(e) = client.read_exact(&mut command_bytes).await {
                                    println!("Pipe read error (command): {}", e);
                                    break;
                                }
                                let command = i16::from_le_bytes(command_bytes);

                                let mut size_bytes = [0u8; 4];
                                if let Err(e) = client.read_exact(&mut size_bytes).await {
                                    println!("Pipe read error (size): {}", e);
                                    break;
                                }
                                let size = i32::from_le_bytes(size_bytes);
                                if size < 0 {
                                    println!("Negative size read: {}, closing connection", size);
                                    break;
                                }

                                let mut payload = vec![0u8; size as usize];
                                if let Err(e) = client.read_exact(&mut payload).await {
                                    println!("Pipe read error (payload): {}", e);
                                    break;
                                }

                                match command {
                                    0 => { // Command.Data
                                        if let Ok(parsed) = parse_hardware_data(&payload) {
                                            data_state.last_poll_time = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap_or_default()
                                                .as_millis() as u64;
                                            data_state.hardwares = parsed.hardwares;
                                            data_state.sensors = parsed.sensors;
                                            
                                            let _ = app_handle.emit("hardware-data", &data_state);
                                        }
                                    }
                                    3 => { // Command.PresentMonApps
                                        if let Ok(apps) = parse_present_mon_apps(&payload) {
                                            data_state.present_mon_apps = apps;
                                            let _ = app_handle.emit("hardware-data", &data_state);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Err(e) => {
                            println!("Failed to connect to Named Pipe: {}. Retrying in 1.5s...", e);
                        }
                    }
                    
                    let _ = app_handle.emit("pipe-status", "disconnected");
                    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                }
            });
        }
    }

    pub async fn select_present_mon_app(&self, name: String) -> Result<(), String> {
        #[cfg(not(target_os = "windows"))]
        {
            println!("Mock selected game focus: {}", name);
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            let name_bytes = name.as_bytes();
            let mut packet = Vec::with_capacity(2 + 2 + name_bytes.len());
            packet.extend_from_slice(&2i16.to_le_bytes());
            packet.extend_from_slice(&(name_bytes.len() as i16).to_le_bytes());
            packet.extend_from_slice(name_bytes);
            self.send_raw_packet(packet).await
        }
    }

    pub async fn select_polling_rate(&self, interval_ms: i16) -> Result<(), String> {
        #[cfg(not(target_os = "windows"))]
        {
            println!("Mock selected polling rate: {}ms", interval_ms);
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            let mut packet = Vec::with_capacity(4);
            packet.extend_from_slice(&4i16.to_le_bytes());
            packet.extend_from_slice(&interval_ms.to_le_bytes());
            self.send_raw_packet(packet).await
        }
    }

    pub async fn refresh_present_mon_apps(&self) -> Result<(), String> {
        #[cfg(not(target_os = "windows"))]
        {
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            let mut packet = Vec::with_capacity(2);
            packet.extend_from_slice(&1i16.to_le_bytes());
            self.send_raw_packet(packet).await
        }
    }

    #[cfg(target_os = "windows")]
    async fn send_raw_packet(&self, packet: Vec<u8>) -> Result<(), String> {
        match ClientOptions::new().open(PIPE_NAME) {
            Ok(mut client) => {
                client.write_all(&packet).await
                    .map_err(|e| format!("Failed to write packet to pipe: {}", e))?;
                client.flush().await
                    .map_err(|e| format!("Failed to flush pipe: {}", e))?;
                Ok(())
            }
            Err(e) => Err(format!("Could not open Named Pipe to write: {}", e)),
        }
    }
}

struct ByteCursor<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> ByteCursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn read_i16(&mut self) -> Result<i16, String> {
        if self.offset + 2 > self.data.len() {
            return Err("Unexpected end of payload".to_string());
        }
        let bytes = [self.data[self.offset], self.data[self.offset + 1]];
        self.offset += 2;
        Ok(i16::from_le_bytes(bytes))
    }

    fn read_i32(&mut self) -> Result<i32, String> {
        if self.offset + 4 > self.data.len() {
            return Err("Unexpected end of payload".to_string());
        }
        let bytes = [
            self.data[self.offset],
            self.data[self.offset + 1],
            self.data[self.offset + 2],
            self.data[self.offset + 3],
        ];
        self.offset += 4;
        Ok(i32::from_le_bytes(bytes))
    }

    fn read_f32(&mut self) -> Result<f32, String> {
        if self.offset + 4 > self.data.len() {
            return Err("Unexpected end of payload".to_string());
        }
        let bytes = [
            self.data[self.offset],
            self.data[self.offset + 1],
            self.data[self.offset + 2],
            self.data[self.offset + 3],
        ];
        self.offset += 4;
        Ok(f32::from_le_bytes(bytes))
    }

    fn read_string(&mut self, len: usize) -> Result<String, String> {
        if self.offset + len > self.data.len() {
            return Err("Unexpected end of payload".to_string());
        }
        let slice = &self.data[self.offset..self.offset + len];
        self.offset += len;
        Ok(String::from_utf8_lossy(slice).into_owned())
    }
}

struct ParsedData {
    hardwares: Vec<Hardware>,
    sensors: Vec<Sensor>,
}

fn parse_hardware_data(payload: &[u8]) -> Result<ParsedData, String> {
    let mut cursor = ByteCursor::new(payload);
    let hw_count = cursor.read_i32()?;
    let sensor_count = cursor.read_i32()?;

    let mut hardwares = Vec::with_capacity(hw_count as usize);
    for _ in 0..hw_count {
        let name_len = cursor.read_i16()? as usize;
        let id_len = cursor.read_i16()? as usize;
        let name = cursor.read_string(name_len)?;
        let identifier = cursor.read_string(id_len)?;
        let hw_type = cursor.read_i32()?;
        
        hardwares.push(Hardware {
            name,
            identifier,
            hardware_type: hw_type,
        });
    }

    let mut sensors = Vec::with_capacity(sensor_count as usize);
    for _ in 0..sensor_count {
        let name_len = cursor.read_i16()? as usize;
        let id_len = cursor.read_i16()? as usize;
        let hw_id_len = cursor.read_i16()? as usize;
        let name = cursor.read_string(name_len)?;
        let identifier = cursor.read_string(id_len)?;
        let hardware_identifier = cursor.read_string(hw_id_len)?;
        let sensor_type = cursor.read_i32()?;
        let value = cursor.read_f32()?;

        sensors.push(Sensor {
            name,
            identifier,
            hardware_identifier,
            sensor_type,
            value,
        });
    }

    Ok(ParsedData { hardwares, sensors })
}

fn parse_present_mon_apps(payload: &[u8]) -> Result<Vec<String>, String> {
    let mut cursor = ByteCursor::new(payload);
    let apps_count = cursor.read_i16()?;
    
    let mut apps = Vec::with_capacity(apps_count as usize + 1);
    apps.push("Auto".to_string());
    
    for _ in 0..apps_count {
        let app_bytes = cursor.read_string(NAME_SIZE)?;
        let cleaned = app_bytes.trim_matches('\0').trim().to_string();
        if !cleaned.is_empty() {
            apps.push(cleaned);
        }
    }

    Ok(apps)
}
