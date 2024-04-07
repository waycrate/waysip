use thiserror::Error;
use wayland_client::{globals::BindError, DispatchError};
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
