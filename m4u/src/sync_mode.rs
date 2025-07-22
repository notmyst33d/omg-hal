#[repr(u32)]
pub enum SyncMode {
    CleanByRange,
    InvalidByRange,
    FlushByRange,
    CleanAll,
    InvalidAll,
    FlushAll,
}

impl SyncMode {
    pub fn raw(&self) -> u32 {
        unsafe { *(self as *const SyncMode as *const u32) }
    }
}
