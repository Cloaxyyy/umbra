use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};
use windows_sys::Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT},
    UI::WindowsAndMessaging::{
        EnumWindows, GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId, IsIconic,
        IsWindowVisible, SetForegroundWindow,
    },
};

const LABEL: &str = "overlay";
const WIDTH: f64 = 460.0;
const HEIGHT: f64 = 700.0;

static TARGET: Mutex<Option<u32>> = Mutex::new(None);

pub fn target() -> Option<u32> {
    *TARGET.lock().unwrap()
}

pub fn retarget(pid: u32) {
    *TARGET.lock().unwrap() = Some(pid);
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window(LABEL).is_some() {
        return Ok(());
    }
    WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html#overlay".into()))
        .title("Umbra overlay")
        .inner_size(WIDTH, HEIGHT)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .focused(false)
        .shadow(true)
        .build()?;
    Ok(())
}

fn empty_rect() -> RECT {
    RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    }
}

unsafe extern "system" fn each(hwnd: HWND, data: LPARAM) -> BOOL {
    let state = unsafe { &mut *(data as *mut (u32, isize, i64)) };
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
    if pid == state.0 && unsafe { IsWindowVisible(hwnd) } != 0 {
        let mut r = empty_rect();
        unsafe { GetWindowRect(hwnd, &mut r) };
        let area = (r.right - r.left) as i64 * (r.bottom - r.top) as i64;
        if area > state.2 {
            state.2 = area;
            state.1 = hwnd as isize;
        }
    }
    1
}

pub(crate) fn window_of(pid: u32) -> isize {
    let mut state: (u32, isize, i64) = (pid, 0, 0);
    unsafe { EnumWindows(Some(each), &mut state as *mut _ as LPARAM) };
    state.1
}

fn foreground_pid() -> u32 {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(GetForegroundWindow(), &mut pid) };
    pid
}

fn rect(hwnd: isize) -> Option<RECT> {
    if hwnd == 0 {
        return None;
    }
    let mut r = empty_rect();
    let ok = unsafe { GetWindowRect(hwnd as _, &mut r) };
    let iconic = unsafe { IsIconic(hwnd as _) };
    (ok != 0 && iconic == 0 && r.right > r.left).then_some(r)
}

pub fn toggle(app: &AppHandle, running: &super::launch::Running) {
    if create(app).is_err() {
        return;
    }
    let Some(w) = app.get_webview_window(LABEL) else {
        return;
    };
    if w.is_visible().unwrap_or(false) {
        hide(app, true);
        return;
    }
    let front = foreground_pid();
    if running.owner(front).is_some() {
        retarget(front);
    }
    let picked = target().filter(|p| running.owner(*p).is_some());
    if let Some(r) = picked.and_then(|p| rect(window_of(p))) {
        let scale = w.scale_factor().unwrap_or(1.0);
        let width = (WIDTH * scale) as i32;
        let x = (r.right - width - (24.0 * scale) as i32).max(r.left);
        let y = r.top + (56.0 * scale) as i32;
        let _ = w.set_position(PhysicalPosition::new(x, y));
    } else {
        let _ = w.center();
    }
    let _ = w.emit_to(LABEL, "overlay-open", picked);
    let _ = w.show();
    let _ = w.set_focus();
}

pub fn hide(app: &AppHandle, refocus: bool) {
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.hide();
    }
    if refocus {
        if let Some(pid) = target() {
            let hwnd = window_of(pid);
            if hwnd != 0 {
                unsafe {
                    SetForegroundWindow(hwnd as _);
                }
            }
        }
    }
}
