use std::path::Path;
use std::process::Command;
use std::os::windows::process::CommandExt;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::{RedrawWindow, UpdateWindow, RDW_INVALIDATE, RDW_ERASE, RDW_ALLCHILDREN};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::core::BOOL;

static MPV_PROCESS_ID: std::sync::Mutex<Option<u32>> = std::sync::Mutex::new(None);
static IS_WALLPAPER_ACTIVE: AtomicBool = AtomicBool::new(false);
static CURRENT_LIVE_PATH: std::sync::Mutex<Option<std::path::PathBuf>> = std::sync::Mutex::new(None);
static IS_PAUSED_FOR_FULLSCREEN: AtomicBool = AtomicBool::new(false);
static DESKTOP_HOST_WINDOW: std::sync::Mutex<Option<usize>> = std::sync::Mutex::new(None);
static HOST_CLASS_REGISTERED: AtomicBool = AtomicBool::new(false);
const CREATE_NO_WINDOW: u32 = 0x08000000;
const WS_EX_NOREDIRECTIONBITMAP: isize = 0x00200000;

pub fn apply_live_wallpaper(video_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !video_path.exists() {
        return Err(format!("Video file not found: {}", video_path.display()).into());
    }

    stop_live_wallpaper()?;

    let wallpaper_hwnd = resolve_live_wallpaper_hwnd()?;
    let hwnd = wallpaper_hwnd.0 as usize;
    let mpv_exe = find_mpv_executable()?;

    // Ensure the desktop wallpaper canvas is visible and active
    unsafe {
        let _ = ShowWindow(wallpaper_hwnd, SW_SHOW);
        let _ = UpdateWindow(wallpaper_hwnd);
    }

    let mut path_str = video_path.to_string_lossy().to_string();
    if path_str.starts_with(r"\\?\") {
        path_str = path_str[4..].to_string();
    }

    let mute_opt = if crate::settings::Settings::load().mute_live_wallpapers {
        "--mute=yes"
    } else {
        "--mute=no"
    };

    let log_path = crate::settings::get_app_dir().join("mpv_live.log");

    let mut cmd = Command::new(mpv_exe);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.args([
        "--player-operation-mode=cplayer",
        "--no-terminal",
        &format!("--wid={}", hwnd),
        "--force-window=yes",
        "--loop-file=inf",
        mute_opt,
        "--no-border",
        "--no-window-dragging",
        "--no-input-default-bindings",
        "--input-vo-keyboard=no",
        "--vo=gpu",
        "--hwdec=auto-safe",
        "--panscan=1.0",
        "--keep-open=yes",
        &format!("--log-file={}", log_path.to_string_lossy()),
        "--priority=belownormal",
        "--vd-lavc-threads=2",
        "--demuxer-max-bytes=8M",
        "--demuxer-max-back-bytes=2M",
        "--cache=no",
        "--deband=no",
        "--dither-depth=no",
        &path_str,
    ]);

    let child = cmd.spawn()?;

    *MPV_PROCESS_ID.lock().unwrap() = Some(child.id());
    *CURRENT_LIVE_PATH.lock().unwrap() = Some(video_path.to_path_buf());
    IS_WALLPAPER_ACTIVE.store(true, Ordering::SeqCst);
    IS_PAUSED_FOR_FULLSCREEN.store(false, Ordering::SeqCst);

    Ok(())
}

pub fn stop_live_wallpaper() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(pid) = *MPV_PROCESS_ID.lock().unwrap() {
        let mut cmd = Command::new("taskkill");
        cmd.creation_flags(CREATE_NO_WINDOW);
        let _ = cmd.args(["/PID", &pid.to_string(), "/F", "/T"]).status();
    }
    *MPV_PROCESS_ID.lock().unwrap() = None;
    *CURRENT_LIVE_PATH.lock().unwrap() = None;
    IS_WALLPAPER_ACTIVE.store(false, Ordering::SeqCst);
    IS_PAUSED_FOR_FULLSCREEN.store(false, Ordering::SeqCst);

    // Redraw desktop wallpaper / icons
    unsafe {
        if let Ok(progman) = FindWindowW(windows::core::w!("Progman"), None) {
            if !progman.0.is_null() {
                let _ = RedrawWindow(Some(progman), None, None, RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN);
            }
        }
    }

    // Destroy the desktop host window used for the Windows 11 raised-desktop layout
    if let Some(host) = *DESKTOP_HOST_WINDOW.lock().unwrap() {
        let host_hwnd = HWND(host as *mut std::ffi::c_void);
        if !host_hwnd.0.is_null() {
            unsafe {
                let _ = DestroyWindow(host_hwnd);
            }
        }
    }
    *DESKTOP_HOST_WINDOW.lock().unwrap() = None;

    Ok(())
}

pub fn is_foreground_fullscreen() -> bool {
    unsafe {
        let fg = GetForegroundWindow();
        if fg.0.is_null() || !IsWindowVisible(fg).as_bool() || IsIconic(fg).as_bool() {
            return false;
        }

        // Ignore our own application window
        let mut fg_pid: u32 = 0;
        let _ = GetWindowThreadProcessId(fg, Some(&mut fg_pid));
        if fg_pid == std::process::id() || fg_pid == 0 {
            return false;
        }

        let mut class_name = [0u16; 256];
        let len = GetClassNameW(fg, &mut class_name);
        if len > 0 {
            let name = String::from_utf16_lossy(&class_name[..len as usize]);
            if name == "Progman"
                || name == "WorkerW"
                || name == "Shell_TrayWnd"
                || name == "Shell_SecondaryTrayWnd"
                || name == "Windows.UI.Core.CoreWindow"
                || name == "ApplicationFrameWindow"
                || name == "DV2ControlHost"
                || name == "TaskListThumbnailWnd"
                || name == "TopLevelWindowForOverflowXamlIsland" {
                return false;
            }
        }

        let mut rect = RECT::default();
        if GetWindowRect(fg, &mut rect).is_err() {
            return false;
        }

        // Check if the window has standard titlebar/caption (maximized standard window)
        let style = GetWindowLongW(fg, GWL_STYLE) as u32;
        if (style & WS_CAPTION.0) == WS_CAPTION.0 {
            return false;
        }

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);

        rect.left <= 0 && rect.top <= 0 && rect.right >= screen_w && rect.bottom >= screen_h
    }
}

pub fn start_fullscreen_monitor(settings: std::sync::Arc<std::sync::Mutex<crate::settings::Settings>>) {
    std::thread::Builder::new()
        .name("rexpaper-fullscreen-monitor".to_string())
        .spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(1500));

                let pause_enabled = settings
                    .lock()
                    .map(|s| s.pause_on_fullscreen)
                    .unwrap_or(true);

                if !pause_enabled {
                    continue;
                }

                let is_fullscreen = is_foreground_fullscreen();

                if is_fullscreen {
                    if is_live_wallpaper_active() && !IS_PAUSED_FOR_FULLSCREEN.load(Ordering::SeqCst) {
                        IS_PAUSED_FOR_FULLSCREEN.store(true, Ordering::SeqCst);
                        let _ = pause_for_fullscreen();
                    }
                } else {
                    if IS_PAUSED_FOR_FULLSCREEN.load(Ordering::SeqCst) {
                        IS_PAUSED_FOR_FULLSCREEN.store(false, Ordering::SeqCst);
                        let saved_path = CURRENT_LIVE_PATH.lock().unwrap().clone();
                        if let Some(path) = saved_path {
                            let _ = resume_after_fullscreen(&path);
                        }
                    }
                }
            }
        })
        .ok();
}

pub fn pause_for_fullscreen() -> Result<(), Box<dyn std::error::Error>> {
    if is_live_wallpaper_active() {
        if let Some(pid) = *MPV_PROCESS_ID.lock().unwrap() {
            let mut cmd = Command::new("taskkill");
            cmd.creation_flags(CREATE_NO_WINDOW);
            let _ = cmd.args(["/PID", &pid.to_string(), "/F"]).status();
        }
        *MPV_PROCESS_ID.lock().unwrap() = None;
        IS_WALLPAPER_ACTIVE.store(false, Ordering::SeqCst);
    }
    Ok(())
}

pub fn resume_after_fullscreen(video_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !is_live_wallpaper_active() && MPV_PROCESS_ID.lock().unwrap().is_none() {
        apply_live_wallpaper(video_path)?;
    }
    Ok(())
}

pub fn is_live_wallpaper_active() -> bool {
    IS_WALLPAPER_ACTIVE.load(Ordering::SeqCst)
}

struct WorkerWSearch {
    found_shelldll: bool,
    target_workerw: HWND,
}

/// Retrieves the Progman (or shell) desktop root window.
fn get_progman() -> HWND {
    unsafe {
        let shell_wnd = GetShellWindow();
        if !shell_wnd.0.is_null() {
            shell_wnd
        } else {
            FindWindowW(windows::core::w!("Progman"), None).unwrap_or_default()
        }
    }
}

/// Sends the undocumented 0x052C message to Progman to spawn/refresh the wallpaper layer.
fn spawn_workerw_layers(progman: HWND) {
    unsafe {
        let mut res: usize = 0;
        let _ = SendMessageTimeoutW(
            progman,
            0x052C,
            WPARAM(0x0000000D),
            LPARAM(0x00000001),
            SMTO_NORMAL,
            1000,
            Some(&mut res),
        );
        let _ = SendMessageTimeoutW(
            progman,
            0x052C,
            WPARAM(0x0000000D),
            LPARAM(0x00000000),
            SMTO_NORMAL,
            1000,
            Some(&mut res),
        );
    }
}

/// Detects the Windows 11 "raised desktop" composition (builds 26100+ / 24H2) where
/// Progman carries WS_EX_NOREDIRECTIONBITMAP and the shell views are layered children.
fn is_raised_desktop(progman: HWND) -> bool {
    unsafe {
        let ex_style = GetWindowLongPtrW(progman, windows::Win32::UI::WindowsAndMessaging::GWL_EXSTYLE);
        let err = GetLastError();
        if ex_style == 0 && err.0 != 0 {
            eprintln!("[RexPaper] GetWindowLongPtrW failed: {:?}, assuming classic desktop", err);
            return false;
        }
        (ex_style & WS_EX_NOREDIRECTIONBITMAP) != 0
    }
}

unsafe extern "system" fn host_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

unsafe extern "system" fn enum_child_workerw_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let mut class_name = [0u16; 256];
        let len = GetClassNameW(hwnd, &mut class_name);
        if len > 0 {
            let name = String::from_utf16_lossy(&class_name[..len as usize]);
            if name == "WorkerW" {
                let ptr = lparam.0 as *mut HWND;
                *ptr = hwnd;
                return BOOL(0);
            }
        }
        BOOL(1)
    }
}

/// Creates (or reuses) a layered, fully opaque host window parented to Progman and
/// z-ordered between the desktop icons (SHELLDLL_DefView) and the system wallpaper
/// layer. mpv renders into this window via `--wid` for the raised-desktop layout.
fn ensure_host_window(progman: HWND) -> Result<HWND, Box<dyn std::error::Error>> {
    // Hold the lock for the entire check-and-create to prevent races
    let mut guard = DESKTOP_HOST_WINDOW.lock().unwrap();

    // Check if we already have a valid host window
    if let Some(existing) = *guard {
        let existing_hwnd = HWND(existing as *mut std::ffi::c_void);
        if !existing_hwnd.0.is_null() && unsafe { IsWindow(Some(existing_hwnd)).as_bool() } {
            return Ok(existing_hwnd);
        }
    }

    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap_or_default();
        let class_name = windows::core::w!("RexPaperLiveHost");

        if !HOST_CLASS_REGISTERED.load(Ordering::SeqCst) {
            let wnd_class = WNDCLASSW {
                lpfnWndProc: Some(host_wnd_proc),
                hInstance: hinstance.into(),
                lpszClassName: class_name,
                ..Default::default()
            };
            if RegisterClassW(&wnd_class) == 0 {
                let err = GetLastError();
                if err.0 != 1410 { // ERROR_CLASS_ALREADY_EXISTS
                    eprintln!("[RexPaper] RegisterClassW failed: {:?}", err);
                }
            }
            HOST_CLASS_REGISTERED.store(true, Ordering::SeqCst);
        }

        let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class_name,
            windows::core::w!("RexPaperLiveWindow"),
            WS_POPUP,
            x,
            y,
            width,
            height,
            Some(progman),
            None,
            Some(hinstance.into()),
            None,
        );

        let hwnd = match hwnd {
            Ok(h) if !h.0.is_null() => h,
            Ok(_) => return Err("CreateWindowExW returned null handle".into()),
            Err(e) => return Err(format!("CreateWindowExW failed: {}", e).into()),
        };

        if let Err(e) = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA) {
            eprintln!("[RexPaper] SetLayeredWindowAttributes failed: {}", e);
        }

        if let Err(e) = SetParent(hwnd, Some(progman)) {
            eprintln!("[RexPaper] SetParent failed: {}", e);
        }

        // Place the host layer below the desktop icons (z-order after SHELLDLL_DefView)
        if let Ok(defview) =
            FindWindowExW(Some(progman), None, windows::core::w!("SHELLDLL_DefView"), None)
        {
            if !defview.0.is_null() {
                if let Err(e) = SetWindowPos(
                    hwnd,
                    Some(defview),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                ) {
                    eprintln!("[RexPaper] SetWindowPos (host below DefView) failed: {}", e);
                }
            }
        }

        // Push the system wallpaper WorkerW layer behind the host window
        let mut workerw = HWND(std::ptr::null_mut());
        let _ = EnumChildWindows(
            Some(progman),
            Some(enum_child_workerw_proc),
            LPARAM(&mut workerw as *mut HWND as isize),
        );
        if !workerw.0.is_null() {
            if let Err(e) = SetWindowPos(
                workerw,
                Some(hwnd),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            ) {
                eprintln!("[RexPaper] SetWindowPos (WorkerW behind host) failed: {}", e);
            }
        }

        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);

        *guard = Some(hwnd.0 as usize);
        Ok(hwnd)
    }
}

/// Resolves the desktop canvas HWND that mpv should render into.
///
/// Windows 11 24H2+ ("raised desktop") gets a dedicated layered host window parented
/// to Progman and layered between the desktop icons and the static wallpaper layer.
/// Classic layouts (Windows 10 / earlier Windows 11) keep the top-level
/// WorkerW-behind-SHELLDLL_DefView trick.
fn resolve_live_wallpaper_hwnd() -> Result<HWND, Box<dyn std::error::Error>> {
    let progman = get_progman();
    if progman.0.is_null() {
        return Err("Progman desktop window not found".into());
    }

    if is_raised_desktop(progman) {
        spawn_workerw_layers(progman);
        match ensure_host_window(progman) {
            Ok(host) if !host.0.is_null() => return Ok(host),
            Ok(_) => eprintln!("[RexPaper] ensure_host_window returned null handle"),
            Err(e) => eprintln!("[RexPaper] ensure_host_window failed: {}", e),
        }
    } else {
        spawn_workerw_layers(progman);
    }

    find_workerw_classic(progman)
}

/// Finds the classic top-level WorkerW desktop background canvas window (Windows 10
/// and non-raised Windows 11 layouts).
fn find_workerw_classic(progman: HWND) -> Result<HWND, Box<dyn std::error::Error>> {
    unsafe {
        // Pass 1: Find WorkerW immediately behind the window hosting SHELLDLL_DefView
        let mut search = WorkerWSearch {
            found_shelldll: false,
            target_workerw: HWND(std::ptr::null_mut()),
        };
        let _ = EnumWindows(
            Some(enum_windows_proc),
            LPARAM(&mut search as *mut WorkerWSearch as isize),
        );

        if !search.target_workerw.0.is_null() {
            return Ok(search.target_workerw);
        }

        // Pass 2: Fallback to any WorkerW without SHELLDLL_DefView
        let mut fallback_workerw = HWND(std::ptr::null_mut());
        let _ = EnumWindows(
            Some(enum_workerw_fallback_proc),
            LPARAM(&mut fallback_workerw as *mut _ as isize),
        );
        if !fallback_workerw.0.is_null() {
            return Ok(fallback_workerw);
        }

        // Pass 3: Direct Progman or desktop window
        if !progman.0.is_null() {
            return Ok(progman);
        }

        let desktop = GetDesktopWindow();
        if !desktop.0.is_null() {
            return Ok(desktop);
        }

        Err("Desktop window handle not found".into())
    }
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let search = &mut *(lparam.0 as *mut WorkerWSearch);

        let shell_dll = FindWindowExW(Some(hwnd), None, windows::core::w!("SHELLDLL_DefView"), None);
        if let Ok(shell_dll) = shell_dll {
            if !shell_dll.0.is_null() {
                // Check if sibling WorkerW can be queried directly behind hwnd
                if let Ok(workerw) = FindWindowExW(None, Some(hwnd), windows::core::w!("WorkerW"), None) {
                    if !workerw.0.is_null() {
                        search.target_workerw = workerw;
                        return BOOL(0);
                    }
                }
                search.found_shelldll = true;
                return BOOL(1);
            }
        }

        if search.found_shelldll {
            let mut class_name = [0u16; 256];
            let len = GetClassNameW(hwnd, &mut class_name);
            if len > 0 {
                let name = String::from_utf16_lossy(&class_name[..len as usize]);
                if name == "WorkerW" {
                    search.target_workerw = hwnd;
                    return BOOL(0); // Found target WorkerW!
                }
            }
        }

        BOOL(1)
    }
}

unsafe extern "system" fn enum_workerw_fallback_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let mut class_name = [0u16; 256];
        let len = GetClassNameW(hwnd, &mut class_name);
        if len > 0 {
            let name = String::from_utf16_lossy(&class_name[..len as usize]);
            if name == "WorkerW" {
                let shell = FindWindowExW(Some(hwnd), None, windows::core::w!("SHELLDLL_DefView"), None);
                if shell.is_err() || shell.unwrap().0.is_null() {
                    let ptr = lparam.0 as *mut HWND;
                    *ptr = hwnd;
                    return BOOL(0);
                }
            }
        }
        BOOL(1)
    }
}

fn find_mpv_executable() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let current_exe = std::env::current_exe()?;
    let exe_dir = current_exe.parent().ok_or("No parent directory")?;

    // 1. Check alongside the executable (target/release/mpv.exe or install dir)
    let bundled_mpv = exe_dir.join("mpv.exe");
    if bundled_mpv.exists() {
        return Ok(bundled_mpv);
    }

    // 2. Check mpv/ subfolder (e.g. exe_dir/mpv/mpv.exe)
    let subfolder_mpv = exe_dir.join("mpv").join("mpv.exe");
    if subfolder_mpv.exists() {
        return Ok(subfolder_mpv);
    }

    // 3. Check workspace root ./mpv/mpv.exe (for dev / cargo run)
    let workspace_mpv = std::path::Path::new("mpv").join("mpv.exe");
    if workspace_mpv.exists() {
        if let Ok(abs) = workspace_mpv.canonicalize() {
            return Ok(abs);
        }
        return Ok(workspace_mpv);
    }

    // 4. Check system PATH (skip the console-subsystem mpv.com wrapper that %PATHEXT%
    //    would normally resolve in favor of mpv.exe)
    // First try which::which for mpv.exe explicitly
    if let Ok(path) = which::which("mpv.exe") {
        return Ok(path);
    }
    // Fallback: which::which("mpv") may return mpv.com due to PATHEXT ordering
    if let Ok(path) = which::which("mpv") {
        if path.extension().map(|e| e.eq_ignore_ascii_case("exe")).unwrap_or(false) {
            return Ok(path);
        }
        // Check for mpv.exe sibling in the same directory
        let sibling_exe = path.with_extension("exe");
        if sibling_exe.exists() {
            return Ok(sibling_exe);
        }
        // Also check parent directory for mpv.exe (some installations)
        if let Some(parent) = path.parent() {
            let parent_exe = parent.join("mpv.exe");
            if parent_exe.exists() {
                return Ok(parent_exe);
            }
        }
    }
    // 5. Explicitly scan PATH directories for mpv.exe (bypasses PATHEXT issues)
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(';') {
            let candidate = std::path::Path::new(dir).join("mpv.exe");
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    Err("mpv.exe not found. Please ensure mpv.exe is in the application folder or in PATH.".into())
}
