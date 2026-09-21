pub(crate) const WINDOW_TITLE: &str = "桌面日历待办";

#[cfg(target_os = "windows")]
mod imp {
    const GWL_EXSTYLE: i32 = -20;
    const WS_EX_LAYERED: isize = 0x0008_0000;
    const LWA_ALPHA: u32 = 0x2;

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

    pub(crate) fn set_window_opacity(opacity: f32) -> bool {
        let Some(hwnd) = find_main_window() else {
            return false;
        };
        let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
        let style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
        let new_style = if opacity < 1.0 {
            style | WS_EX_LAYERED
        } else {
            style & !WS_EX_LAYERED
        };
        if new_style != style {
            unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style) };
        }
        unsafe { SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA) != 0 }
    }
}

#[cfg(target_os = "windows")]
pub(crate) use imp::set_window_opacity;

#[cfg(not(target_os = "windows"))]
pub(crate) fn set_window_opacity(_opacity: f32) -> bool {
    false
}
