use syscalls::Errno;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("permission denied")]
    PermissionDenied,

    #[error("driver error: {0}")]
    DriverError(i32),
}

impl From<Errno> for Error {
    fn from(value: Errno) -> Self {
        match value {
            Errno::EACCES => Error::PermissionDenied,
            _ => Error::DriverError(value.into_raw()),
        }
    }
}
