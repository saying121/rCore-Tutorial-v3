use crate::{
    config::PAGE_SIZE,
    mm::{page_table::PageTable, PhysAddr, PhysPageNum, VirtAddr},
    task::{
        current_user_token, exit_current_and_run_next, suspend_current_and_run_next, TASK_MANAGER,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let token = current_user_token();
    let page_table = PageTable::from_token(token);
    let sec = ts as usize;
    let usec = sec + 1;

    let sec_pa = get_pa(&page_table, sec);
    let usec_pa = get_pa(&page_table, usec);

    let us = get_time_us();
    unsafe {
        *sec_pa = us / 1_000_000;
        *usec_pa = us % 1_000_000;
    }
    0
}

fn get_pa(page_table: &PageTable, sec: usize) -> *mut usize {
    let sec_va = VirtAddr::from(sec);
    let sec_vpn = sec_va.floor();
    let sec_ppn: PhysPageNum = page_table
        .translate(sec_vpn)
        .unwrap()
        .ppn();

    let mut sec_pa = PhysAddr::from(sec_ppn);
    sec_pa.0 += sec_va.page_offset();
    let sec_pa = sec_pa.0 as *mut usize;
    sec_pa
}

pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    if start % PAGE_SIZE != 0 {
        return -1;
    }

    if prot & !0x7 != 0 {
        return -1;
    }

    if prot & 0x7 == 0 {
        return -1;
    }

    TASK_MANAGER.mmap(start, len, prot)
}

pub fn sys_munmap(start: usize, len: usize) -> isize {
    TASK_MANAGER.munmap(start, len)
}
