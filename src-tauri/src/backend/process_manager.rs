use std::process::Command;
use std::sync::Mutex;
use std::path::PathBuf;
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;
use tauri::AppHandle;

static SIDECAR_CHILD: Mutex<Option<CommandChild>> = Mutex::new(None);

const REGISTRY_RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const REG_APP_NAME: &str = "cleanmeter";

#[cfg(target_os = "windows")]
extern "system" {
    fn ShellExecuteW(
        hwnd: isize,
        lp_operation: *const u16,
        lp_file: *const u16,
        lp_parameters: *const u16,
        lp_directory: *const u16,
        n_show_cmd: i32,
    ) -> isize;
}

pub struct ProcessManager;

impl ProcessManager {
    // Launch companion sidecar binary
    pub fn start_sidecar(app_handle: &AppHandle) -> Result<(), String> {
        let mut child_guard = SIDECAR_CHILD.lock().unwrap();
        if child_guard.is_some() {
            return Ok(()); // Already running
        }

        println!("Spawning C# HardwareMonitor sidecar...");
        
        // Under non-windows, we just mock the sidecar spawning as successful!
        #[cfg(not(target_os = "windows"))]
        {
            println!("Mock sidecar spawned successfully!");
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            let sidecar = app_handle.shell().sidecar("HardwareMonitor")
                .map_err(|e| format!("Failed to resolve HardwareMonitor sidecar: {}", e))?;
            
            let (_, child) = sidecar.spawn()
                .map_err(|e| format!("Failed to spawn sidecar process: {}", e))?;
            
            *child_guard = Some(child);
            println!("Sidecar process spawned successfully!");
            Ok(())
        }
    }

    // Stop sidecar process
    pub fn stop_sidecar() {
        let mut child_guard = SIDECAR_CHILD.lock().unwrap();
        if let Some(child) = child_guard.take() {
            println!("Terminating HardwareMonitor sidecar...");
            #[cfg(target_os = "windows")]
            {
                let _ = child.kill();
            }
        }
    }

    // Checking if the GUI process is running as Administrator
    pub fn is_elevated() -> bool {
        #[cfg(not(target_os = "windows"))]
        {
            return false;
        }

        #[cfg(target_os = "windows")]
        {
            let test_path = PathBuf::from("C:\\cleanmeter_elevation.lock");
            match std::fs::File::create(&test_path) {
                Ok(_) => {
                    let _ = std::fs::remove_file(&test_path);
                    true
                }
                Err(_) => false,
            }
        }
    }

    // Elevate current GUI process via ShellExecuteW "runas" fallback
    pub fn elevate_process() {
        #[cfg(target_os = "windows")]
        {
            if let Ok(current_exe) = std::env::current_exe() {
                unsafe {
                    let mut wide_operation: Vec<u16> = "runas".encode_utf16().collect();
                    wide_operation.push(0);

                    let exe_str = current_exe.to_string_lossy().to_string();
                    let mut wide_file: Vec<u16> = exe_str.encode_utf16().collect();
                    wide_file.push(0);

                    let mut wide_params: Vec<u16> = Vec::new();
                    wide_params.push(0);

                    let mut wide_dir: Vec<u16> = Vec::new();
                    wide_dir.push(0);

                    ShellExecuteW(
                        0,
                        wide_operation.as_ptr(),
                        wide_file.as_ptr(),
                        wide_params.as_ptr(),
                        wide_dir.as_ptr(),
                        1, // SW_SHOWNORMAL
                    );
                }
                std::process::exit(0);
            }
        }
    }

    // Startup Autostart Settings under HKCU (HKEY_CURRENT_USER) - No Elevation Required!
    pub fn is_autostart_enabled() -> bool {
        #[cfg(not(target_os = "windows"))]
        {
            return false;
        }

        #[cfg(target_os = "windows")]
        {
            let output = Command::new("reg")
                .args(&["query", REGISTRY_RUN_KEY, "/v", REG_APP_NAME])
                .output();

            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    stdout.contains(REG_APP_NAME)
                }
                Err(_) => false,
            }
        }
    }

    pub fn enable_autostart() -> Result<(), String> {
        #[cfg(not(target_os = "windows"))]
        {
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            let current_exe = std::env::current_exe()
                .map_err(|e| format!("Failed to get current executable path: {}", e))?;
            let exe_path = current_exe.to_string_lossy().to_string();
            let value = format!("\"{}\" --autostart", exe_path);

            let output = Command::new("reg")
                .args(&["add", REGISTRY_RUN_KEY, "/v", REG_APP_NAME, "/t", "REG_SZ", "/d", &value, "/f"])
                .output()
                .map_err(|e| format!("Failed to execute reg add: {}", e))?;

            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Registry write failed: {}", err));
            }

            Ok(())
        }
    }

    pub fn disable_autostart() -> Result<(), String> {
        #[cfg(not(target_os = "windows"))]
        {
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            let output = Command::new("reg")
                .args(&["delete", REGISTRY_RUN_KEY, "/v", REG_APP_NAME, "/f"])
                .output()
                .map_err(|e| format!("Failed to execute reg delete: {}", e))?;

            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                if !err.contains("The system was unable to find the specified registry key or value") {
                    return Err(format!("Registry delete failed: {}", err));
                }
            }

            Ok(())
        }
    }

    // Windows Service controls (Requires Admin elevation)
    pub fn install_service(app_handle: &AppHandle) -> Result<(), String> {
        #[cfg(not(target_os = "windows"))]
        {
            let _ = app_handle;
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            if !Self.is_elevated() {
                return Err("Administrator privileges are required to install Windows Service.".to_string());
            }

            let filename = if cfg!(target_env = "msvc") {
                "HardwareMonitor-x86_64-pc-windows-msvc.exe"
            } else {
                "HardwareMonitor-x86_64-pc-windows-gnu.exe"
            };

            let sidecar_path = app_handle.path().resource_dir()
                .map_err(|e| format!("Resource dir error: {}", e))?
                .join("_up_")
                .join(filename);

            let path_str = sidecar_path.to_string_lossy().to_string();

            let output = Command::new("sc")
                .args(&[
                    "create",
                    "svcleanmeter",
                    "displayname=",
                    "CleanMeter Service",
                    "binPath=",
                    &path_str,
                    "start=",
                    "auto",
                ])
                .output()
                .map_err(|e| format!("Failed to run sc create command: {}", e))?;

            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Service creation failed: {}", err));
            }

            let _ = Command::new("sc").args(&["start", "svcleanmeter"]).output();

            Ok(())
        }
    }

    pub fn uninstall_service() -> Result<(), String> {
        #[cfg(not(target_os = "windows"))]
        {
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            if !Self.is_elevated() {
                return Err("Administrator privileges are required to uninstall Windows Service.".to_string());
            }

            let _ = Command::new("sc").args(&["stop", "svcleanmeter"]).output();

            let output = Command::new("sc")
                .args(&["delete", "svcleanmeter"])
                .output()
                .map_err(|e| format!("Failed to run sc delete: {}", e))?;

            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                if !err.contains("The specified service does not exist") {
                    return Err(format!("Service delete failed: {}", err));
                }
            }

            Ok(())
        }
    }
}
