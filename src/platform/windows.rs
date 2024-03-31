#![allow(unused_unsafe)]

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct PlatformState {
    instance: HINSTANCE,
    window: HWND,
}
impl PlatformState {
    pub fn startup(app_name: &str, x: i32, y: i32, width: i32, height: i32) -> Result<Self> {
        let mut out = Self {
            instance: HINSTANCE::default(),
            window: HWND::default(),
        };
        unsafe {
            GetModuleHandleExW(0, None, &mut out.instance.into() as *mut _)?;
        }
        if unsafe {
            RegisterClassExW(&WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_DBLCLKS,
                lpfnWndProc: Some(Self::win32_process_messages),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: out.instance,
                hIcon: unsafe { LoadIconW(out.instance, IDI_APPLICATION)? },
                hCursor: unsafe { LoadCursorW(out.instance, IDC_ARROW)? },
                hbrBackground: HBRUSH(0),
                lpszMenuName: PCWSTR::null(),
                lpszClassName: w!("strings_window_class"),
                hIconSm: HICON(0),
            } as *const _)
        } == 0
        {
            unsafe {
                MessageBoxW(
                    None,
                    w!("Window registration failed"),
                    w!("Error"),
                    MB_ICONEXCLAMATION | MB_OK,
                );
            }
            return Err(Error::from_win32());
        }

        let client_x = x as u32;
        let client_y = y as u32;
        let client_width = width as u32;
        let client_height = height as u32;

        let mut window_x = client_x;
        let mut window_y = client_y;
        let mut window_width = client_width;
        let mut window_height = client_height;

        let mut window_style = WS_OVERLAPPED | WS_SYSMENU | WS_CAPTION;
        let window_ex_style = WS_EX_APPWINDOW;

        window_style |= WS_MAXIMIZEBOX;
        window_style |= WS_MINIMIZEBOX;
        window_style |= WS_THICKFRAME;

        let mut border = RECT::default();
        unsafe {
            AdjustWindowRectEx(&mut border as *mut _, window_style, None, window_ex_style)?;
        }

        window_x = (window_x as i32 + border.left) as u32;
        window_y = (window_y as i32 + border.top) as u32;

        window_width = (window_width as i32 + (border.right - border.left)) as u32;
        window_height = (window_height as i32 + (border.bottom - border.top)) as u32;

        out.window = unsafe {
            CreateWindowExW(
                window_ex_style,
                w!("strings_window_class"),
                PCWSTR::from_raw(
                    app_name
                        .encode_utf16()
                        .chain(std::iter::repeat(0).take(1))
                        .collect::<Vec<_>>()
                        .as_ptr(),
                ),
                window_style,
                window_x as i32,
                window_y as i32,
                window_width as i32,
                window_height as i32,
                None,
                None,
                out.instance,
                None,
            )
        };
        if out.window.0 == 0 {
            unsafe {
                MessageBoxW(
                    None,
                    w!("Window creation failed!"),
                    w!("Error!"),
                    MB_ICONEXCLAMATION | MB_OK,
                );
            }
            crate::fatal!("Window creation failed!");
            return Err(Error::from_win32());
        }

        let should_activate = true;
        unsafe {
            ShowWindow(
                out.window,
                if should_activate {
                    SW_SHOW
                } else {
                    SW_SHOWNOACTIVATE
                },
            );
        }
        colored::control::set_virtual_terminal(true).unwrap();

        Ok(out)
    }
    pub fn shutdown(&mut self) -> Result<()> {
        if self.window.0 != 0 {
            unsafe {
                DestroyWindow(self.window)?;
            }
            self.window = HWND::default();
        }
        Ok(())
    }
    pub fn pump_messages(&self) -> bool {
        let mut message = MSG::default();
        while unsafe { PeekMessageW(&mut message as *mut _, None, 0, 0, PM_REMOVE) }.0 > 0 {
            unsafe {
                TranslateMessage(&message as *const _);
                DispatchMessageW(&message as *const _);
            }
        }
        true
    }
    unsafe extern "system" fn win32_process_messages(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message {
            WM_ERASEBKGND => {
                return LRESULT(1);
            }
            WM_CLOSE => {
                return LRESULT(0);
            }
            WM_DESTROY => {
                unsafe {
                    PostQuitMessage(0);
                }
                return LRESULT(0);
            }
            WM_SIZE => {
                // Get the updated size.
                // let mut r = RECT::default();
                // unsafe {
                //     GetClientRect(window, &mut r as *mut _).unwrap();
                // }
                // let width = (r.right - r.left) as u32;
                // let height = (r.bottom - r.top) as u32;
                // TODO: Fire an event for window resize.
            }
            WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP => {
                // Key pressed/released
                // let pressed = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
                // TODO: input processing
            }
            WM_MOUSEMOVE => {
                // Mouse move
                // let x_position = (lparam.0 & 0xffff) as i32;
                // let y_position = ((lparam.0 >> 16) & 0xffff) as i32;
                // TODO: input processing.
            }
            WM_MOUSEWHEEL => {
                // let z_delta = ((wparam.0 >> 16) & 0xffff) as i32;
                // if z_delta != 0 {
                //     // Flatten the input to an OS-independent (-1, 1)
                //     z_delta = if z_delta < 0 {-1} else {1};
                //     // TODO: input processing.
                // }
            }
            WM_LBUTTONDOWN | WM_MBUTTONDOWN | WM_RBUTTONDOWN | WM_LBUTTONUP | WM_MBUTTONUP
            | WM_RBUTTONUP => {
                // let pressed = message == WM_LBUTTONDOWN || message == WM_RBUTTONDOWN || message == WM_MBUTTONDOWN;
                // TODO: input processing.
            }
            _ => {}
        }
        DefWindowProcW(window, message, wparam, lparam)
    }
}