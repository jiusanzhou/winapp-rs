fn main() {
    windows::build! {
        // Note that we're using the `Intl` namespace which is nested inside the `Win32` namespace
        // which itself is inside the `Windows` namespace.
        Windows::Win32::System::Registry::{
            HKEY_LOCAL_MACHINE,
            HKEY_CURRENT_USER,
            RegGetValueA,
            RegOpenKeyExA,
            RegQueryValueExA,
        },
        Windows::Win32::Foundation::MAX_PATH,
        Windows::UI::Xaml::Controls::WebView,
    };
}
