use std::path::Path;

pub fn apply_static_wallpaper(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::SystemParametersInfoW;
        use windows::Win32::UI::WindowsAndMessaging::*;
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        eprintln!("[RexPaper] apply_static_wallpaper called for: {}", path.display());
        
        // Terminate any running live wallpaper process and hide the WorkerW canvas
        eprintln!("[RexPaper] Calling stop_live_wallpaper()...");
        let _ = crate::platform::windows::stop_live_wallpaper();
        eprintln!("[RexPaper] stop_live_wallpaper() returned");

        // Brief pause to ensure live wallpaper cleanup (host window destroy, WorkerW restore) is complete
        std::thread::sleep(std::time::Duration::from_millis(200));

        // SystemParametersInfoW with SPI_SETDESKWALLPAPER only supports BMP (and sometimes JPG).
        // Convert unsupported formats (PNG, WEBP, AVIF, etc.) to a temporary BMP file.
        let bmp_path = ensure_bmp_wallpaper(path)?;

        // CRITICAL: SystemParametersInfoW requires an ABSOLUTE path
        let absolute_path = std::fs::canonicalize(&bmp_path)
            .unwrap_or_else(|_| bmp_path.clone());
        
        eprintln!("[RexPaper] Applying static wallpaper (absolute): {}", absolute_path.display());
        eprintln!("[RexPaper] File exists: {}", absolute_path.exists());
        
        let metadata = std::fs::metadata(&absolute_path);
        eprintln!("[RexPaper] File size: {:?} bytes", metadata.map(|m| m.len()));

        let wide_path: Vec<u16> = OsStr::new(&absolute_path).encode_wide().chain(Some(0)).collect();
        
        unsafe {
            // Single standard broadcast via SPI itself (SPIF_SENDCHANGE), exactly once.
            // A separate manual WM_SETTINGCHANGE broadcast to HWND_BROADCAST is what
            // triggered the blank File Explorer window when combined with SPI's own
            // broadcast - do NOT add another one here.
            let result = SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                Some(wide_path.as_ptr() as *mut _),
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            );
            if let Err(e) = result {
                // Surface the failure: the command layer records the wallpaper as
                // active on Ok, so returning Ok here would leave settings claiming
                // a wallpaper that Windows never actually applied.
                let err = windows::Win32::Foundation::GetLastError();
                eprintln!(
                    "[RexPaper] SystemParametersInfoW failed for {}: {:?} (GetLastError: {:?})",
                    absolute_path.display(),
                    e,
                    err
                );
                return Err(format!(
                    "Windows could not apply {} as the desktop wallpaper.",
                    path.display()
                )
                .into());
            }
            eprintln!("[RexPaper] SystemParametersInfoW succeeded (SPIF_SENDCHANGE)");
        }

        // Reclaim temp BMPs produced by earlier conversions. Only reached after a
        // successful apply, so the currently registered wallpaper is never deleted.
        cleanup_stale_temp_bmps(&bmp_path);
    }
    Ok(())
}

/// Removes leftover `rexpaper_wallpaper_*.bmp` files created by earlier
/// conversions, keeping `keep` (the file just registered as the desktop
/// wallpaper). Each PNG/WEBP/AVIF apply writes a multi-megabyte BMP to %TEMP%;
/// without this they accumulate indefinitely.
#[cfg(target_os = "windows")]
fn cleanup_stale_temp_bmps(keep: &Path) {
    let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) else {
        return;
    };
    for path in entries.flatten().map(|entry| entry.path()) {
        if path == keep {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with("rexpaper_wallpaper_") && name.ends_with(".bmp") {
            let _ = std::fs::remove_file(&path);
        }
    }
}

/// Ensures the wallpaper file is in a format supported by SystemParametersInfoW (BMP/JPG).
/// If the file is PNG, WEBP, AVIF, etc., converts it to a temporary BMP file.
fn ensure_bmp_wallpaper(path: &Path) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    
    // Already supported format
    if matches!(ext.as_str(), "bmp" | "jpg" | "jpeg") {
        eprintln!("[RexPaper] Using original file (supported format): {}", path.display());
        return Ok(path.to_path_buf());
    }

    // Convert to temporary BMP file
    let temp_dir = std::env::temp_dir();
    let temp_name = format!("rexpaper_wallpaper_{}.bmp", uuid::Uuid::new_v4().simple());
    let temp_path = temp_dir.join(temp_name);

    eprintln!("[RexPaper] Converting {} to BMP: {}", path.display(), temp_path.display());
    
    let img = image::open(path)?;
    let rgb_img = img.to_rgb8();
    rgb_img.save(&temp_path)?;
    
    if !temp_path.exists() {
        return Err("Failed to create temporary BMP file".into());
    }
    
    let size = std::fs::metadata(&temp_path)?.len();
    eprintln!("[RexPaper] Created BMP ({} bytes): {}", size, temp_path.display());
    
    Ok(temp_path)
}

pub fn scan_and_load_static(root: &Path, state: crate::SharedState) -> Result<(), Box<dyn std::error::Error>> {
    crate::scanner::scan_static(root, state)?;
    Ok(())
}