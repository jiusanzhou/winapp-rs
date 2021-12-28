#[warn(unused_macros)]

use std::io::Error;
use windows::runtime::Result as WindowsCrateResult;

/// Result type often returned from methods that can have hyper `Error`s.
pub type Result<T> = std::result::Result<T, Error>;

pub enum WindowsResult<T, E> {
    Err(E),
    Ok(T),
}

macro_rules! impl_from_integer_for_windows_result {
    ( $( $integer_type:ty ),+ ) => {
        $(
            impl From<$integer_type> for WindowsResult<$integer_type, Error> {
                fn from(return_value: $integer_type) -> Self {
                    match return_value {
                        0 => Self::Err(std::io::Error::last_os_error().into()),
                        _ => Self::Ok(return_value),
                    }
                }
            }
        )+
    };
}

impl_from_integer_for_windows_result!(isize, u32, i32);

impl<T, E> From<WindowsResult<T, E>> for std::result::Result<T, E> {
    fn from(result: WindowsResult<T, E>) -> Self {
        match result {
            WindowsResult::Err(error) => Self::Err(error),
            WindowsResult::Ok(ok) => Self::Ok(ok),
        }
    }
}

pub trait TakeWindowsCrateResult<T> {
    fn end(self) -> std::io::Result<T>;
}

macro_rules! impl_end_windows_crate_result {
    ( $($input:ty => $deref:ty),+ $(,)? ) => (
        paste::paste! {
            $(
                impl TakeWindowsCrateResult<$deref> for WindowsCrateResult<$input> {
                    fn end(self) -> std::io::Result<$deref> {
                        match self {
                            Ok(value) => Ok(value),
                            Err(error) => Err(error.into()),
                        }
                    }
                }
            )+
        }
    );
}

// impl_end_windows_crate_result!(
//     HWND => HWND,
// );

impl<T> TakeWindowsCrateResult<T> for WindowsCrateResult<T> {
    fn end(self) -> std::io::Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(error.into()),
        }
    }
}

