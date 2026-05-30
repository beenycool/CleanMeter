use serde::{Serialize, Deserialize};

const MEMORY_MAP_FILE_NAME: &str = "MAHMSharedMemory";
const MAX_STRING_LENGTH: usize = 260;

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Memory::{OpenFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ, MEMORY_MAPPED_VIEW_ADDRESS};

#[cfg(not(target_os = "windows"))]
type HANDLE = isize;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MahmHeader {
    pub dw_signature: u32,
    pub dw_version: u32,
    pub dw_header_size: u32,
    pub dw_num_entries: u32,
    pub dw_entry_size: u32,
    pub last_check: u32,
    pub dw_num_gpu_entries: u32,
    pub dw_gpu_entry_size: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MahmEntry {
    pub sz_src_name: [u8; MAX_STRING_LENGTH],
    pub sz_src_units: [u8; MAX_STRING_LENGTH],
    pub sz_localised_src_name: [u8; MAX_STRING_LENGTH],
    pub sz_localised_src_units: [u8; MAX_STRING_LENGTH],
    pub sz_recommended_format: [u8; MAX_STRING_LENGTH],
    pub data: f32,
    pub min_limit: f32,
    pub max_limit: f32,
    pub dw_flags: u32,
    pub dw_gpu: u32,
    pub dw_src_id: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MahmGpuEntry {
    pub sz_gpu_id: [u8; MAX_STRING_LENGTH],
    pub sz_family: [u8; MAX_STRING_LENGTH],
    pub sz_device: [u8; MAX_STRING_LENGTH],
    pub sz_driver: [u8; MAX_STRING_LENGTH],
    pub sz_bios: [u8; MAX_STRING_LENGTH],
    pub dw_mem_amount: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MahmEntryData {
    pub src_name: String,
    pub src_units: String,
    pub localised_src_name: String,
    pub localised_src_units: String,
    pub recommended_format: String,
    pub data: f32,
    pub min_limit: f32,
    pub max_limit: f32,
    pub flags: u32,
    pub gpu: u32,
    pub src_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MahmGpuEntryData {
    pub gpu_id: String,
    pub family: String,
    pub device: String,
    pub driver: String,
    pub bios: String,
    pub mem_amount: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MahmData {
    pub signature: u32,
    pub version: u32,
    pub last_check: u32,
    pub entries: Vec<MahmEntryData>,
    pub gpu_entries: Vec<MahmGpuEntryData>,
}

pub struct MahmReader {
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

            if ptr.Value.is_null() {
                CloseHandle(handle);
                return None;
            }

            Some(Self {
                handle,
                ptr: ptr.Value as *const u8,
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
                UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS { Value: self.ptr as *mut _ });
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

impl MahmReader {
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

    pub fn read_data(&mut self) -> Result<MahmData, String> {
        #[cfg(not(target_os = "windows"))]
        {
            return Err("MSI Afterburner reader is strictly supported on Windows platforms.".to_string());
        }

        #[cfg(target_os = "windows")]
        {
            if !self.try_connect() {
                return Err("AfterBurner shared memory is not available. Is AfterBurner running?".to_string());
            }

            let mapping = self.mapping.as_ref().unwrap();
            let ptr = mapping.ptr;

            unsafe {
                let header: MahmHeader = std::ptr::read_unaligned(ptr as *const MahmHeader);

                if header.dw_signature != 0x4D48414D && header.dw_signature != 0xDEAD {
                    return Err("Invalid AfterBurner signature".to_string());
                }

                let mut entries = Vec::with_capacity(header.dw_num_entries as usize);
                let mut current_offset = header.dw_header_size as usize;

                for _ in 0..header.dw_num_entries {
                    let entry_ptr = ptr.add(current_offset) as *const MahmEntry;
                    let entry = std::ptr::read_unaligned(entry_ptr);

                    entries.push(MahmEntryData {
                        src_name: read_utf8_string(&entry.sz_src_name),
                        src_units: read_utf8_string(&entry.sz_src_units),
                        localised_src_name: read_utf8_string(&entry.sz_localised_src_name),
                        localised_src_units: read_utf8_string(&entry.sz_localised_src_units),
                        recommended_format: read_utf8_string(&entry.sz_recommended_format),
                        data: entry.data,
                        min_limit: entry.min_limit,
                        max_limit: entry.max_limit,
                        flags: entry.dw_flags,
                        gpu: entry.dw_gpu,
                        src_id: entry.dw_src_id as i32,
                    });

                    current_offset += header.dw_entry_size as usize;
                }

                let mut gpu_entries = Vec::with_capacity(header.dw_num_gpu_entries as usize);
                for _ in 0..header.dw_num_gpu_entries {
                    let gpu_entry_ptr = ptr.add(current_offset) as *const MahmGpuEntry;
                    let gpu_entry = std::ptr::read_unaligned(gpu_entry_ptr);

                    gpu_entries.push(MahmGpuEntryData {
                        gpu_id: read_utf8_string(&gpu_entry.sz_gpu_id),
                        family: read_utf8_string(&gpu_entry.sz_family),
                        device: read_utf8_string(&gpu_entry.sz_device),
                        driver: read_utf8_string(&gpu_entry.sz_driver),
                        bios: read_utf8_string(&gpu_entry.sz_bios),
                        mem_amount: gpu_entry.dw_mem_amount,
                    });

                    current_offset += header.dw_gpu_entry_size as usize;
                }

                Ok(MahmData {
                    signature: header.dw_signature,
                    version: header.dw_version,
                    last_check: header.last_check,
                    entries,
                    gpu_entries,
                })
            }
        }
    }
}

fn read_utf8_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&x| x == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned().trim().to_string()
}
