#![no_std]
#![no_main]

mod lang_items;
mod sbi;
#[macro_use]
mod console;

use core::arch::global_asm;

global_asm!(include_str!("./entry.asm"));

#[unsafe(no_mangle)]
pub fn rust_main() {
    clear_bss();
    println!("test");
    panic!("Shutdown machine!");
}

fn clear_bss() {
    unsafe extern "C" {
        // safe fn sbss();
        // safe fn ebss();
        safe static sbss: usize;
        safe static ebss: usize;
    }
    // (sbss as usize..ebss as usize).for_each(|a| unsafe {
    (sbss..ebss).for_each(|a| unsafe {
        (a as *mut u8).write_volatile(0);
    });
}
