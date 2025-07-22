pub(crate) const MTK_M4U_T_ALLOC_MVA: usize = 0xc0046704;
pub(crate) const MTK_M4U_T_DEALLOC_MVA: usize = 0x40046705;
pub(crate) const MTK_M4U_T_CACHE_SYNC: usize = 0x4004670a;

#[repr(C)]
pub(crate) struct Module {
    pub(crate) port: u32,
    pub(crate) buf_addr: usize,
    pub(crate) buf_size: u32,
    pub(crate) prot: u32,
    pub(crate) mva_start: u32,
    pub(crate) mva_end: u32,
    pub(crate) flags: u32,
}

#[repr(C)]
pub(crate) struct Cache {
    pub(crate) port: u32,
    pub(crate) sync_mode: u32,
    pub(crate) va: usize,
    pub(crate) size: u32,
    pub(crate) mva: u32,
}
