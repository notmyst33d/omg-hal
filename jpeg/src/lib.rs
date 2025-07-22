#![no_std]

use mtk_m4u::{M4u, Port, SyncMode};
use syscalls::{Errno, Sysno, syscall};

const JPEG_ENC_IOCTL_INIT: usize = 0x780b;
const JPEG_ENC_IOCTL_DEINIT: usize = 0x780e;
const JPEG_ENC_IOCTL_START: usize = 0x780f;
const JPEG_ENC_IOCTL_WAIT: usize = 0xc020780d;
const JPEG_ENC_IOCTL_CONFIG: usize = 0x4040780c;

#[repr(u32)]
#[derive(Default)]
pub enum Format {
    #[default]
    Yuy2 = 0,
    Yvyu = 1,
    Nv12 = 2,
    Nv21 = 3,
    Yuv444 = 444,
    Yuv422 = 422,
    Yuv411 = 411,
    Yuv420 = 420,
    Grayscale = 400,
}

#[repr(u32)]
#[derive(Default)]
pub enum Quality {
    Q60 = 0x0,
    Q80 = 0x1,
    #[default]
    Q90 = 0x2,
    Q95 = 0x3,

    Q39 = 0x4,
    Q68 = 0x5,
    Q84 = 0x6,
    Q92 = 0x7,

    Q48 = 0x9,
    Q74 = 0xA,
    Q87 = 0xB,

    Q34 = 0xD,
    Q64 = 0xE,
    Q82 = 0xF,

    QAll = 0x10,
}

#[repr(C)]
#[derive(Default)]
struct MtkEncodeConfig {
    dst_addr: u32,
    dst_size: u32,

    width: u32,
    height: u32,

    dst_exif_en: u8,
    _alloc_buffer: u8, // Unused

    quality: Quality,
    yuv_format: Format,

    _disable_gmc: u32, // Unused
    restart_interval: u32,
    luma_addr: u32,
    chroma_addr: u32,
    img_stride: u32,
    mem_stride: u32,
    total_enc_du: u32,
    dst_offset_addr: u32,
    dst_byte_offset_mask: u32,
}

#[repr(C)]
struct MtkEncodeResult {
    timeout: u64,
    file_size: *mut u32,
    result: *mut u32,
    cycle_count: *mut u32,
}

pub struct EncodeConfig<'a, T: Port> {
    pub width: u32,
    pub height: u32,
    pub uv_plane: &'a [u8],
    pub y_plane: &'a [u8],
    pub output: &'a mut [u8],
    pub read_port: T,
    pub write_port: T,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("permission denied")]
    PermissionDenied,

    #[error("syscall error: {0}")]
    SyscallError(i32),

    #[error("{0} ioctl error: {1}")]
    IoctlError(&'static str, &'static str),

    #[error(transparent)]
    M4uError(#[from] mtk_m4u::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum EncodeError {
    #[error("output buffer too small")]
    OutputBufferTooSmall,

    #[error("timeout")]
    Timeout,

    #[error("unknown error")]
    Unknown,

    #[error(transparent)]
    IoError(#[from] Error),

    #[error(transparent)]
    M4uError(#[from] mtk_m4u::Error),
}

impl From<Errno> for Error {
    fn from(value: Errno) -> Self {
        match value {
            Errno::EACCES => Error::PermissionDenied,
            _ => Error::SyscallError(value.into_raw()),
        }
    }
}

trait MathExt {
    type Output;

    fn stepceil(self, step: Self) -> Self::Output;
}

impl MathExt for u32 {
    type Output = u32;

    fn stepceil(self, step: Self) -> Self::Output {
        let x = self;
        let a = step;
        (x + (a - 1)) & !(a - 1)
    }
}

trait ResultExt {
    fn into_ioctl_result(self, ioctl_name: &'static str) -> Result<usize, Error>;
}

impl ResultExt for Result<usize, Errno> {
    fn into_ioctl_result(self, ioctl_name: &'static str) -> Result<usize, Error> {
        self.map_err(move |e| Error::IoctlError(ioctl_name, e.name().unwrap_or("no name")))
    }
}

/// # Mediatek JPEG Encoder HAL
/// # Safety
/// This struct supports `Drop` and will
/// automatically free all resources.
pub struct MtkJpeg {
    fd: usize,
    m4u: M4u,
}

impl Drop for MtkJpeg {
    fn drop(&mut self) {
        unsafe {
            syscall!(Sysno::ioctl, self.fd, JPEG_ENC_IOCTL_DEINIT).unwrap();
            syscall!(Sysno::close, self.fd).unwrap();
        }
    }
}

impl MtkJpeg {
    pub fn open() -> Result<MtkJpeg, Error> {
        let fd = unsafe { syscall!(Sysno::openat, 0, c"/proc/mtk_jpeg".as_ptr(), 0) }?;
        unsafe { syscall!(Sysno::ioctl, fd, JPEG_ENC_IOCTL_INIT) }
            .into_ioctl_result("JPEG_ENC_IOCTL_INIT")?;
        Ok(MtkJpeg {
            fd,
            m4u: M4u::open()?,
        })
    }

    /// Encode NV12 buffers using Mediatek JPEG encoder
    pub fn encode<T: Port>(&self, config: &EncodeConfig<T>) -> Result<u32, EncodeError> {
        let y_plane = self.m4u.alloc(
            config.read_port,
            config.y_plane.as_ptr(),
            config.y_plane.len() as u32,
        )?;
        let uv_plane = self.m4u.alloc(
            config.read_port,
            config.uv_plane.as_ptr(),
            config.uv_plane.len() as u32,
        )?;
        let output = self.m4u.alloc(
            config.write_port,
            config.output.as_ptr(),
            config.output.len() as u32,
        )?;

        // M4U should have up-to-date buffers.
        y_plane.flush(SyncMode::FlushByRange)?;
        uv_plane.flush(SyncMode::FlushByRange)?;
        output.flush(SyncMode::InvalidByRange)?;

        let mtk_config = MtkEncodeConfig {
            width: config.width,
            height: config.height,
            yuv_format: Format::Nv12,
            quality: Quality::Q90,
            luma_addr: y_plane.start,
            chroma_addr: uv_plane.start,
            total_enc_du: (config.width.stepceil(16) / 16) * (config.height.stepceil(16) / 16) * 6
                - 1,
            dst_addr: output.start,
            dst_size: output.len,
            restart_interval: 0,
            img_stride: config.width.stepceil(16),
            mem_stride: config.width.stepceil(16),
            ..Default::default()
        };

        unsafe {
            syscall!(
                Sysno::ioctl,
                self.fd,
                JPEG_ENC_IOCTL_CONFIG,
                &mtk_config as *const MtkEncodeConfig
            )
        }
        .into_ioctl_result("JPEG_ENC_IOCTL_CONFIG")?;

        unsafe { syscall!(Sysno::ioctl, self.fd, JPEG_ENC_IOCTL_START) }
            .into_ioctl_result("JPEG_ENC_IOCTL_START")?;

        let mut result: u32 = 0;
        let mut file_size: u32 = 0;
        let mut cycle_count: u32 = 0;
        let encode_result = MtkEncodeResult {
            timeout: 1000,
            file_size: &mut file_size as *mut u32,
            result: &mut result as *mut u32,
            cycle_count: &mut cycle_count as *mut u32,
        };

        if let Err(e) = unsafe {
            syscall!(
                Sysno::ioctl,
                self.fd,
                JPEG_ENC_IOCTL_WAIT,
                &encode_result as *const MtkEncodeResult
            )
        } {
            // EFAULT should not be considered an IoctlError.
            // This error usually means that the output buffer is too small.
            if e == Errno::EFAULT {
                return Err(EncodeError::OutputBufferTooSmall);
            }
            return Err(EncodeError::IoError(Error::IoctlError(
                "JPEG_ENC_IOCTL_WAIT",
                e.name().unwrap_or("no name"),
            )));
        }

        // Flush all data from M4U cache.
        y_plane.flush(SyncMode::FlushByRange)?;
        uv_plane.flush(SyncMode::FlushByRange)?;
        output.flush(SyncMode::InvalidByRange)?;

        match result {
            0 => Ok(file_size),
            1 => Err(EncodeError::OutputBufferTooSmall),
            2 => Err(EncodeError::Timeout),
            _ => Err(EncodeError::Unknown),
        }
    }
}
