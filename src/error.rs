use thiserror::Error;
use wayland_client::{globals::BindError, DispatchError};
/// Error
/// it describe three kind of error
/// 1. failed when init
/// 2. failed in queue
/// 3. failed in shm copy
#[derive(Error, Debug)]
pub enum WaySipError {
    #[error("Init Failed")]
    InitFailed(String),
    #[error("Error during queue")]
    DispatchError(DispatchError),
    #[error("Not supported protocol")]
    NotSupportedProtocol(BindError),
    #[error("Cannot get cursor")]
    NotGetCursorTheme
}
