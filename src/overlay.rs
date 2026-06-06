#[cfg(windows)]
pub mod win {
    use crate::ops;
    use windows::core::w;
    use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::Graphics::Gdi::{
        BeginPaint, CreatePen, CreateSolidBrush, DeleteObject, EndPaint, FillRect, LineTo,
        MoveToEx, SelectObject, TextOutW, HBRUSH, HDC, PAINTSTRUCT, PS_SOLID,
    };
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, GetSystemMetrics, RegisterClassW,
        SetLayeredWindowAttributes, ShowWindow, CS_HREDRAW, CS_VREDRAW, LWA_ALPHA, SM_CXSCREEN,
        SM_CYSCREEN, SW_HIDE, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_PAINT, WNDCLASSW,
        WS_DISABLED, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
    };

    const CLASS_NAME: windows::core::PCWSTR = w!("ZoneSplitOverlayWindow");

    pub struct Overlay {
        hwnd: HWND,
        visible: bool,
    }

    impl Overlay {
        pub fn new_primary_split() -> ops::Result<Self> {
            let hinstance = unsafe { GetModuleHandleW(None) }
                .map_err(|err| format!("GetModuleHandleW failed: {err}"))?;

            let class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: hinstance.into(),
                lpszClassName: CLASS_NAME,
                ..Default::default()
            };

            unsafe {
                RegisterClassW(&class);
            }

            let width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
            let height = unsafe { GetSystemMetrics(SM_CYSCREEN) };

            let hwnd = unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE(
                        WS_EX_LAYERED.0
                            | WS_EX_TRANSPARENT.0
                            | WS_EX_TOPMOST.0
                            | WS_EX_TOOLWINDOW.0,
                    ),
                    CLASS_NAME,
                    w!("zone-split zones"),
                    WINDOW_STYLE(WS_POPUP.0 | WS_DISABLED.0),
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
                let _ = ShowWindow(hwnd, SW_SHOW);
            }

            Ok(Self {
                hwnd,
                visible: true,
            })
        }

        pub fn toggle(&mut self) {
            self.visible = !self.visible;
            unsafe {
                let _ = ShowWindow(self.hwnd, if self.visible { SW_SHOW } else { SW_HIDE });
            }
        }
    }

    impl Drop for Overlay {
        fn drop(&mut self) {
            unsafe {
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }

    extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_PAINT => {
                paint_overlay(hwnd);
                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
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
