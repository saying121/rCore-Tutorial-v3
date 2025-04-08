#![no_std]
#![no_main]

#[macro_use]
mod console;
mod lang_items;
mod logging;
mod sbi;

use core::arch::global_asm;

use log::{debug, error, info, trace, warn};

global_asm!(include_str!("./entry.asm"));

unsafe extern "C" {
    safe fn stext(); // begin addr of text segment
    safe fn etext(); // end addr of text segment
    safe fn srodata(); // start addr of Read-Only data segment
    safe fn erodata(); // end addr of Read-Only data ssegment
    safe fn sdata(); // start addr of data segment
    safe fn edata(); // end addr of data segment
    safe fn sbss(); // start addr of BSS segment
    safe fn ebss(); // end addr of BSS segment
    safe fn boot_stack_lower_bound(); // stack lower bound
    safe fn boot_stack_top(); // stack top
}

#[unsafe(no_mangle)]
pub fn rust_main() {
    clear_bss();
    logging::Logger::init();
    println!("[kernel], Hello, world!");

    trace!(
        "[kernel] .text [{:#x}, {:#x})",
        stext as usize, etext as usize
    );
    debug!(
        "[kernel] .rodata [{:#x}, {:#x})",
        srodata as usize, erodata as usize
    );
    info!(
        "[kernel] .data [{:#x}, {:#x})",
        sdata as usize, edata as usize
    );
    warn!(
        "[kernel] boot_stack top=bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as usize, boot_stack_lower_bound as usize
    );
    error!("[kernel] .bss [{:#x}, {:#x})", sbss as usize, ebss as usize);

    sbi::shutdown(false);
}

fn clear_bss() {
    // unsafe extern "C" {
    //     safe static sbss: usize;
    //     safe static ebss: usize;
    // }
    (sbss as usize..ebss as usize).for_each(|a| unsafe {
        // (sbss..ebss).for_each(|a| unsafe {
        (a as *mut u8).write_volatile(0);
    });
}
