use std::vec;

use windows_sys::core::BOOL;
use windows_sys::Win32::Foundation::{HWND, LPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows_sys::Win32::System::Console::GetConsoleWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, ShowWindow, HWND_TOP,
    SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_SHOWWINDOW,
};

#[derive(Clone)]
pub struct WindowInfo {
    pub handle: HWND,
    pub title: String,
    pub visible: bool,
}

#[derive(Clone, Copy)]
pub struct MonitorBounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Clone, Copy)]
pub struct MonitorInfo {
    pub handle: HMONITOR,
    pub bounds: MonitorBounds,
    pub id: u32,
}

pub struct MonitorWindowCollection {
    pub monitor: MonitorInfo,
    pub windows: Vec<WindowInfo>,
}

impl WindowInfo {
    pub fn is_visible(&self) -> bool {
        self.visible && !self.title.is_empty()
    }
}

impl ToString for MonitorBounds {
    fn to_string(&self) -> String {
        format!(
            "Left: {}, Top: {}, Right: {}, Bottom: {}",
            self.left, self.top, self.right, self.bottom
        )
    }
}

impl ToString for MonitorInfo {
    fn to_string(&self) -> String {
        format!("id: {}, Bounds: {}", self.id, self.bounds.to_string())
    }
}

impl ToString for WindowInfo {
    fn to_string(&self) -> String {
        format!("Title: {}, Visible: {}", self.title, self.visible)
    }
}

pub fn enumerate_windows(visible_only: bool) -> Vec<WindowInfo> {
    let mut windows = Vec::new();
    let param = &mut windows as *mut _ as LPARAM;
    unsafe {
        EnumWindows(Some(enumerate_windows_callback), param);
    };
    if !visible_only {
        return windows;
    }

    windows
        .into_iter()
        .filter(|window: &WindowInfo| window.visible && !window.title.is_empty())
        .collect()
}

pub fn map_windows_to_monitors(
    windows: &Vec<WindowInfo>,
    monitors: &Vec<MonitorInfo>,
) -> Vec<MonitorWindowCollection> {
    let mut montor_window_collections = monitors
        .iter()
        .map(|monitor| MonitorWindowCollection {
            monitor: *monitor,
            windows: Vec::new(),
        })
        .collect::<Vec<_>>();

    for window in windows {
        let monitor_hwnd = unsafe {
            windows_sys::Win32::Graphics::Gdi::MonitorFromWindow(
                window.handle,
                MONITOR_DEFAULTTONEAREST,
            )
        };

        // could do a dictionary lookup here, but how many monitors do you have? 1-3? I think this is fine for now.
        let monitor_id = monitors
            .iter()
            .position(|monitor| monitor.handle == monitor_hwnd);

        montor_window_collections[monitor_id.unwrap_or(0) as usize]
            .windows
            .push(window.clone());
    }

    montor_window_collections
}

unsafe extern "system" fn enumerate_windows_callback(handle: HWND, param: LPARAM) -> BOOL {
    let visible = unsafe { IsWindowVisible(handle) } != 0;
    let str_len = unsafe { GetWindowTextLengthW(handle) };
    let buf = vec![0u16; (str_len + 1) as usize];

    let title = unsafe {
        GetWindowTextW(handle, buf.as_ptr() as *mut u16, str_len + 1);
        String::from_utf16_lossy(&buf[..str_len as usize])
    };

    let window_info = WindowInfo {
        handle,
        title,
        visible,
    };

    let vec = unsafe { &mut *(param as *mut Vec<WindowInfo>) };
    vec.push(window_info);

    1
}

pub fn enumerate_monitors() -> Vec<MonitorInfo> {
    let mut monitors = Vec::new();
    let param = &mut monitors as *mut _ as LPARAM;
    unsafe {
        windows_sys::Win32::Graphics::Gdi::EnumDisplayMonitors(
            0 as HMONITOR,
            std::ptr::null(),
            Some(enumerate_monitor_callback),
            param,
        );
    }
    monitors
}

unsafe extern "system" fn enumerate_monitor_callback(
    monitor: HMONITOR,
    hdc: HDC,
    lprc: *mut windows_sys::Win32::Foundation::RECT,
    param: LPARAM,
) -> BOOL {
    let mut info: MONITORINFO = std::mem::zeroed();
    info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;

    if GetMonitorInfoW(monitor, &mut info) == 0 {
        return 0;
    }

    let vec: &mut Vec<MonitorInfo> = &mut *(param as *mut Vec<MonitorInfo>);

    vec.push(MonitorInfo {
        handle: monitor,
        bounds: MonitorBounds {
            left: info.rcMonitor.left,
            top: info.rcMonitor.top,
            right: info.rcMonitor.right,
            bottom: info.rcMonitor.bottom,
        },
        id: vec.len() as u32,
    });

    1
}

pub fn move_window_to_monitor(window: &WindowInfo, monitor: &MonitorInfo) {
    let bounds = monitor.bounds;

    // show
    let res = unsafe {
        ShowWindow(
            window.handle,
            windows_sys::Win32::UI::WindowsAndMessaging::SW_RESTORE,
        )
    };

    if res == 0 {
        let error = unsafe { windows_sys::Win32::Foundation::GetLastError() };
        println!("Failed to show window: {:?}", error);
    }

    let flags = SWP_SHOWWINDOW | SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE;
    let res = unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
            window.handle,
            HWND_TOP,
            bounds.left,
            bounds.top,
            bounds.right - bounds.left,
            bounds.bottom - bounds.top,
            flags,
        )
    };

    if res == 0 {
        let error = unsafe { windows_sys::Win32::Foundation::GetLastError() };
        println!("Failed to move window: {:?}", error);
    }

    let res = unsafe {
        ShowWindow(
            window.handle,
            windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW,
        )
    };

    if res == 0 {
        let error = unsafe { windows_sys::Win32::Foundation::GetLastError() };
        println!("Failed to show window: {:?}", error);
    }
}

pub fn get_current_monitor_window_collections(collections: &Vec<MonitorWindowCollection>) -> HWND {
    let hwnd = unsafe { GetConsoleWindow() };
    hwnd
}
