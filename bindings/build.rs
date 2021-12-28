fn main() {
    windows::runtime::build! {
        // Note that we're using the `Intl` namespace which is nested inside the `Win32` namespace
        // which itself is inside the `Windows` namespace.
        Windows::Win32::System::Registry::*,
        Windows::Win32::Foundation::*,
        Windows::Win32::System::Threading::*,
        Windows::Win32::Graphics::Dwm::*,
        Windows::Win32::UI::Accessibility::*,
        Windows::Win32::UI::WindowsAndMessaging::*,
        Windows::Win32::UI::Input::KeyboardAndMouse::*,
    };
}
