
use std::fmt::Display;
use std::fmt::Formatter;
use std::convert::TryFrom;

use bindings::Windows::Win32::Foundation::HANDLE;
use bindings::Windows::Win32::Foundation::POINT;
use bindings::Windows::Win32::System::Threading::OpenProcess;
use bindings::Windows::Win32::System::Threading::PROCESS_NAME_FORMAT;
use bindings::Windows::Win32::System::Threading::PROCESS_QUERY_INFORMATION;
use bindings::Windows::Win32::System::Threading::QueryFullProcessImageNameW;
use bindings::Windows::Win32::UI::WindowsAndMessaging::FindWindowW;
use bindings::Windows::Win32::UI::WindowsAndMessaging::GWLP_HWNDPARENT;
use bindings::Windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use bindings::Windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
use bindings::Windows::Win32::UI::WindowsAndMessaging::HWND_NOTOPMOST;
use bindings::Windows::Win32::UI::WindowsAndMessaging::HWND_TOPMOST;
use bindings::Windows::Win32::UI::WindowsAndMessaging::MoveWindow;
use bindings::Windows::Win32::UI::WindowsAndMessaging::SET_WINDOW_POS_FLAGS;
use bindings::Windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE;
use bindings::Windows::Win32::UI::WindowsAndMessaging::SetCursorPos;
use bindings::Windows::Win32::UI::WindowsAndMessaging::SetParent;
use bindings::Windows::Win32::UI::WindowsAndMessaging::SetWindowLongA;
use bindings::Windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrA;
use bindings::Windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW;
use bindings::Windows::Win32::UI::WindowsAndMessaging::SetWindowLongW;
use bindings::Windows::Win32::UI::WindowsAndMessaging::SetWindowPos;
use bindings::Windows::Win32::UI::WindowsAndMessaging::WindowFromPoint;
use bindings::Windows::Win32::{Foundation::{
        HWND,
        PWSTR,
    }, Graphics::Dwm::{DWMWA_CLOAKED, DWMWINDOWATTRIBUTE, DWM_CLOAKED_APP, DWM_CLOAKED_INHERITED, DWM_CLOAKED_SHELL, DwmGetWindowAttribute}, System::Threading::AttachThreadInput, UI::{Input::KeyboardAndMouse::SetFocus, WindowsAndMessaging::{FindWindowA, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, RealGetWindowClassW, SHOW_WINDOW_CMD, SW_HIDE, SW_MAXIMIZE, SW_RESTORE, SW_SHOW, SetForegroundWindow, ShowWindow, WINDOWINFO}}};

use super::rect::Rect;
use super::window_enum::enum_windows;

use super::error::TakeWindowsCrateResult;
use super::error::Result;

// attach 2 windows to each other


// get handle from window
// add event hook
// and make position or visiable changing

// the window for handle
#[derive(Clone, Copy, Debug)]
pub struct Window {
    // window handle
    pub hwnd: HWND,

    // TODO: should we use process/handle handle?
    pub pid: u32,
    pub tid: u32,
}

impl PartialEq for Window {
    fn eq(&self, other: &Window) -> bool {
        self.hwnd == other.hwnd
    }
}

impl Display for Window {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut display = format!("(hwnd: {}", self.hwnd.0);

        if let Some(title) = self.title() {
            display.push_str(&format!(", title: {}", title));
        }

        if let Ok(exe) = self.exe_path() {
            display.push_str(&format!(", exe: {}", exe));
        }

        if let Ok(class) = self.class() {
            display.push_str(&format!(", class: {}", class));
        }

        display.push(')');

        write!(f, "{}", display)
    }
}

impl Default for Window {
    
    fn default() -> Self {
        Window{hwnd: HWND(0), pid: 0, tid: 0}
    }

}

impl From<HWND> for Window {

    fn from(hwnd: HWND) -> Self {
        // get process id and thread id
        let (pid, tid) = Window::get_pid_tid(hwnd);

        // TODO: get process and thread from window hwnd

        Self::new(hwnd, pid, tid)
    }
}

// implement factory methods for window
impl Window {
    pub fn new(hwnd: HWND, pid: u32, tid: u32) -> Self {
        Self {
            hwnd, pid, tid,
        }
    }

    // should use from From trait?
    // pub fn from(hwnd: HWND) -> Option<Self> {
    //     // check invailad
    //     if ! unsafe {IsWindow(hwnd)}.as_bool() {
    //         return None;
    //     }

    //     // get process id and thread id
    //     let (pid, tid) = Window::get_pid_tid(hwnd);

    //     // TODO: get process and thread from window hwnd

    //     Some(Self::new(hwnd, pid, tid))
    // }

    pub fn from_hwnd(hwnd: HWND) -> Option<Self> {
        if !Window::is_valide_window(hwnd) {
            return None;
        }

        Some(hwnd.into())
    }

    // find window by name directlly
    pub fn from_name(class: Option<&str>, name: &str) -> Option<Self> {
        // if class is none use default vlaue of PSTR
        let hwnd = unsafe { 
            if let Some(class) = class {
                FindWindowW(class, name)
            } else {
                FindWindowW(PWSTR::default(), name)
            }
        };

        Self::from_hwnd(hwnd)
    }

    // find first one window by name: enums and filter
    pub fn from_first_name(class: Option<&str>, name: &str) -> Option<Self> {
        // use enum to filter the first one
        let mut my = None ;
        enum_windows(|w| {
            if class.is_none() // don't offer a class to match
                || class.unwrap().eq(w.class().unwrap().as_str())
                || name.eq(w.title().unwrap_or_default().as_str())
            {
                // matched create or copy the a new window object
                my = Self::from_hwnd(w.hwnd);

                // stop enum loop
                return false;
            }

            // don't match continue
            true
        });
        
        // reutrn the result
        my
    }

    // find all windows by name by EnumWindows: enums and filter
    pub fn from_all_name(class: Option<&str>, name: &str) -> Vec<Self> {
        let mut ws: Vec<Window> = vec![];
        enum_windows(|w| {
            if class.is_none() // don't offer a class to match
                || class.unwrap().eq(w.class().unwrap().as_str())
                || name.eq(w.title().unwrap_or_default().as_str())
            {
                // matched create or copy the a new window object
                let my = Self::from_hwnd(w.hwnd);
                if my.is_some() {
                    ws.push(my.unwrap());
                }
            }

            // continue
            true
        });

        ws
    }
    
    // create window from foreground
    pub fn foreground() -> Option<Self> {
        // get foreground window
        let hwnd = unsafe { GetForegroundWindow() };

        Self::from_hwnd(hwnd)
    }

}

// implement static functions
impl Window {

    pub fn is_valide_window(hwnd: HWND) -> bool {
        unsafe {IsWindow(hwnd)}.as_bool()
    }

    pub fn get_pid_tid(hwnd: HWND) -> (u32, u32) {
        let mut pid: u32 = 0;
        let tid = unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
    
        (pid, tid)
    }

    pub fn show_window(hwnd: HWND, cmd: SHOW_WINDOW_CMD) {
        unsafe { ShowWindow(hwnd, cmd) };
    }

    pub fn set_foreground_window(hwnd: HWND) {
        unsafe { SetForegroundWindow(hwnd) };
    }
    
    pub fn set_window_focus(hwnd: HWND) {
        unsafe { SetFocus(hwnd) };
    }

    pub fn dwm_get_window_attribute<T>(
        hwnd: HWND,
        attribute: DWMWINDOWATTRIBUTE,
        value: &mut T,
    ) -> Result<()> {
        unsafe {
            DwmGetWindowAttribute(
                hwnd,
                attribute,
                (value as *mut T).cast(),
                u32::try_from(std::mem::size_of::<T>()).unwrap(),
            )?;
        }

        Ok(())
    }
    
    pub fn is_window_cloaked(hwnd: HWND) -> Result<bool> {
        let mut cloaked: u32 = 0;
        Self::dwm_get_window_attribute(hwnd, DWMWA_CLOAKED, &mut cloaked)?;

        Ok(matches!(
            cloaked,
            DWM_CLOAKED_APP | DWM_CLOAKED_SHELL | DWM_CLOAKED_INHERITED
        ))
    }

    pub fn is_window_visible(hwnd: HWND) -> bool {
        unsafe { IsWindowVisible(hwnd) }.into()
    }

    pub fn is_iconic(hwnd: HWND) -> bool {
        unsafe { IsIconic(hwnd) }.into()
    }

    pub fn get_exe_path(handle: HANDLE) -> Result<String> {
        let mut len = 260_u32;
        let mut path: Vec<u16> = vec![0; len as usize];
        let text_ptr = path.as_mut_ptr();

        unsafe {
            QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_FORMAT(0),
                PWSTR(text_ptr),
                &mut len as *mut u32,
            ).ok()?
        }

        Ok(String::from_utf16(&path[..len as usize]).unwrap())
    }

    pub fn set_window_rect(hwnd: HWND, rect: &Rect, position: HWND, flags: SET_WINDOW_POS_FLAGS) -> Result<()> {
        unsafe {
            SetWindowPos(
                hwnd,
                position, // constant or after window handle
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                flags,
            )
        };

        Ok(())
    }

    pub fn move_window(hwnd: HWND, rect: &Rect, repaint: bool) -> Result<()> {
        unsafe {
            MoveWindow(hwnd, rect.x, rect.y, rect.width, rect.height, repaint)
        };

        Ok(())
    }

    pub fn get_window_rect(hwnd: HWND) -> Result<Rect> {
        let mut rect = unsafe { std::mem::zeroed() };
        unsafe { GetWindowRect(hwnd, &mut rect) }.unwrap();

        Ok(Rect::from(rect))
    }

    fn set_cursor_pos(x: i32, y: i32) -> Result<()> {
        unsafe { SetCursorPos(x, y) }.ok().end()
    }

    pub fn window_from_point(point: (i32, i32)) -> Result<HWND> {
        let mut p = POINT::default();
        p.x = point.0;
        p.y = point.1;
        let hwnd = unsafe { WindowFromPoint(p) };

        Ok(hwnd)
    }

    pub fn window_at_cursor_pos() -> Result<HWND> {
        Self::window_from_point(Self::get_cursor_pos().unwrap())
    }

    pub fn center_cursor_in_rect(rect: &Rect) -> Result<()> {
        Self::set_cursor_pos(rect.x + (rect.width / 2), rect.y + (rect.height / 2))
    }

    pub fn get_cursor_pos() -> Result<(i32, i32)> {
        let mut p = POINT::default();
        unsafe { GetCursorPos(&mut p) };

        Ok((p.x, p.y))
    }

    pub fn set_window_parent(child: HWND, parent: HWND) -> Result<()> {
        unsafe { SetParent(child, parent); }
        Ok(())
    }

    pub fn set_window_owner(child: HWND, owner: HWND) -> Result<()> {
        unsafe { SetWindowLongPtrA(child, GWLP_HWNDPARENT, owner.0); }
        Ok(())
    }

    pub fn set_window_long_w(hwnd: HWND) -> Result<()> {
        // unsafe { SetWindowLongW(hwnd, nindex, dwnewlong); }
        Ok(())
    }
}

// implement getter method for window
impl Window {

    pub fn is_valide(self) -> bool {
        unsafe {IsWindow(self.hwnd)}.as_bool()
    }

    // get process id and thread id
    pub fn pid_tid(&self) -> (u32, u32) {
        Self::get_pid_tid(self.hwnd)
    }
    
    // get class name of window
    pub fn class(&self) -> Result<String> {
        const BUF_SIZE: usize = 512;
        let mut buff: [u16; BUF_SIZE] = [0; BUF_SIZE];

        let writ_chars =
            unsafe { RealGetWindowClassW(self.hwnd, PWSTR(buff.as_mut_ptr()), BUF_SIZE as u32) };

        if writ_chars == 0 {
            return Err(std::io::Error::last_os_error().into());
        }

        Ok(String::from_utf16_lossy(&buff[0..writ_chars as usize]))
    }

    // get title of window
    pub fn title(self) -> Option<String> {
        let mut text: [u16; 512] = [0; 512];
        let len = unsafe { GetWindowTextW(self.hwnd, PWSTR(text.as_mut_ptr()), text.len() as i32) };
        let text = String::from_utf16_lossy(&text[..len as usize]);

        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }

    // get execute path
    pub fn exe_path(self) -> Result<String> {
        let (pid, _) = self.pid_tid();

        let handle = unsafe {
            OpenProcess(PROCESS_QUERY_INFORMATION, false, pid)
        };

        Self::get_exe_path(handle)
    }

    // get rect
    pub fn rect(self) -> Result<Rect> {
        Self::get_window_rect(self.hwnd)
    }
}

// implmenent other method for window
impl Window {

    pub fn attach_thread_input(self, target_thread: u32, attach: bool) -> Result<()> {
        let ok = unsafe { AttachThreadInput(self.tid, target_thread, attach) };
        if ok.as_bool() {
            return Ok(());
        }

        Err(std::io::Error::last_os_error().into())
    }

    pub fn set_foreground(self) {
        Self::set_foreground_window(self.hwnd);
    }

    pub fn show(self) {
        Self::show_window(self.hwnd, SW_SHOW);
    }

    pub fn hidden(self) {
        Self::show_window(self.hwnd, SW_HIDE);
    }

    pub fn restore(self) {
        Self::show_window(self.hwnd, SW_RESTORE);
    }

    pub fn maximize(self) {
        Self::show_window(self.hwnd, SW_MAXIMIZE);
    }

    pub fn set_focus(self) {
        Self::set_window_focus(self.hwnd);
    }

    // TODO: after some window?
    pub fn set_rect(self, rect: &Rect, top: bool) {
        let position = if top { HWND_TOPMOST } else { HWND_NOTOPMOST };
        let _ = Self::set_window_rect(self.hwnd, rect, position, SWP_NOACTIVATE);
    }

    pub fn set_pos(self, point: (i32, i32)) {
        // get rect first
        let mut rect = self.rect().unwrap();
        rect.x = point.0;
        rect.y = point.1;
        let _ = Self::move_window(self.hwnd, &rect, true);
    }

    pub fn set_size(self, size: (i32, i32)) {
        // get rect first
        let mut rect = self.rect().unwrap();
        rect.width = size.0;
        rect.height = size.1;
        let _ = Self::move_window(self.hwnd, &rect, true);
    }

    pub fn set_parent(self, parent: Window) {
        let _ = Self::set_window_parent(self.hwnd, parent.hwnd);
    }

    pub fn set_owner(self, owner: Window) {
        let _ = Self::set_window_owner(self.hwnd, owner.hwnd);
    }
}

#[derive(Debug)]
pub struct WindowInfo {
    pub window_rect: Rect,
    pub client_rect: Rect,
    pub styles: u32,
    pub extended_styles: u32,
    pub x_borders: u32,
    pub y_borders: u32,
}

impl From<WINDOWINFO> for WindowInfo {
    fn from(info: WINDOWINFO) -> Self {
        WindowInfo {
            window_rect: info.rcWindow.into(),
            client_rect: info.rcClient.into(),
            styles: info.dwStyle,
            extended_styles: info.dwExStyle,
            x_borders: info.cxWindowBorders,
            y_borders: info.cxWindowBorders,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_window_find() {
        assert_ne!(Window::from_name(Some("Windows.UI.Core.CoreWindow"), "Window"), None);
        assert_ne!(Window::from_name(None, "Window"), None);
        assert_ne!(Window::from_first_name(None, "Window"), None);
        assert_ne!(Window::from_first_name(Some("Windows.UI.Core.CoreWindow"), "Window"), None);
    }

    #[test]
    fn test_window_setowner() {
        Window::from_name(None, "MINGW64:/d/Zoe").unwrap()
            .set_owner(
                Window::from_name(
                    None,
                     "MINGW64:/c/Users/Zoe",
                ).unwrap()
            )
    }

}
