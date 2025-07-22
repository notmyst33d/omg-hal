#![no_std]

pub mod error;
mod ioctl;
pub mod mva;
pub mod port;
pub mod sync_mode;

use syscalls::{Sysno, syscall};

pub use crate::error::Error;
use crate::ioctl::{MTK_M4U_T_ALLOC_MVA, Module};
use crate::mva::Mva;
pub use crate::port::*;
pub use crate::sync_mode::SyncMode;

/// # M4U HAL
/// # Safety
/// This struct supports `Drop` and will
/// automatically free all resources.
pub struct M4u(usize);

impl Drop for M4u {
    fn drop(&mut self) {
        unsafe {
            syscall!(Sysno::close, self.0).unwrap();
        }
    }
}

impl M4u {
    pub fn open() -> Result<M4u, Error> {
        Ok(M4u(unsafe {
            syscall!(Sysno::openat, 0, c"/proc/m4u".as_ptr(), 0)
        }?))
    }

    /// Allocate new buffer using M4U
    pub fn alloc<T: Port>(&self, port: T, buf: *const u8, len: u32) -> Result<Mva<T>, Error> {
        // M4U requires 16-byte alignment
        assert_eq!(buf.align_offset(16), 0, "invalid alignment for m4u");

        let va = buf.expose_provenance();
        let module = Module {
            port: port.raw(),
            buf_addr: va,
            buf_size: len,
            prot: 3, // TODO: This sets RW permissions
            mva_start: 0,
            mva_end: 0,
            flags: 0,
        };

        unsafe {
            syscall!(
                Sysno::ioctl,
                self.0,
                MTK_M4U_T_ALLOC_MVA,
                (&module) as *const Module
            )?;
        }

        Ok(Mva {
            port,
            va,
            start: module.mva_start,
            len,
            _parent: self,
        })
    }
}
