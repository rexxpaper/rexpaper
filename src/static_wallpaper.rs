use std::path::Path;

pub fn apply_static_wallpaper(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::SystemParametersInfoW;
        use windows::Win32::UI::WindowsAndMessaging::*;
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        // Terminate any running live wallpaper process and hide the WorkerW canvas
        let _ = crate::platform::windows::stop_live_wallpaper();

        // Brief pause to ensure live wallpaper cleanup (host window destroy, WorkerW restore) is complete
        std::thread::sleep(std::time::Duration::from_millis(200));

        // SystemParametersInfoW with SPI_SETDESKWALLPAPER only supports BMP (and sometimes JPG).
        // Convert unsupported formats (PNG, WEBP, AVIF, etc.) to a temporary BMP file.
        let bmp_path = ensure_bmp_wallpaper(path)?;

        let wide_path: Vec<u16> = OsStr::new(&bmp_path).encode_wide().chain(Some(0)).collect();
        
        unsafe {
            let result = SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                Some(wide_path.as_ptr() as *mut _),
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            );
            if result.is_err() {
                eprintln!("[RexPaper] SystemParametersInfoW failed: {:?}", result);
            }
        }
    }
    Ok(())
}

/// Ensures the wallpaper file is in a format supported by SystemParametersInfoW (BMP/JPG).
/// If the file is PNG, WEBP, AVIF, etc., converts it to a temporary BMP file.
fn ensure_bmp_wallpaper(path: &Path) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    
    // Already supported format
    if matches!(ext.as_str(), "bmp" | "jpg" | "jpeg") {
        return Ok(path.to_path_buf());
    }

    // Convert to temporary BMP file
    let temp_dir = std::env::temp_dir();
    let temp_name = format!("rexpaper_wallpaper_{}.bmp", uuid::Uuid::new_v4().simple());
    let temp_path = temp_dir.join(temp_name);

    let img = image::open(path)?;
    let rgb_img = img.to_rgb8();
    rgb_img.save(&temp_path)?;
    
    eprintln!("[RexPaper] Converted wallpaper to BMP: {}", temp_path.display());
    Ok(temp_path)
}

pub fn scan_and_load_static(root: &Path, state: crate::SharedState) -> Result<(), Box<dyn std::error::Error>> {
    crate::scanner::scan_static(root, state)?;
    Ok(())
}