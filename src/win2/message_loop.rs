// message loop

use std::thread;
use std::time::Duration;

use bindings::Windows::Win32::Foundation::HWND;
use bindings::Windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW,
    PM_REMOVE,
    MSG, PeekMessageW, TranslateMessage};


#[derive(Debug, Copy, Clone)]
pub struct MessageLoop;

impl MessageLoop {
    pub fn start(sleep: u64, cb: impl Fn(Option<MSG>) -> bool) {
        Self::start_with_sleep(sleep, cb);
    }

    pub fn start_with_sleep(sleep: u64, cb: impl Fn(Option<MSG>) -> bool) {
        let mut msg: MSG = MSG::default();
        loop {
            let mut value: Option<MSG> = None;
            unsafe {
                if !bool::from(!PeekMessageW(
                    &mut msg, 
                    HWND(0),
                    0, 
                    0, 
                    PM_REMOVE,
                )) {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);

                    value = Some(msg);
                }
            }

            // thread::sleep(Duration::from_millis(sleep));

            if !cb(value) {
                break;
            }
        }
    }
}
