use crate::fs::inode::ROOT_INODE;
use crate::fs::{make_pipe, open_file, OpenFlags, Stat};
use crate::mm::{
    translated_byte_buffer, translated_refmut, translated_str, PageTable, UserBuffer, VirtAddr,
};
use crate::task::{self, current_task, current_user_token};
use alloc::sync::Arc;
use alloc::vec::Vec;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let mut inner = task.inner_exclusive_access();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}

pub fn sys_dup(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    let new_fd = inner.alloc_fd();
    inner.fd_table[new_fd] = Some(Arc::clone(inner.fd_table[fd].as_ref().unwrap()));
    new_fd as isize
}

pub fn sys_linkat(old_path: *const u8, new_path: *const u8) -> isize {
    let token = current_user_token();
    let old = translated_str(token, old_path);
    let new = translated_str(token, new_path);
    ROOT_INODE.linkat(&old, &new)
}

pub fn sys_unlinkat(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    ROOT_INODE.unlink(&path)
}

pub fn sys_fstat(fd: i32, st: *mut Stat) -> isize {
    let fd = fd as usize;

    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }

    let Some(fd) = &inner.fd_table[fd].clone() else {
        return -1;
    };
    drop(inner);
    let fd = fd.clone();
    let token = current_user_token();
    let st = translated_refmut(token, st);
    *st = fd.stat();

    0
}

pub fn sys_mail_read(buf: *mut u8, len: usize) -> isize {
    let len = if len > 256 { 256 } else { len };
    // println!("read: {}", len);

    let token = current_user_token();

    if -1 == fun_name(buf, len, token) {
        return -1;
    }
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if inner.mail.is_empty() {
        return -1;
    }
    if len == 0 {
        return 0;
    }
    let Some(frame) = inner.mail.pop_front() else {
        return -1;
    };

    let readed_buf = translated_byte_buffer(token, buf, len);
    let mut user_buf = UserBuffer::new(readed_buf).into_iter();

    let mut res = 0;
    for (i, &ele) in frame.iter().enumerate().take(len) {
        if let Some(b) = user_buf.next() {
            res = i + 1;
            unsafe { *b = ele };
        } else {
            break;
        }
    }

    // if res == 0 {
    //     return -1;
    // }
    res as _
}

pub fn sys_mail_write(pid: usize, buf: *mut u8, len: usize) -> isize {
    let len = if len > 256 { 256 } else { len };
    // println!("write: {}", len);

    let token = current_user_token();

    if -1 == fun_name(buf, len, token) {
        return -1;
    }

    let target = crate::task::get_task(pid).unwrap();
    let mut inner = target.inner_exclusive_access();
    let len1 = inner.mail.len();
    // println!("=== mail len: {}", len1);
    if len1 == 16 {
        return -1;
    }
    if len == 0 {
        return 0;
    }
    // println!("start get buffers");
    let buffers = translated_byte_buffer(token, buf, len);
    // println!("get buffers");
    let u_buf = UserBuffer::new(buffers).into_iter();
    let mut b = Vec::with_capacity(len);
    for ele in u_buf {
        unsafe {
            b.push(*ele);
        }
    }
    inner.mail.push_back(b);
    let len1 = inner.mail.len();
    // println!(">>> mail len: {}", len1);

    len as _
}

fn fun_name(buf: *mut u8, len: usize, token: usize) -> isize {
    let start = buf as usize;
    let end = start + len;
    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(end);
    let vpn = start_va.floor();
    let evpn = end_va.floor();
    let page_table = PageTable::from_token(token);
    if page_table.translate(vpn).is_none() || page_table.translate(evpn).is_none() {
        return -1;
    }
    0
}
