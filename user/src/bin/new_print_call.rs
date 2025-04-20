#![no_std]
#![no_main]

use core::arch::asm;
use core::ptr;

#[macro_use]
extern crate user_lib;

#[unsafe(no_mangle)]
fn main() -> i32 {
    read_sp();
    0
}
pub fn read_sp() {
    let mut fp: *const usize;
    unsafe { asm!("mv {}, fp", out(reg) fp) }
    while !fp.is_null() {
        unsafe {
            let return_address = ptr::read(fp.sub(1)); // ra
            let old_fp = ptr::read(fp.sub(2)) as *const usize; // prev fp

            println!("\nReturn address: 0x{:016x}", return_address);
            println!("Old address: 0x{:016x}", old_fp as usize);
            println!("");
            fp = old_fp;
        }
    }
}
