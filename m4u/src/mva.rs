use syscalls::{Sysno, syscall};

use crate::{
    Error, M4u, Port, SyncMode,
    ioctl::{Cache, MTK_M4U_T_CACHE_SYNC, MTK_M4U_T_DEALLOC_MVA, Module},
};

pub struct Mva<'a, T: Port> {
    pub port: T,
    pub start: u32,
    pub va: usize,
    pub len: u32,
    pub(crate) _parent: &'a M4u,
}

impl<'a, T: Port> Drop for Mva<'a, T> {
    fn drop(&mut self) {
        unsafe {
            syscall!(
                Sysno::ioctl,
                self._parent.0,
                MTK_M4U_T_DEALLOC_MVA,
                (&Module {
                    port: self.port.raw(),
                    buf_addr: 0,
                    buf_size: 0,
                    prot: 0,
                    mva_start: self.start,
                    mva_end: 0,
                    flags: 0,
                }) as *const Module
            )
            .unwrap();
        }
    }
}

impl<'a, T: Port> Mva<'a, T> {
    pub fn flush(&self, mode: SyncMode) -> Result<(), Error> {
        unsafe {
            syscall!(
                Sysno::ioctl,
                self._parent.0,
                MTK_M4U_T_CACHE_SYNC,
                (&Cache {
                    port: self.port.raw(),
                    sync_mode: mode.raw(),
                    va: self.va,
                    size: self.len,
                    mva: self.start
                }) as *const Cache
            )?;
        }
        Ok(())
    }
}
