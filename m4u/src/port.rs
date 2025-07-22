pub trait Port: Copy {
    fn raw(&self) -> u32;
}

pub mod mt6768 {
    use crate::Port as CommonPort;

    #[repr(u32)]
    #[derive(Clone, Copy)]
    pub enum Port {
        JpgencRdma = 55,
        JpgencBsdma = 56,
    }

    impl CommonPort for Port {
        fn raw(&self) -> u32 {
            unsafe { *(self as *const Port as *const u32) }
        }
    }
}
