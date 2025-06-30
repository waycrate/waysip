use thiserror::Error;
use wayland_client::{DispatchError, globals::BindError};
/// Error
/// it describe three kind of error
/// 1. failed when init
/// 2. failed in queue
/// 3. failed when protocol not supported
/// 4. when not get the cursor
#[derive(Error, Debug)]
pub enum WaySipError {
    #[error("Failed to initialize app state")]
    InitFailed(String),
    #[error("Wayland dispatch failed!")]
    DispatchError(DispatchError),
    #[error("Protocol not supported")]
    NotSupportedProtocol(BindError),
    #[error("Cannot get cursor")]
    CursorThemeFetchFailed,
}

#[derive(Debug, Error)]
pub enum ColorError {
    #[error("Invalid color format `{0}`, expected `#rrggbbaa/rrggbbaa`")]
    InvalidColorFormat(String),
    #[error("Invalid color value: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}

#[derive(Debug, Error)]
pub enum BoxInfoError {
    #[error("Invalid string format for Box `{0}`, expected `x,y WIDTHxHEIGHT`")]
    InvalidBoxString(String),
    #[error("Invalid box coordinates string `{0}`, expected `x,y`")]
    InvalidBoxCoordsString(String),
    #[error("Invalid box size string `{0}`, expected `WIDTHxHEIGHT`")]
    InvalidBoxSizeString(String),
    #[error("Invalid box info value: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}
