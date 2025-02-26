use serde::{Deserialize, Serialize};
use std::process::Command;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::Graphics::Gdi::{BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, GetDC, HBITMAP, SRCCOPY};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW, SetCursorPos, mouse_event, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

#[derive(Serialize, Deserialize)]
struct WindowInfo {
    title: String,
    hwnd: isize,
}

#[tokio::main]
pub async fn take_screenshot() -> Result<String, String> {
    #[cfg(windows)]
    {
        let hdc_screen = unsafe { GetDC(None) };
        let hdc_mem = unsafe { CreateCompatibleDC(hdc_screen) };
        let bitmap = unsafe { CreateCompatibleBitmap(hdc_screen, 1920, 1080) }; // Adjust resolution as needed
        unsafe { BitBlt(hdc_mem, 0, 0, 1920, 1080, hdc_screen, 0, 0, SRCCOPY) };
        // Save bitmap to file (simplified; use a library like `image` for real saving)
        Ok("Screenshot captured (Windows)".to_string())
    }
    #[cfg(not(windows))]
    {
        if cfg!(target_os = "linux") && std::env::var("WSL_DISTRO_NAME").is_ok() {
            // WSL bridge to Windows
            let output = Command::new("powershell.exe")
                .arg("-Command")
                .arg("Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('{PRTSC}');")
                .output()
                .map_err(|e| e.to_string())?;
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            // Use scrot or screencapture for Linux/macOS
            Err("Native screenshot not implemented for non-Windows".to_string())
        }
    }
}

#[tokio::main]
pub async fn list_windows() -> Result<Vec<WindowInfo>, String> {
    #[cfg(windows)]
    {
        let mut windows_list = Vec::new();
        unsafe {
            EnumWindows(
                Some(|hwnd: HWND, lparam: isize| -> windows::core::Result<()> {
                    let mut title = [0u16; 512];
                    let len = GetWindowTextW(hwnd, &mut title);
                    if len > 0 {
                        let title_str = String::from_utf16_lossy(&title[..len as usize]);
                        let list = &mut *(lparam as *mut Vec<WindowInfo> as isize);
                        list.push(WindowInfo { title: title_str, hwnd: hwnd.0 });
                    }
                    Ok(())
                }),
                &mut windows_list as *mut Vec<WindowInfo> as isize,
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(windows_list)
    }
    #[cfg(not(windows))]
    {
        Err("Window listing not implemented for non-Windows".to_string())
    }
}

pub fn click_mouse(x: i32, y: i32) -> Result<(), String> {
    #[cfg(windows)]
    {
        unsafe {
            SetCursorPos(x, y).map_err(|e| e.to_string())?;
            mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
            mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        Err("Mouse control not implemented for non-Windows".to_string())
    }
}

pub fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        Command::new("cmd")
            .args(&["/C", "start", url])
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        if cfg!(target_os = "linux") && std::env::var("WSL_DISTRO_NAME").is_ok() {
            Command::new("/mnt/c/Windows/System32/cmd.exe")
                .args(&["/C", "start", url])
                .spawn()
                .map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Browser opening not implemented for non-Windows".to_string())
        }
    }
}
