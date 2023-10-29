//! Process management syscalls

use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE},
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, TASK_MANAGER, get_current_user_token, current_insert_framed_area, unmap_the_area, find_current_pagetable_pte,
    }, timer::{get_time_ms, get_time_us}, mm::{translated_byte_buffer, VirtAddr, VPNRange, MapPermission},
};



#[repr(C)]
#[derive(Debug)]

/// Time Value 
pub struct TimeVal {
    /// seconds
    pub sec: usize,
    /// micro seconds
    pub usec: usize,
}
const MAXVA:usize= 1 << (9 + 9 + 9 + 12 - 1);
/// Task information
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
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
/// create a taskinfo 
pub fn create_taskinfo(status:TaskStatus) ->TaskInfo{
    TaskInfo { status: status, syscall_times: [0;MAX_SYSCALL_NUM], time:0 }
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
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let current_time=get_time_us();
    let _dst_ptrs =translated_byte_buffer(
        get_current_user_token(),
        _ts as * const u8,
        core::mem::size_of::<TimeVal>());
    let ref  time_val=TimeVal{
        usec:current_time % 1_000_000,
        sec:current_time / 1_000_000,
    };
    let src_ptr= time_val as * const TimeVal;
    for (index,dst) in _dst_ptrs.into_iter().enumerate(){
        let unit_length=dst.len();
        unsafe{
            dst.copy_from_slice(core::slice::from_raw_parts(src_ptr.wrapping_byte_add(index*unit_length) as * const u8, unit_length));
        }
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info!");

    let _dst_ptrs= translated_byte_buffer(
        get_current_user_token(),
         _ti as *const u8, core::mem::size_of::<TaskInfo>());
    let ref taskinfo=TASK_MANAGER.get_current_taskinfo();
    let src_ptr=taskinfo as * const TaskInfo;
    for (index,dst) in _dst_ptrs.into_iter().enumerate(){
        let unit_length=dst.len();
        unsafe{
            dst.copy_from_slice(
                core::slice::from_raw_parts(src_ptr.wrapping_byte_add(index*unit_length) as * const u8, unit_length),
            );
        }
    }

    0
}

/// syscall mmap
// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    // the upper bits are not zero
    if (_port& !0x7)!=0{
        return -1;
    }
    // read,write,execute flags are all 0.

    if (_port & 0x7) ==0{
        return -1;
    }
    // start va is not alined
    if _start&(PAGE_SIZE-1)!=0{
        return -1;
    }
    // beyond the limit of max virtual address
    if _start>MAXVA{
        return -1;
    }
    let start_vpn=VirtAddr::from(_start).floor();
    let  end_vpn =VirtAddr::from(_start+_len).ceil();
    let vpns =VPNRange::new(start_vpn, end_vpn);
    for vpn in vpns{
        // the virtual page is already existing
        if let Some(pte)=find_current_pagetable_pte(vpn){
            if pte.is_valid(){
                return -1;
            }
        }
    }
    current_insert_framed_area(start_vpn.into(), end_vpn.into(),
    MapPermission::from_bits_truncate((_port<<1)as u8)|MapPermission::U);
    
    
    0
}

// YOUR JOB: Implement munmap.
/// syscall munmap implemention
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if _start>MAXVA{
        return -1;
    }
    let mut _end=_start+_len;
    if _end>MAXVA{
        _end=MAXVA;
    }

    let start_vpn=VirtAddr::from(_start).floor();
    let end_vpn= VirtAddr::from(_end).ceil();
    
    unmap_the_area(start_vpn, end_vpn)
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
