//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    mm::{translated_byte_buffer, VirtAddr},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_first_run_time,
        map_memory, query_syscall_counter, suspend_current_and_run_next, unmap_memory, TaskStatus,
    },
    timer::{get_time_ms, get_time_us},
};
use core::mem::size_of;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let tm = get_time_us();
    let tm = TimeVal {
        sec: tm / 1000000,
        usec: tm % 1000000,
    };
    const LEN: usize = size_of::<TimeVal>();
    let buffers = translated_byte_buffer(current_user_token(), ts as *const u8, LEN);
    unsafe {
        let data: [u8; LEN] = core::mem::transmute(tm);
        let mut idx = 0;
        for buf in buffers {
            for by in buf {
                *by = data[idx];
                idx += 1;
            }
        }
    }

    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let taskinfo = TaskInfo {
        status: TaskStatus::Running,
        syscall_times: query_syscall_counter(),
        time: get_time_ms() - get_first_run_time(),
    };
    const LEN: usize = size_of::<TaskInfo>();
    let buffers = translated_byte_buffer(current_user_token(), ti as *const u8, LEN);
    unsafe {
        let data: [u8; LEN] = core::mem::transmute(taskinfo);
        let mut idx = 0;
        for buf in buffers {
            for by in buf {
                *by = data[idx];
                idx += 1;
            }
        }
    }

    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if prot & 0x7 == 0 || prot & !0x7 != 0 {
        error!("invalid mmap prot");
        return -1;
    }
    if !VirtAddr::from(start).aligned() {
        error!("mmap request unaligned start");
        return -1;
    }
    map_memory(start, len, prot)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if !VirtAddr::from(start).aligned() {
        error!("munmap request unaligned start");
        return -1;
    }
    unmap_memory(start, len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
