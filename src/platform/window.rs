pub(crate) const WINDOW_TITLE: &str = "桌面日历待办";

#[cfg(target_os = "windows")]
mod imp {
    use std::sync::atomic::{AtomicIsize, AtomicU32, Ordering};

    const GWL_EXSTYLE: i32 = -20;
    const WS_EX_LAYERED: isize = 0x0008_0000;
    const LWA_ALPHA: u32 = 0x2;
    const NO_OPACITY: u32 = u32::MAX;

    struct FindCtx {
        pid: u32,
        hwnd: Option<isize>,
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn EnumWindows(
            lpEnumFunc: unsafe extern "system" fn(isize, isize) -> i32,
            lparam: isize,
        ) -> i32;
        fn GetWindowThreadProcessId(hwnd: isize, lpdwProcessId: *mut u32) -> u32;
        fn IsWindow(hwnd: isize) -> i32;
        fn IsWindowVisible(hwnd: isize) -> i32;
        fn GetWindowTextLengthW(hwnd: isize) -> i32;
        fn GetWindowTextW(hwnd: isize, lpString: *mut u16, nMaxCount: i32) -> i32;
        fn GetWindowLongPtrW(hwnd: isize, nIndex: i32) -> isize;
        fn SetWindowLongPtrW(hwnd: isize, nIndex: i32, dwNewLong: isize) -> isize;
        fn SetLayeredWindowAttributes(
            hwnd: isize,
            crKey: u32,
            bAlpha: u8,
            dwFlags: u32,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcessId() -> u32;
    }

    unsafe extern "system" fn enum_callback(hwnd: isize, lparam: isize) -> i32 {
        let ctx = unsafe { &mut *(lparam as *mut FindCtx) };
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        if pid != ctx.pid || unsafe { IsWindowVisible(hwnd) } == 0 {
            return 1;
        }
        let len = unsafe { GetWindowTextLengthW(hwnd) };
        if len <= 0 {
            return 1;
        }
        let mut buf = vec![0u16; len as usize + 1];
        unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), len + 1) };
        let title = String::from_utf16_lossy(&buf[..len as usize]);
        if title == super::WINDOW_TITLE {
            ctx.hwnd = Some(hwnd);
            0
        } else {
            1
        }
    }

    fn find_main_window() -> Option<isize> {
        let mut ctx = FindCtx {
            pid: unsafe { GetCurrentProcessId() },
            hwnd: None,
        };
        unsafe {
            EnumWindows(enum_callback, &mut ctx as *mut FindCtx as isize);
        }
        ctx.hwnd
    }

    /// 缓存主窗口句柄，避免每帧枚举全部顶层窗口。
    fn main_window() -> Option<isize> {
        static CACHED: AtomicIsize = AtomicIsize::new(0);
        let cached = CACHED.load(Ordering::Relaxed);
        if cached != 0 && unsafe { IsWindow(cached) } != 0 {
            return Some(cached);
        }
        let hwnd = find_main_window()?;
        CACHED.store(hwnd, Ordering::Relaxed);
        Some(hwnd)
    }

    /// 幂等地应用窗口透明度。
    ///
    /// winit 在置顶、显示等状态切换时会用缓存的扩展样式覆写 `GWL_EXSTYLE`，
    /// 顺带抹掉外部设置的 `WS_EX_LAYERED`。因此这里每帧都校验该位，
    /// 一旦发现被覆写就重设样式并重申 alpha，保证透明度始终生效。
    pub(crate) fn set_window_opacity(opacity: f32) -> bool {
        static APPLIED: AtomicU32 = AtomicU32::new(NO_OPACITY);
        let Some(hwnd) = main_window() else {
            return false;
        };
        let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
        let want_layered = opacity < 1.0;
        let style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
        let has_layered = (style & WS_EX_LAYERED) != 0;
        let mut needs_alpha = APPLIED.load(Ordering::Relaxed) != opacity.to_bits();

        if want_layered && !has_layered {
            unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_LAYERED) };
            needs_alpha = true;
        } else if !want_layered && has_layered {
            unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style & !WS_EX_LAYERED) };
        }

        if want_layered && needs_alpha {
            unsafe { SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA) };
        }
        APPLIED.store(opacity.to_bits(), Ordering::Relaxed);
        true
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSApplication;
    use std::sync::atomic::{AtomicU32, Ordering};

    const NO_OPACITY: u32 = u32::MAX;

    pub(crate) fn set_window_opacity(opacity: f32) -> bool {
        static APPLIED: AtomicU32 = AtomicU32::new(NO_OPACITY);
        let alpha = opacity.clamp(0.0, 1.0);
        if APPLIED.load(Ordering::Relaxed) == alpha.to_bits() {
            return true;
        }
        let Some(mtm) = MainThreadMarker::new() else {
            return false;
        };
        let app = NSApplication::sharedApplication(mtm);
        let Some(window) = app.mainWindow().or_else(|| app.keyWindow()) else {
            return false;
        };
        window.setAlphaValue(alpha as f64);
        APPLIED.store(alpha.to_bits(), Ordering::Relaxed);
        true
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _, PropMode};
    use x11rb::rust_connection::RustConnection;
    use x11rb::wrapper::ConnectionExt as _;

    fn intern_atom(conn: &RustConnection, name: &[u8]) -> Option<u32> {
        let cookie = conn.intern_atom(false, name).ok()?;
        let reply = cookie.reply().ok()?;
        Some(reply.atom)
    }

    fn property_u32(
        conn: &RustConnection,
        window: u32,
        property: u32,
        type_: AtomEnum,
    ) -> Option<Vec<u32>> {
        let cookie = conn
            .get_property(false, window, property, type_, 0, 64)
            .ok()?;
        let reply = cookie.reply().ok()?;
        reply.value32().map(|values| values.collect())
    }

    pub(crate) fn set_window_opacity(opacity: f32) -> bool {
        use std::sync::atomic::{AtomicU32, Ordering};
        static APPLIED: AtomicU32 = AtomicU32::new(u32::MAX);
        let alpha = opacity.clamp(0.0, 1.0);
        if APPLIED.load(Ordering::Relaxed) == alpha.to_bits() {
            return true;
        }
        let Ok((conn, screen_num)) = RustConnection::connect(None) else {
            return false;
        };
        let Some(root) = conn.setup().roots.get(screen_num) else {
            return false;
        };
        let root = root.root;
        let Some(client_list) = intern_atom(&conn, b"_NET_CLIENT_LIST") else {
            return false;
        };
        let Some(pid_atom) = intern_atom(&conn, b"_NET_WM_PID") else {
            return false;
        };
        let Some(opacity_atom) = intern_atom(&conn, b"_NET_WM_WINDOW_OPACITY") else {
            return false;
        };
        let Some(windows) = property_u32(&conn, root, client_list, AtomEnum::WINDOW) else {
            return false;
        };
        let my_pid = std::process::id();
        for window in windows {
            let matches_pid = property_u32(&conn, window, pid_atom, AtomEnum::CARDINAL)
                .is_some_and(|pids| pids.first() == Some(&my_pid));
            if !matches_pid {
                continue;
            }
            let value = (alpha as f64 * u32::MAX as f64).round() as u32;
            if conn
                .change_property32(
                    PropMode::REPLACE,
                    window,
                    opacity_atom,
                    AtomEnum::CARDINAL,
                    &[value],
                )
                .is_err()
            {
                return false;
            }
            let flushed = conn.flush().is_ok();
            if flushed {
                APPLIED.store(alpha.to_bits(), Ordering::Relaxed);
            }
            return flushed;
        }
        false
    }
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
pub(crate) use imp::set_window_opacity;

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub(crate) fn set_window_opacity(_opacity: f32) -> bool {
    false
}
