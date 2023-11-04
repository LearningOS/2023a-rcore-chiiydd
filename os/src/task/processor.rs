//!Implementation of [`Processor`] and Intersection of control flow
//!
//! Here, the continuous operation of user apps in CPU is maintained,
//! the current running state of CPU is recorded,
//! and the replacement and transfer of control flow of different applications are executed.


use super::__switch;
use super::{fetch_task, TaskStatus};
use super::{TaskContext, TaskControlBlock};
use crate::mm::{PageTableEntry, VirtPageNum, VirtAddr, MapPermission, VPNRange};
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use lazy_static::*;

/// Processor management structure
pub struct Processor {
    ///The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,

    ///The basic control flow of each core, helping to select and switch process
    idle_task_cx: TaskContext,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }

    ///Get mutable reference to `idle_task_cx`
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }

    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }

    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            // release coming task_inner manually
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            warn!("no tasks available in run_tasks");
        }
    }
}

///  get current task create time 
pub fn get_current_task_create_time() ->usize{
    
    current_task().unwrap().inner_exclusive_access().create_time
}

/// modify the task priority
pub fn set_task_priority(priority:usize){
    let current=current_task().unwrap();
    current.inner_exclusive_access().set_task_priority(priority);
    // current.pid.0
}
/// modify the syscall times 
pub fn modify_syscall_times(syscall_id:usize){
    current_task().unwrap().inner_exclusive_access().count_syscall_times(syscall_id);
}
/// get the current task's syscall times 
pub fn get_current_task_syscall_times () ->[u32;500]{
    current_task().unwrap().inner_exclusive_access().syscall_times
}
/// get the current task status 
pub fn get_current_task_status() ->TaskStatus{
    current_task().unwrap().inner_exclusive_access().task_status
}
/// look up the current task pageable 
pub fn find_current_task_pagetable(vpn :VirtPageNum) ->Option<PageTableEntry>{
    current_task().unwrap().inner_exclusive_access().memory_set.translate(vpn)
}
/// insert a new area to current task 
pub fn current_insert_framed_area(start_va:VirtAddr,end_va:VirtAddr,permission:MapPermission){
    current_task().unwrap().inner_exclusive_access().memory_set.insert_framed_area(start_va, end_va, permission);
}
/// unmap the area 
pub fn unmap_the_area(_start:usize,_len:usize) ->isize{
    let start_vpn=VirtAddr::from(_start).floor();
    let end_vpn=VirtAddr::from(_start+_len).ceil();
    let current_task=current_task().unwrap();
    let mut inner=current_task.inner_exclusive_access();
    let vpns =VPNRange::new(start_vpn, end_vpn);
    for vpn in vpns{
        if let Some(pte)=inner.memory_set.translate(vpn){
            if !pte.is_valid(){
                return -1;
            }
        }else{
            return -1;
        }
    }
    inner.memory_set.unmap_at_once(start_vpn, end_vpn)

}


/// Get current task through take, leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

/// Get a copy of the current task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

/// Get the current user token(addr of page table)
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    task.get_user_token()
}

///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}
