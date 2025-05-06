use alloc::sync::Arc;

use crate::{
    config::PAGE_SIZE,
    loader::get_app_data_by_name,
    mm::{
        page_table::PageTable, translated_refmut, translated_str, PhysAddr, PhysPageNum, VirtAddr,
    },
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, task::TaskControlBlock,
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
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();
    // find a child process

    // ---- access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    if inner
        .children
        .iter()
        .find(|p| pid == -1 || pid as usize == p.getpid())
        .is_none()
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB lock exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after removing from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child TCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB lock automatically
}

pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let token = current_user_token();
    let page_table = PageTable::from_token(token);
    let sec = ts as usize;

    let sec_pa = get_pa(&page_table, sec);
    let usec_pa = unsafe { sec_pa.add(1) };

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
    let sec_ppn: PhysPageNum = page_table.translate(sec_vpn).unwrap().ppn();

    let mut sec_pa = PhysAddr::from(sec_ppn);
    sec_pa.0 += sec_va.page_offset();

    sec_pa.0 as *mut usize
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

    current_task().unwrap().mmap(start, len, prot)
}

pub fn sys_munmap(start: usize, len: usize) -> isize {
    current_task().unwrap().munmap(start, len)
}

pub fn sys_spawn(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);

    let Some(data) = get_app_data_by_name(&path) else {
        return -1;
    };

    let tcb = Arc::new(TaskControlBlock::new(data));
    let pid = tcb.pid.0;
    {
        let mut new_tcb_inner = tcb.inner_exclusive_access();
        let parent_task = current_task().unwrap();
        new_tcb_inner.parent = Some(Arc::downgrade(&parent_task));
        new_tcb_inner.get_trap_cx().x[10] = 0;

        let mut parent_inner = parent_task.inner_exclusive_access();
        parent_inner.children.push(tcb.clone());
    }

    add_task(tcb);

    pid as isize
}
