use bindings::Windows::Win32::{
    Foundation::{
        HWND,
        BOOL,
        LPARAM,
    },
    UI::WindowsAndMessaging::{
        EnumWindows,
    }
};

use super::window::Window;

struct EnumWindowsContext<'a> {
	callback: &'a mut dyn FnMut(Window) -> bool,
}

#[allow(non_snake_case)]
unsafe extern "system" fn thunk(hwnd: HWND, lParam: LPARAM) -> BOOL {
	let context = &mut *(lParam.0 as *mut EnumWindowsContext);
	if (context.callback)(hwnd.into()) { true.into() }
	else { false.into() }
}

// enum windows
pub fn enum_windows<F>(mut f: F) -> bool where F: FnMut(Window) -> bool {
    // create the new windows enum context
    let mut ctx = EnumWindowsContext{
        callback: &mut f,
    };
    unsafe {
        EnumWindows(Some(thunk), LPARAM(&mut ctx as *mut _ as _)).into()
    }
}

#[cfg(test)]
mod tests {
    use super::enum_windows;

    #[test]
    fn test_enum_windows() {
        let mut count = 0;
        enum_windows(|w| {
            println!("{}", w);

            if count > 10 {
                return false;
            }

            count += 1;

            true
        });
    }
}