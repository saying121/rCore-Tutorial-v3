use sbi_rt::{NoReason, Shutdown, SystemFailure, system_reset};

pub fn console_putchar(c: usize) {
    #[expect(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}

pub fn console_getchar() -> usize {
    #[expect(deprecated)]
    sbi_rt::legacy::console_getchar()
}

pub fn shutdown(failure: bool) -> ! {
    if failure {
        system_reset(Shutdown, SystemFailure);
    } else {
        system_reset(Shutdown, NoReason);
    }
    unreachable!()
}
