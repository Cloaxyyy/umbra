use windows_sys::Win32::{
    Foundation::{HWND, RECT},
    UI::WindowsAndMessaging::{
        GetWindowTextW, SPI_GETWORKAREA, SW_RESTORE, SWP_NOACTIVATE, SWP_NOZORDER, SetWindowPos,
        SetWindowTextW, ShowWindow, SystemParametersInfoW,
    },
};

pub fn label(pid: u32, text: &str) {
    let hwnd = super::overlay::window_of(pid);
    if hwnd == 0 {
        return;
    }
    let mut buf = [0u16; 256];
    let n = unsafe { GetWindowTextW(hwnd as HWND, buf.as_mut_ptr(), buf.len() as i32) };
    if String::from_utf16_lossy(&buf[..n.max(0) as usize]) == text {
        return;
    }
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe { SetWindowTextW(hwnd as HWND, wide.as_ptr()) };
}

pub fn grid(n: usize, left: i32, top: i32, right: i32, bottom: i32) -> Vec<(i32, i32, i32, i32)> {
    if n == 0 {
        return vec![];
    }
    let cols = (n as f64).sqrt().ceil() as usize;
    let rows = n.div_ceil(cols);
    let w = (right - left) / cols as i32;
    let h = (bottom - top) / rows as i32;
    (0..n)
        .map(|i| {
            (
                left + (i % cols) as i32 * w,
                top + (i / cols) as i32 * h,
                w,
                h,
            )
        })
        .collect()
}

pub fn arrange(pids: &[u32]) -> usize {
    let hwnds: Vec<isize> = pids
        .iter()
        .map(|p| super::overlay::window_of(*p))
        .filter(|h| *h != 0)
        .collect();
    let mut area = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let ok = unsafe { SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut area as *mut RECT as _, 0) };
    if ok == 0 || area.right <= area.left || area.bottom <= area.top {
        return 0;
    }
    let cells = grid(hwnds.len(), area.left, area.top, area.right, area.bottom);
    for (h, (x, y, w, ht)) in hwnds.iter().zip(cells) {
        unsafe {
            ShowWindow(*h as HWND, SW_RESTORE);
            SetWindowPos(
                *h as HWND,
                std::ptr::null_mut(),
                x,
                y,
                w,
                ht,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }
    hwnds.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grids_fill_the_work_area() {
        assert!(grid(0, 0, 0, 1920, 1040).is_empty());
        assert_eq!(grid(1, 0, 0, 1920, 1040), vec![(0, 0, 1920, 1040)]);
        assert_eq!(
            grid(2, 0, 0, 1920, 1040),
            vec![(0, 0, 960, 1040), (960, 0, 960, 1040)]
        );
        let four = grid(4, 0, 0, 1920, 1040);
        assert_eq!(four[3], (960, 520, 960, 520));
        let five = grid(5, 100, 0, 1900, 1000);
        assert_eq!(five.len(), 5);
        assert_eq!(five[4], (700, 500, 600, 500));
    }
}
