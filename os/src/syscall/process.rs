//! Process management syscalls
use core::usize;

use crate::{
    config::MAX_SYSCALL_NUM,
    task::{exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, TASK_MANAGER},
    timer::{get_time_us, get_time_ms},
};

/// Time Value 
#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    /// second
    pub sec: usize,
    /// micro sencond
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
#[derive(Copy,Clone)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}
/// create a new taskinfo 
pub fn create_taskinfo(status:TaskStatus) -> TaskInfo{
        TaskInfo { status: status, syscall_times: [0;MAX_SYSCALL_NUM], time: 0 }
}
/// implement  taskinfo
impl TaskInfo {
    /// change status
    pub fn change_status(&mut self,status:TaskStatus){
        self.status=status;
    }
    /// count syscall_times
    pub fn count_syscall_times(&mut self,syscall_number:usize,num:u32) {
        self.syscall_times[syscall_number]+=num;
    }
    /// update the  running time of task 
    pub fn update_time(&mut self,ctime:usize ) {
        self.time=get_time_ms()-ctime;
    }
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {

    
    trace!("kernel: sys_task_info");
    
    let taskinfo=TASK_MANAGER.get_current_taskinfo();
    unsafe {
        *_ti=TaskInfo{
            status: taskinfo.status,
            syscall_times:taskinfo.syscall_times.clone(),
            time: taskinfo.time,
        }
    }
    0
}
