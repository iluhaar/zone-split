#[cfg(windows)]
pub mod win {
    use crate::ops;
    use std::sync::atomic::{AtomicIsize, Ordering};
    use windows::core::w;
    use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::Graphics::Gdi::{
        BeginPaint, CreatePen, CreateSolidBrush, DeleteObject, EndPaint, FillRect, LineTo,
        MoveToEx, SelectObject, TextOutW, HBRUSH, HDC, PAINTSTRUCT, PS_SOLID,
    };
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::Input::KeyboardAndMouse::{SetFocus, VK_ESCAPE, VK_RETURN};
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, GetSystemMetrics, RegisterClassW,
        SetLayeredWindowAttributes, ShowWindow, CS_HREDRAW, CS_VREDRAW, HMENU, LWA_ALPHA,
        SM_CXSCREEN, SM_CYSCREEN, SW_HIDE, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND,
        WM_KEYDOWN, WM_PAINT, WNDCLASSW, WS_CHILD, WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_WINDOWEDGE,
        WS_OVERLAPPEDWINDOW, WS_POPUP, WS_VISIBLE,
    };

    const PICKER_CLASS: windows::core::PCWSTR = w!("ZoneSplitPickerWindow");
    const OVERLAY_CLASS: windows::core::PCWSTR = w!("ZoneSplitOverlayWindow");
    const BUTTON_SPLIT_50: isize = 1001;

    static PICKER_HWND: AtomicIsize = AtomicIsize::new(0);
    static OVERLAY_HWND: AtomicIsize = AtomicIsize::new(0);

    pub struct OverlayUi {
        picker_hwnd: HWND,
        overlay_hwnd: HWND,
    }

    impl OverlayUi {
        pub fn new() -> ops::Result<Self> {
            let hinstance = unsafe { GetModuleHandleW(None) }
                .map_err(|err| format!("GetModuleHandleW failed: {err}"))?;

            register_class(PICKER_CLASS, Some(picker_wnd_proc))?;
            register_class(OVERLAY_CLASS, Some(overlay_wnd_proc))?;

            let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
            let picker_width = 380;
            let picker_height = 180;
            let picker_x = (screen_width - picker_width) / 2;

            let picker_hwnd = unsafe {
                CreateWindowExW(
                    WS_EX_WINDOWEDGE,
                    PICKER_CLASS,
                    w!("zone-split overlays"),
                    WS_OVERLAPPEDWINDOW,
                    picker_x,
                    120,
                    picker_width,
                    picker_height,
                    None,
                    None,
                    hinstance,
                    None,
                )
                .map_err(|err| format!("CreateWindowExW picker failed: {err}"))?
            };

            unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    w!("BUTTON"),
                    w!("50% / 50% Split"),
                    WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0),
                    32,
                    48,
                    300,
                    56,
                    picker_hwnd,
                    HMENU(BUTTON_SPLIT_50 as *mut std::ffi::c_void),
                    hinstance,
                    None,
                )
                .map_err(|err| format!("CreateWindowExW split button failed: {err}"))?;
            }

            let overlay_hwnd = create_overlay_window()?;

            PICKER_HWND.store(picker_hwnd.0 as isize, Ordering::Relaxed);
            OVERLAY_HWND.store(overlay_hwnd.0 as isize, Ordering::Relaxed);

            unsafe {
                let _ = ShowWindow(picker_hwnd, SW_SHOW);
                let _ = SetFocus(picker_hwnd);
            }

            Ok(Self {
                picker_hwnd,
                overlay_hwnd,
            })
        }
    }

    impl Drop for OverlayUi {
        fn drop(&mut self) {
            unsafe {
                let _ = DestroyWindow(self.overlay_hwnd);
                let _ = DestroyWindow(self.picker_hwnd);
            }
        }
    }

    fn register_class(
        class_name: windows::core::PCWSTR,
        wnd_proc: windows::Win32::UI::WindowsAndMessaging::WNDPROC,
    ) -> ops::Result<()> {
        let hinstance = unsafe { GetModuleHandleW(None) }
            .map_err(|err| format!("GetModuleHandleW failed: {err}"))?;
        let class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: wnd_proc,
            hInstance: hinstance.into(),
            lpszClassName: class_name,
            ..Default::default()
        };

        unsafe {
            RegisterClassW(&class);
        }

        Ok(())
    }

    fn create_overlay_window() -> ops::Result<HWND> {
        let hinstance = unsafe { GetModuleHandleW(None) }
            .map_err(|err| format!("GetModuleHandleW failed: {err}"))?;
        let width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let height = unsafe { GetSystemMetrics(SM_CYSCREEN) };

        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(WS_EX_LAYERED.0 | WS_EX_TOPMOST.0),
                OVERLAY_CLASS,
                w!("zone-split zones"),
                WS_POPUP,
                0,
                0,
                width,
                height,
                None,
                None,
                hinstance,
                None,
            )
            .map_err(|err| format!("CreateWindowExW overlay failed: {err}"))?
        };

        unsafe {
            SetLayeredWindowAttributes(hwnd, COLORREF(0), 150, LWA_ALPHA)
                .map_err(|err| format!("SetLayeredWindowAttributes failed: {err}"))?;
            let _ = ShowWindow(hwnd, SW_HIDE);
        }

        Ok(hwnd)
    }

    extern "system" fn picker_wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_COMMAND if command_id(wparam) == BUTTON_SPLIT_50 => {
                show_overlay();
                LRESULT(0)
            }
            WM_KEYDOWN if wparam.0 as u16 == VK_ESCAPE.0 => {
                let _ = unsafe { ShowWindow(hwnd, SW_SHOW) };
                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
        }
    }

    extern "system" fn overlay_wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_PAINT => {
                paint_overlay(hwnd);
                LRESULT(0)
            }
            WM_KEYDOWN if wparam.0 as u16 == VK_RETURN.0 => {
                hide_overlay();
                LRESULT(0)
            }
            WM_KEYDOWN if wparam.0 as u16 == VK_ESCAPE.0 => {
                hide_overlay();
                show_picker();
                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
        }
    }

    fn command_id(wparam: WPARAM) -> isize {
        (wparam.0 & 0xffff) as isize
    }

    fn hwnd_from_key(key: isize) -> HWND {
        HWND(key as *mut std::ffi::c_void)
    }

    fn show_overlay() {
        let overlay = hwnd_from_key(OVERLAY_HWND.load(Ordering::Relaxed));
        let picker = hwnd_from_key(PICKER_HWND.load(Ordering::Relaxed));
        unsafe {
            let _ = ShowWindow(picker, SW_HIDE);
            let _ = ShowWindow(overlay, SW_SHOW);
            let _ = SetFocus(overlay);
        }
    }

    fn hide_overlay() {
        let overlay = hwnd_from_key(OVERLAY_HWND.load(Ordering::Relaxed));
        unsafe {
            let _ = ShowWindow(overlay, SW_HIDE);
        }
    }

    fn show_picker() {
        let picker = hwnd_from_key(PICKER_HWND.load(Ordering::Relaxed));
        unsafe {
            let _ = ShowWindow(picker, SW_SHOW);
            let _ = SetFocus(picker);
        }
    }

    fn paint_overlay(hwnd: HWND) {
        unsafe {
            let mut paint = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut paint);
            let width = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);
            let mid_x = width / 2;

            fill_rect(hdc, 0, 0, mid_x, height, COLORREF(0x001E3A8A));
            fill_rect(hdc, mid_x, 0, width, height, COLORREF(0x004B1D95));

            let border_pen = CreatePen(PS_SOLID, 4, COLORREF(0x0000B4FF));
            let old_pen = SelectObject(hdc, border_pen);

            let _ = MoveToEx(hdc, mid_x, 0, None);
            let _ = LineTo(hdc, mid_x, height);
            draw_rect_outline(hdc, 8, 8, mid_x - 8, height - 8);
            draw_rect_outline(hdc, mid_x + 8, 8, width - 8, height - 8);

            SelectObject(hdc, old_pen);
            let _ = DeleteObject(border_pen);

            draw_text(hdc, mid_x / 2 - 45, 32, "Left 50%");
            draw_text(hdc, mid_x + (width - mid_x) / 2 - 50, 32, "Right 50%");

            let _ = EndPaint(hwnd, &paint);
        }
    }

    unsafe fn fill_rect(hdc: HDC, left: i32, top: i32, right: i32, bottom: i32, color: COLORREF) {
        let brush = CreateSolidBrush(color);
        let rect = windows::Win32::Foundation::RECT {
            left,
            top,
            right,
            bottom,
        };
        FillRect(hdc, &rect, HBRUSH(brush.0));
        let _ = DeleteObject(brush);
    }

    unsafe fn draw_rect_outline(hdc: HDC, left: i32, top: i32, right: i32, bottom: i32) {
        let _ = MoveToEx(hdc, left, top, None);
        let _ = LineTo(hdc, right, top);
        let _ = LineTo(hdc, right, bottom);
        let _ = LineTo(hdc, left, bottom);
        let _ = LineTo(hdc, left, top);
    }

    unsafe fn draw_text(hdc: HDC, x: i32, y: i32, text: &str) {
        let text: Vec<u16> = text.encode_utf16().collect();
        let _ = TextOutW(hdc, x, y, &text);
    }
}
