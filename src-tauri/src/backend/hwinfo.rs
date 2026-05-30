use serde::{Serialize, Deserialize};

const MEMORY_MAP_FILE_NAME: &str = "Global\\HWiNFO_SENS_SM2";

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Memory::{OpenFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ};

#[cfg(not(target_os = "windows"))]
type HANDLE = isize;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct SensorSharedMem {
    pub dw_signature: u32,
    pub dw_version: u32,
    pub dw_revision: u32,
    pub poll_time: u64,
    pub dw_offset_of_sensor_section: u32,
    pub dw_size_of_sensor_element: u32,
    pub dw_num_sensor_elements: u32,
    pub dw_offset_of_reading_section: u32,
    pub dw_size_of_reading_element: u32,
    pub dw_num_reading_elements: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct SensorElement {
    pub dw_sensor_id: u64,
    pub dw_sensor_inst: u64,
    pub sz_sensor_name_orig: [u8; 128],
    pub sz_sensor_name_user: [u8; 128],
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct SensorReadingElement {
    pub reading_type: u32,
    pub dw_sensor_index: u32,
    pub dw_reading_id: u32,
    pub sz_label_orig: [u8; 128],
    pub sz_label_user: [u8; 128],
    pub sz_unit: [u8; 16],
    pub value: f64,
    pub value_min: f64,
    pub value_max: f64,
    pub value_avg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HwInfoSensor {
    pub sensor_id: u64,
    pub sensor_inst: u64,
    pub name_orig: String,
    pub name_user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HwInfoReading {
    pub reading_type: u32,
    pub sensor_index: u32,
    pub reading_id: u32,
    pub label_orig: String,
    pub label_user: String,
    pub unit: String,
    pub value: f32,
    pub value_min: f32,
    pub value_max: f32,
    pub value_avg: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HwInfoData {
    pub poll_time: u64,
    pub sensors: Vec<HwInfoSensor>,
    pub readings: Vec<HwInfoReading>,
}

pub struct HwInfoReader {
    mapping: Option<WinSharedMemory>,
}

struct WinSharedMemory {
    handle: HANDLE,
    ptr: *const u8,
}

#[cfg(target_os = "windows")]
impl WinSharedMemory {
    fn open(name: &str) -> Option<Self> {
        unsafe {
            let mut wide_name: Vec<u16> = name.encode_utf16().collect();
            wide_name.push(0);

            let handle = OpenFileMappingW(
                FILE_MAP_READ,
                0,
                wide_name.as_ptr(),
            );

            if handle == 0 {
                return None;
            }

            let ptr = MapViewOfFile(
                handle,
                FILE_MAP_READ,
                0,
                0,
                0,
            );

            if ptr.is_null() {
                CloseHandle(handle);
                return None;
            }

            Some(Self {
                handle,
                ptr: ptr as *const u8,
            })
        }
    }
}

#[cfg(not(target_os = "windows"))]
impl WinSharedMemory {
    fn open(_name: &str) -> Option<Self> {
        None
    }
}

#[cfg(target_os = "windows")]
impl Drop for WinSharedMemory {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                UnmapViewOfFile(self.ptr as *const _);
            }
            if self.handle != 0 {
                CloseHandle(self.handle);
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
impl Drop for WinSharedMemory {
    fn drop(&mut self) {}
}

impl HwInfoReader {
    pub fn new() -> Self {
        Self { mapping: None }
    }

    pub fn try_connect(&mut self) -> bool {
        if self.mapping.is_some() {
            return true;
        }
        self.mapping = WinSharedMemory::open(MEMORY_MAP_FILE_NAME);
        self.mapping.is_some()
    }

    pub fn disconnect(&mut self) {
        self.mapping = None;
    }

    pub fn read_data(&mut self) -> Result<HwInfoData, String> {
        #[cfg(not(target_os = "windows"))]
        {
            return Err("HWiNFO reader is strictly supported on Windows platforms.".to_string());
        }

        #[cfg(target_os = "windows")]
        {
            if !self.try_connect() {
                return Err("HWiNFO shared memory is not available. Is HWiNFO running?".to_string());
            }

            let mapping = self.mapping.as_ref().unwrap();
            let ptr = mapping.ptr;

            unsafe {
                let header: SensorSharedMem = std::ptr::read_unaligned(ptr as *const SensorSharedMem);

                if header.dw_signature != 0x4857694E { // "HWiN"
                    return Err("Invalid HWiNFO signature".to_string());
                }

                let mut sensors = Vec::with_capacity(header.dw_num_sensor_elements as usize);
                for i in 0..header.dw_num_sensor_elements {
                    let offset = header.dw_offset_of_sensor_section as usize 
                        + (header.dw_size_of_sensor_element as usize * i as usize);
                    let element_ptr = ptr.add(offset) as *const SensorElement;
                    let element = std::ptr::read_unaligned(element_ptr);

                    sensors.push(HwInfoSensor {
                        sensor_id: element.dw_sensor_id,
                        sensor_inst: element.dw_sensor_inst,
                        name_orig: read_utf8_string(&element.sz_sensor_name_orig),
                        name_user: read_utf8_string(&element.sz_sensor_name_user),
                    });
                }

                let mut readings = Vec::with_capacity(header.dw_num_reading_elements as usize);
                for i in 0..header.dw_num_reading_elements {
                    let offset = header.dw_offset_of_reading_section as usize 
                        + (header.dw_size_of_reading_element as usize * i as usize);
                    let element_ptr = ptr.add(offset) as *const SensorReadingElement;
                    let element = std::ptr::read_unaligned(element_ptr);

                    readings.push(HwInfoReading {
                        reading_type: element.reading_type,
                        sensor_index: element.dw_sensor_index,
                        reading_id: element.dw_reading_id,
                        label_orig: read_utf8_string(&element.sz_label_orig),
                        label_user: read_utf8_string(&element.sz_label_user),
                        unit: read_utf8_string(&element.sz_unit),
                        value: element.value as f32,
                        value_min: element.value_min as f32,
                        value_max: element.value_max as f32,
                        value_avg: element.value_avg as f32,
                    });
                }

                Ok(HwInfoData {
                    poll_time: header.poll_time,
                    sensors,
                    readings,
                })
            }
        }
    }
}

fn read_utf8_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&x| x == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned().trim().to_string()
}
