//! File and filesystem-related syscalls

use core::ops::Range;

use log::debug;

use crate::batch::{get_cur_app_range, get_user_stack_range};

const FD_STDOUT: usize = 1;

fn in_range(range: &Range<usize>, start: &usize, end: &usize) -> bool {
    range.contains(start) && range.contains(end)
}

/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let app_range = get_cur_app_range();
    debug!("app range: {:?}", app_range);
    let user_stack_ragne = get_user_stack_range();
    debug!("user stack range: {:?}", user_stack_ragne);

    let ptr = buf as usize;
    let ptr_end = unsafe { buf.add(len) } as usize;
    debug!("str range: {:?}", ptr..ptr_end);
    if !(in_range(&app_range, &ptr, &ptr_end) || in_range(&user_stack_ragne, &ptr, &ptr_end)) {
        println!("out of range");
        return -1;
    }
    match fd {
        FD_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        },
        _ => -1,
    }
}
