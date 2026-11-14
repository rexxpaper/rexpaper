use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{PCWSTR, w};

const WM_TRAYICON: u32 = WM_USER + 101;
const ID_TRAY_OPEN: usize = 1001;
const ID_TRAY_STATIC: usize = 1002;
const ID_TRAY_LIVE: usize = 1003;
const ID_TRAY_SETTINGS: usize = 1004;
const ID_TRAY_QUIT: usize = 1005;
const ID_TRAY_PAUSE: usize = 1006;
const ID_TRAY_NEXT: usize = 1007;
const ID_TRAY_FOLDER: usize = 1008;

/// Mirrors the live-wallpaper paused state so the tray label stays in sync.
static TRAY_PAUSED: AtomicBool = AtomicBool::new(false);

/// Updates the Pause/Resume label used the next time the tray menu opens.
pub fn set_tray_paused(paused: bool) {
    TRAY_PAUSED.store(paused, Ordering::SeqCst);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    Open,
    StaticPage,
    LivePage,
    SettingsPage,
    TogglePause,
    NextWallpaper,
    OpenFolder,
    Quit,
}

pub fn setup_tray<F>(callback: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(TrayAction) + Send + Sync + 'static,
{
    let callback = Arc::new(callback);
    std::thread::Builder::new()
        .name("rexpaper-tray".to_string())
        .spawn(move || unsafe {
            let hinstance = GetModuleHandleW(None).unwrap_or_default();
            let class_name = w!("RexPaperTrayClass");

            let wnd_class = WNDCLASSW {
                lpfnWndProc: Some(tray_wnd_proc),
                hInstance: hinstance.into(),
                lpszClassName: class_name,
                ..Default::default()
            };

            let _ = RegisterClassW(&wnd_class);

            // Message-only window (HWND_MESSAGE + WS_EX_TOOLWINDOW): never shows in taskbar or Alt-Tab
            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                class_name,
                w!("RexPaperTrayWindow"),
                WS_POPUP,
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                Some(hinstance.into()),
                None,
            )
            .unwrap_or_default();

            if hwnd.0.is_null() {
                return;
            }

            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: 1,
                uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
                uCallbackMessage: WM_TRAYICON,
                ..Default::default()
            };

            nid.hIcon = load_tray_icon();

            // Set tooltip
            let tip = "RexPaper - Wallpaper Manager";
            for (i, c) in tip.encode_utf16().enumerate() {
                if i < nid.szTip.len() - 1 {
                    nid.szTip[i] = c;
                }
            }

            let _ = Shell_NotifyIconW(NIM_ADD, &nid);

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).into() {
                if msg.message == WM_TRAYICON {
                    let event = msg.lParam.0 as u32;
                    if event == WM_LBUTTONUP || event == WM_LBUTTONDBLCLK {
                        callback(TrayAction::Open);
                    } else if event == WM_RBUTTONUP {
                        let mut pt = POINT::default();
                        let _ = GetCursorPos(&mut pt);
                        let _ = SetForegroundWindow(hwnd);

                        let menu = CreatePopupMenu().unwrap_or_default();
                        if !menu.0.is_null() {
                            let _ = AppendMenuW(menu, MF_STRING, ID_TRAY_OPEN, w!("Open RexPaper"));
                            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
                            let _ = AppendMenuW(
                                menu,
                                MF_STRING,
                                ID_TRAY_STATIC,
                                w!("Static Wallpapers"),
                            );
                            let _ =
                                AppendMenuW(menu, MF_STRING, ID_TRAY_LIVE, w!("Live Wallpapers"));
                            let _ = AppendMenuW(menu, MF_STRING, ID_TRAY_SETTINGS, w!("Settings"));
                            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);

                            let pause_label = if TRAY_PAUSED.load(Ordering::SeqCst) {
                                "Resume Live Wallpaper"
                            } else {
                                "Pause Live Wallpaper"
                            };
                            let pause_wide: Vec<u16> =
                                pause_label.encode_utf16().chain(Some(0)).collect();
                            let _ = AppendMenuW(
                                menu,
                                MF_STRING,
                                ID_TRAY_PAUSE,
                                PCWSTR(pause_wide.as_ptr()),
                            );
                            let _ =
                                AppendMenuW(menu, MF_STRING, ID_TRAY_NEXT, w!("Next Wallpaper"));
                            let _ = AppendMenuW(
                                menu,
                                MF_STRING,
                                ID_TRAY_FOLDER,
                                w!("Open Wallpaper Folder"),
                            );

                            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
                            let _ = AppendMenuW(menu, MF_STRING, ID_TRAY_QUIT, w!("Quit RexPaper"));

                            let cmd = TrackPopupMenu(
                                menu,
                                TPM_RIGHTBUTTON | TPM_BOTTOMALIGN | TPM_RETURNCMD,
                                pt.x,
                                pt.y,
                                Some(0),
                                hwnd,
                                None,
                            );
                            let _ = DestroyMenu(menu);

                            match cmd.0 as usize {
                                ID_TRAY_OPEN => callback(TrayAction::Open),
                                ID_TRAY_STATIC => callback(TrayAction::StaticPage),
                                ID_TRAY_LIVE => callback(TrayAction::LivePage),
                                ID_TRAY_SETTINGS => callback(TrayAction::SettingsPage),
                                ID_TRAY_PAUSE => callback(TrayAction::TogglePause),
                                ID_TRAY_NEXT => callback(TrayAction::NextWallpaper),
                                ID_TRAY_FOLDER => callback(TrayAction::OpenFolder),
                                ID_TRAY_QUIT => callback(TrayAction::Quit),
                                _ => {}
                            }
                        }
                    }
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
        })?;

    Ok(())
}

unsafe extern "system" fn tray_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

static ICON_BYTES: &[u8] = include_bytes!("../../../../assets/icon.ico");

unsafe fn load_tray_icon() -> HICON {
    unsafe {
        let cx = GetSystemMetrics(SM_CXSMICON);
        let cy = GetSystemMetrics(SM_CYSMICON);
        let hinstance = GetModuleHandleW(None).unwrap_or_default();

        // 1. Try loading from embedded byte stream (guaranteed pixel-perfect match)
        if let Some(hicon) = load_hicon_from_ico_bytes(ICON_BYTES, cx, cy) {
            return hicon;
        }

        // 2. Try loading named resource "MAINICON"
        if let Ok(h) = LoadImageW(
            Some(hinstance.into()),
            windows::core::w!("MAINICON"),
            IMAGE_ICON,
            cx,
            cy,
            LR_DEFAULTCOLOR,
        ) && !h.0.is_null()
        {
            return HICON(h.0);
        }

        // 3. Try loading resource ID 1
        if let Ok(h) = LoadImageW(
            Some(hinstance.into()),
            windows::core::PCWSTR(1 as _),
            IMAGE_ICON,
            cx,
            cy,
            LR_DEFAULTCOLOR,
        ) && !h.0.is_null()
        {
            return HICON(h.0);
        }

        // 4. Fallback default
        LoadIconW(None, IDI_APPLICATION).unwrap_or_default()
    }
}

unsafe fn load_hicon_from_ico_bytes(data: &[u8], target_w: i32, target_h: i32) -> Option<HICON> {
    unsafe {
        if data.len() < 6 {
            return None;
        }

        let count = u16::from_le_bytes([data[4], data[5]]) as usize;
        if data.len() < 6 + count * 16 {
            return None;
        }

        let mut best_entry: Option<(usize, usize)> = None; // (offset, size)
        let mut best_diff = i32::MAX;

        for i in 0..count {
            let entry_offset = 6 + i * 16;
            let w = if data[entry_offset] == 0 {
                256
            } else {
                data[entry_offset] as i32
            };
            let dw_bytes = u32::from_le_bytes([
                data[entry_offset + 8],
                data[entry_offset + 9],
                data[entry_offset + 10],
                data[entry_offset + 11],
            ]) as usize;
            let dw_offset = u32::from_le_bytes([
                data[entry_offset + 12],
                data[entry_offset + 13],
                data[entry_offset + 14],
                data[entry_offset + 15],
            ]) as usize;

            if data.len() >= dw_offset + dw_bytes {
                let diff = (w - target_w).abs();
                if diff < best_diff {
                    best_diff = diff;
                    best_entry = Some((dw_offset, dw_bytes));
                }
            }
        }

        if let Some((offset, size)) = best_entry {
            let icon_slice = &data[offset..offset + size];
            if let Ok(hicon) = CreateIconFromResourceEx(
                icon_slice,
                true, // TRUE for icon
                0x00030000,
                target_w,
                target_h,
                LR_DEFAULTCOLOR,
            ) && !hicon.0.is_null()
            {
                return Some(hicon);
            }
        }

        None
    }
}
