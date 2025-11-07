//! RusTOS - Rust Real Time Operating System 
//! Copyright (C) 2025 - Fabio Zanin - fabio.zanin93@outlook.com
//! 
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License.
//! 
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU General Public License for more details.
//! 
//! You should have received a copy of the GNU General Public License
//! along with this program.  If not, see <https://www.gnu.org/licenses/>.
//! 
//! ************************************************* KERNEL OF RusTOS ************************************************
//! 
//! This module is the heart of RusTOS, as it implements all foundamentals objects:
//! Timers, Tasks, Kernel, Semaphores, Rendezvous, Mutexes, Queues and Stream Buffers.
//! 
//! Scheduling logic is implemented by Kernel struct with a couple of functions that interacts
//! with an assembly code, specific for the CPU architecture which RusTOS will be run on.
//! 
//! Static declaration of things was used as a foundamental development logic: RusTOS will statically
//! allocate objects when is possible; dynamic allocation will be implemented, but will be used as
//! last resource.
//! 
//! *******************************************************************************************************************

mod arch;
use arch::core::CorePeripherals;
use arch::core::CpuContext;

#[cfg(has_fpu)]
#[cfg(feature = "fpu_enabled")]
use arch::core::FpuContext;

#[cfg(feature = "mpu_enabled")]
use arch::core::MpuContext;

use arch::core::ExceptionFrame;

pub mod time;
pub use time::*;
pub(crate) mod registers;

use crate::bitvec::AtomicBitVec;
use crate::bitvec::BitVec;

use core::cell::Cell;
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::panic::PanicInfo;
use core::ptr::NonNull;

//*********************************************************************************************************************
// TYPES DEFINITION
//*********************************************************************************************************************

pub type TaskFn = fn(&mut Task) -> !;
pub type SystemTicks = u64;
pub type Ticks = u32;

#[doc(hidden)]
#[derive(Clone, Copy)]
pub(crate) union Vector {
    pub handler: unsafe extern "C" fn(),
    pub reserved: usize,
}


#[derive(PartialEq, PartialOrd, Clone, Copy)]
#[repr(u8)]
pub(crate) enum SysCallID {
    Nop = 0,
    StartScheduler = 1,
    SetTaskIdle = 2,
    SetTaskSleep = 3,
    SetTaskStop = 4,
}

impl Into<SysCallID> for u32 {
    fn into(self) -> SysCallID {
        match self {
            1 => SysCallID::StartScheduler,
            2 => SysCallID::SetTaskIdle,
            3 => SysCallID::SetTaskSleep,
            4 => SysCallID::SetTaskStop,
            _ => SysCallID::Nop,
        }
    }
}

#[repr(C)]
#[derive(PartialEq, Eq)]
pub enum ExecContext {
    Privileged = 0,
    Process = 1,
    Error = 2,
}

impl From<usize> for ExecContext {
    fn from(value: usize) -> Self {
        match value {
            0 => ExecContext::Privileged,
            1 => ExecContext::Process,
            _ => ExecContext::Error,
        }
    }
}

impl ExecContext {
    pub fn is_privileged(&self) -> bool {
        *self == ExecContext::Privileged
    }

    pub fn is_process(&self) -> bool {
        *self == ExecContext::Process
    }
}


/// Token to start a Critical Section
/// Creating a CritSect disables all interrupts, meanwhile Drop methods re-enables them.
#[must_use]
pub struct CritSect;

impl CritSect {
    pub fn activate() -> Self {
        Kernel::interrupt_disable();
        CritSect
    }

    pub fn deactivate(self) {
        drop(self);
    }
}

impl Drop for CritSect {
    fn drop(&mut self) {
        Kernel::interrupt_enable();
    }
}


/// Abstraction to make Sync-safe shared globals.
/// This way we can have a mutable access to statics and use mutable APIs.
/// This is Sync-safe as we are running on a single-core system.
/// Disabling interrupts makes impossible to have a race-condition when modifing data.
#[repr(transparent)]
pub struct CriticalCell<T: Sized> {
    data: UnsafeCell<T>,
    phantom: Cell<PhantomData<T>>
}

unsafe impl<T: Sized> Sync for CriticalCell<T> {}

impl<T: Sized> CriticalCell<T> {
    pub const fn new(value: T) -> Self {
        Self {
            data: UnsafeCell::new(value),
            phantom: Cell::new(PhantomData),
        }
    }

    pub fn with(&self, f: impl FnOnce(CritSect, &mut T)) {
        let cs = CritSect::activate();
        unsafe {
            f(cs, &mut *self.data.get());
        };
    }

    pub const fn read(&self) -> &T {
        unsafe { & (*self.data.get()) }
    }

    pub const fn access(&self, _cs: &CritSect) -> &mut T {
        unsafe { &mut (*self.data.get()) }
    }

    pub const unsafe fn access_unsafe(&self) -> &mut T {
        &mut *self.data.get()
    }
}

impl<T: Sized + Default> CriticalCell<T> {
    pub fn default() -> Self {
        Self {
            data: UnsafeCell::new(T::default()),
            phantom: Cell::new(PhantomData),
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub(crate) struct NullablePtr<T: ?Sized> {
    ptr: Option<NonNull<T>>
}

impl<T: ?Sized> NullablePtr<T> {
    pub const fn null() -> Self {
        Self {
            ptr: None
        }
    }

    pub const fn new(ptr: &T) -> Self {
        Self {
            ptr: unsafe { Some(NonNull::new_unchecked(ptr as *const T as *mut T)) }
        }
    }

    #[inline]
    pub const fn set(&mut self, ptr: Option<&T>) {
        if let Some(ptr) = ptr {
            self.ptr = unsafe { Some(NonNull::new_unchecked(ptr as *const T as *mut T)) };
        } else {
            self.ptr = None;
        }
    }

    pub const fn replace(&mut self, val: Option<&T>) -> Option<&mut T> {
        if let Some(mut ptr) = self.ptr {
            self.set(val);
            unsafe { Some(ptr.as_mut()) }
        } else {
            self.set(val);
            None
        }
    }

    pub const fn take(&mut self) -> Option<&mut T> {
        self.replace(None)
    }

    pub const fn get(&self) -> Option<&T> {
        if let Some(ptr) = self.ptr {
            unsafe { Some(ptr.as_ref()) }
        } else {
            None
        }
    }

    pub const fn get_mut(&mut self) -> Option<&mut T> {
        if let Some(mut ptr) = self.ptr {
            unsafe { Some(ptr.as_mut()) }
        } else {
            None
        }
    }

    pub const fn get_ptr(&self) -> Option<*const T> {
        if let Some(ptr) = self.ptr {
            Some(ptr.as_ptr())
        } else {
            None
        }
    }

    #[inline]
    pub const fn is_non_null(&self) -> bool {
        self.ptr.is_some()
    }

    #[inline]
    pub const fn is_null(&self) -> bool {
        self.ptr.is_none()
    }
}


//*********************************************************************************************************************
// STATICS AND CONSTANTS VARIABLES
//*********************************************************************************************************************
#[no_mangle]
pub static KERNEL: CriticalCell<Kernel> = CriticalCell::new(Kernel::new());

const IDLE_PRIO: usize = 255;
static IDLE_STACK: Stack::<32> = Stack::new();
pub static mut IDLE_TASK: Task = Task::new(idle_task, IDLE_PRIO, &IDLE_STACK);


//*********************************************************************************************************************
// OPERATING SYSTEM FUNCTIONS
//*********************************************************************************************************************

pub(crate) fn idle_task(_task: &mut Task) -> ! {
    loop {
        Kernel::core_sleep();
    }
}


#[panic_handler]
fn panic_handler(_: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
#[allow(non_snake_case)]
fn OSHardFault(_frame: &ExceptionFrame, running: &mut Task) {
    running.stop();

    let cs = CritSect::activate();
    KERNEL.access(&cs).schedule_next(cs);
}

#[no_mangle]
#[cfg(not(armv6m))]
#[allow(non_snake_case)]
fn OSMemoryFault(_frame: &ExceptionFrame, _running: &mut Task) {
    loop {

    }
}

#[no_mangle]
#[cfg(not(armv6m))]
#[allow(non_snake_case)]
fn OSBusFault(_frame: &ExceptionFrame, _running: &mut Task) {
    loop {

    }
}

#[no_mangle]
#[cfg(not(armv6m))]
#[allow(non_snake_case)]
fn OSUsageFault(_frame: &ExceptionFrame, _running: &mut Task) {
    loop {

    }
}

#[no_mangle]
#[cfg(armv8m)]
#[allow(non_snake_case)]
fn OSSecureFault(_frame: &ExceptionFrame, running: &mut Task) {
    loop {

    }
}

//*********************************************************************************************************************
// STACK AND TASK (TCB)
//*********************************************************************************************************************

/// This Cell struct is used to avoid compiler to put a Stack into FLASH memory, but
/// forcing it to keep Stack into RAM due to internal mutability (even with a zero-sized field).
pub struct Stack<const WORDS: usize> {
    buff: [usize; WORDS],
    ram_allocation: Cell<PhantomData<usize>>,
}

unsafe impl<const WORDS: usize> Sync for Stack<WORDS> {}

impl<const WORDS: usize> Stack<WORDS> {
    pub const fn new() -> Self {
        if WORDS < 32 {
            panic!("Stack too small!");
        }

        Self {
            buff: [0; WORDS],
            ram_allocation: Cell::new(PhantomData),
        }
    }

    pub const fn as_slice(&self) -> &[usize] {
        &self.buff
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Task {
    sp: usize,
    stack: *const [usize],
    stack_start: usize,
    stack_watermark: usize,

    task: TaskFn,
    prio: usize,
    semaphore: NullablePtr<Semaphore>,

    /// Forcing compiler to RAM-allocate this structure due to Cell presence
    ram_allocation: Cell<PhantomData<*const usize>>,

    context: CpuContext,

    #[cfg(feature = "fpu_enabled")]
    fpu: FpuContext,

    #[cfg(feature = "mpu_enabled")]
    mpu: MpuContext,
}

/// Here as all is static and no one should modify ''stack: *const [usize]'' field
unsafe impl Sync for Task {}

impl Task {
    pub const fn new<const WORDS: usize>(task: TaskFn, prio: usize, stack: &Stack<WORDS>) -> Self {
        if prio != IDLE_PRIO && prio > BitVec::HIGHEST_BIT {
            panic!("Priority too high!");
        }

        Self {
            sp: 0,
            stack: stack.as_slice(),
            stack_start: 0,
            stack_watermark: 0,

            task,
            prio,
            semaphore: NullablePtr::null(),
            
            ram_allocation: Cell::new(PhantomData),
            
            context: CpuContext::new(),
            //fpu: FpuContext::new(),
            //mpu: MpuContext::new(),
        }
    }

    pub(crate) fn update_watermark(&mut self) {
        let words = (self.stack_start - self.sp) >> 2;
        if words > self.stack_watermark {
            self.stack_watermark = words;
        }
    }

    pub const fn handle(&self) -> TaskFn {
        self.task
    }

    pub const fn prio(&self) -> usize {
        self.prio
    }

    pub fn idle(&mut self) {
        KERNEL.with(|cs, k| {
                k.tasks.idle(self.prio);
                k.schedule_next(cs);
            }
        );
    }

    pub fn stop(&mut self) {
        KERNEL.with(|cs, k| {
                k.tasks.stop(self.prio);
                k.schedule_next(cs);
            }
        );
    }

    pub fn sleep(&mut self, ticks: Ticks) {
        KERNEL.with(|cs, k| {
                k.tasks.sleep(self.prio, ticks);
                k.schedule_next(cs);
            }
        );
    }
}


struct TaskList {
    list: [MaybeUninit<*const Task>; BitVec::BITS],
    sleep_time: [Ticks; BitVec::BITS],
    used: BitVec,
    ready: BitVec,
    sleeping: BitVec,
}

impl TaskList {
    pub const fn new() -> Self {
        Self {
            list: [const { MaybeUninit::zeroed() }; BitVec::BITS],
            sleep_time: [0; BitVec::BITS],
            used: BitVec::new(),
            ready: BitVec::new(),
            sleeping: BitVec::new(),
        }
    }

    const fn add_task(&mut self, task: &Task) -> Result<(), ()> {
        if self.used.check(task.prio) == true {
            return Err(());
        }

        self.list[task.prio] = MaybeUninit::new(task);
        self.used.set(task.prio);
        self.ready.set(task.prio);

        Ok(())
    }

    const fn remove_task(&mut self, task: &Task) -> Result<(), ()> {
        if self.used.check(task.prio) == false {
            return Err(());
        }

        self.list[task.prio] = MaybeUninit::zeroed();
        self.used.clear(task.prio);
        self.ready.clear(task.prio);
        self.sleeping.clear(task.prio);

        Ok(())
    }

    #[inline]
    const fn get_ref_mut(&mut self, prio: usize) -> &mut Task {
        unsafe { &mut *(self.list[prio].assume_init_read() as *mut Task) }
    }

    #[inline]
    const fn get_ref(&self, prio: usize) -> &Task {
        unsafe { &*self.list[prio].assume_init_read() }
    }

    #[inline]
    const fn next_waiting(&self) -> Result<usize, ()>  {
        self.ready.find_highest_set()
    }

    #[inline]
    const fn idle(&mut self, prio: usize) {
        self.ready.set(prio);
        self.sleeping.clear(prio);
    }

    #[inline]
    const fn stop(&mut self, prio: usize) {
        self.ready.clear(prio);
        self.sleeping.clear(prio);
    }

    #[inline]
    const fn sleep(&mut self, prio: usize, ticks: Ticks) {
        self.sleep_time[prio] = ticks;
        self.ready.clear(prio);
        self.sleeping.set(prio);
    }

    fn setup(&mut self) {
        // Setup of all inserted tasks
        for prio in self.used.into_iter() {
            self.get_ref_mut(prio).setup();
        }
    }

    fn tick_sleeping(&mut self) {
        // NOTE: could we use SIMD here?
        for id in self.sleeping.into_iter() {
            self.sleep_time[id] -= 1;
            if self.sleep_time[id] == 0 {
                self.ready.set(id);
                self.sleeping.clear(id);

                if let Some(smph) = self.get_ref_mut(id).semaphore.take() {
                    smph.locked.clear(id);
                }
            }
        }
    }
}


//*********************************************************************************************************************
// SOFTWARE TIMERS
//*********************************************************************************************************************

#[cfg(feature = "timers")]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimerMode {
    OneShot,
    Looping,
    #[cfg(feature = "timers_bursts")]
    LoopingBurst(Ticks, u8),
    Counted(u8),
    #[cfg(feature = "timers_bursts")]
    CountedBurst(u8, Ticks, u8),
    Expired,
}

#[cfg(feature = "timers")]
pub struct Timer {
    id: u8,
    callback: fn(),
    mode: TimerMode,
    period: Duration,
    cntdwn: Ticks,
    #[cfg(feature = "timers_bursts")]
    bursts_cnt: u8,
}

#[cfg(feature = "timers")]
impl Timer {
    const fn new(id: u8, period: Duration, callback: fn()) -> Self {
        Self {
            id,
            callback,
            mode : TimerMode::Expired,
            period,
            cntdwn: period.ticks(),
            #[cfg(feature = "timers_bursts")]
            bursts_cnt: 0,
        }
    }

    pub fn start(&self) {
        let cs = CritSect::activate();
        KERNEL.access(&cs).timers.active.set(self.id as usize);
    }

    pub fn stop(&self) {
        let cs = CritSect::activate();
        KERNEL.access(&cs).timers.active.clear(self.id as usize);
    }

    pub const fn set_mode(&mut self, mode: TimerMode) {
        // Set mode and reloads timer countdown
        self.mode = mode;
        self.cntdwn = self.period.ticks();

        #[cfg(feature = "timers_bursts")]
        match mode {
            TimerMode::CountedBurst(_, _, bursts) => self.bursts_cnt = bursts,
            TimerMode::LoopingBurst(_, bursts) => self.bursts_cnt = bursts,
            _ => (),
        }
    }

    fn tick_and_fire(&mut self) -> TimerMode {
        self.cntdwn -= 1;
        
        if self.cntdwn == 0 {
            self.fire();
        }

        self.mode
    }

    #[inline]
    fn fire(&mut self) {
        match self.mode {
            TimerMode::OneShot => {
                (self.callback)();
                self.mode = TimerMode::Expired;
            }

            TimerMode::Looping => {
                (self.callback)();
                self.cntdwn = self.period.ticks();
            }

            #[cfg(feature = "timers_bursts")]
            TimerMode::LoopingBurst(ticks, bursts) => {
                (self.callback)();
                self.bursts_cnt -= 1;

                if self.bursts_cnt == 0 {
                    // Burst exausted. Reload burst count and reload long period
                    self.cntdwn = self.period.ticks();
                    self.bursts_cnt = bursts;
                } else {
                    // Bursts ongoing: load burst tick count 
                    self.cntdwn = ticks;
                }
            }

            TimerMode::Counted(cnt) if cnt > 0 => {
                (self.callback)();
                self.mode = TimerMode::Counted(cnt - 1);
                self.cntdwn = self.period.ticks();
            }

            #[cfg(feature = "timers_bursts")]
            TimerMode::CountedBurst(cnt, ticks, bursts) if cnt > 0 => {
                (self.callback)();
                self.bursts_cnt -= 1;

                if self.bursts_cnt == 0 {
                    // Burst exausted. Decrement rep count, reload burst count and reload long period
                    self.mode = TimerMode::CountedBurst(cnt - 1, ticks, bursts);
                    self.cntdwn = self.period.ticks();
                    self.bursts_cnt = bursts;
                } else {
                    // Bursts ongoing: load burst tick count 
                    self.cntdwn = ticks;
                }
            }

            _ => self.mode = TimerMode::Expired,
        }
    }
}

#[cfg(feature = "timers")]
struct TimerList {
    list: [MaybeUninit<Timer>; BitVec::BITS],
    used: BitVec,
    active: BitVec,
}

#[cfg(feature = "timers")]
impl TimerList {
    const fn new() -> Self {
        Self {
            list: [const { MaybeUninit::zeroed() }; BitVec::BITS],
            used: BitVec::new(),
            active: BitVec::new(),
        }
    }

    #[inline]
    const fn get_timer(&mut self, slot: usize) -> &mut Timer {
        unsafe { &mut *self.list[slot].as_mut_ptr() }
    }

    fn add_timer(&mut self, period: Duration, callback: fn(), mode: TimerMode) -> Result<&Timer, ()> {
        let id = self.used.find_first_zero()?;
        let mut tim = Timer::new(id as u8, period, callback);
        tim.set_mode(mode);
        self.list[id] = MaybeUninit::new(tim);
        self.used.set(id);
        Ok(unsafe { self.list[id].assume_init_ref() })
    }

    fn remove_timer(&mut self, slot: usize) -> Result<(), ()> {
        if self.used.check(slot) == false {
            return Err(());
        }

        self.list[slot] = MaybeUninit::zeroed();
        self.used.clear(slot);
        Ok(())
    }

    fn tick_timers(&mut self) {
        for slot in self.active.into_iter() {
            let tim = self.get_timer(slot);
            if TimerMode::Expired == tim.tick_and_fire() {
                let id = tim.id as usize;
                self.active.clear(id);
                self.used.clear(id);
                self.list[id] = MaybeUninit::zeroed();
            }
        }
    }
}


//*********************************************************************************************************************
// KERNEL
//*********************************************************************************************************************

trait SysCall {
    fn start_scheduler(task: &Task) -> !;
    fn set_task_idle(task: &Task);
    fn set_task_sleep(task: &Task, ticks: Ticks);
    fn set_task_stop(task: &Task);
}

/// Scheduler keeps in memory variables used to complete context switching.
/// This way we avoid using a series of 'unsafe' to modify static global data.
pub struct Kernel {
    running: MaybeUninit<*const Task>,
    next: MaybeUninit<*const Task>,

    /// Total system ticks till system started
    ticks: SystemTicks,

    /// Core peripherals - depends on CPU HW
    core: CorePeripherals,

    /// Process list
    tasks: TaskList,

    /// Timers list
    #[cfg(feature = "timers")]
    timers: TimerList,
}

impl Kernel {
    pub const fn new() -> Self {
        Self {
            running: MaybeUninit::zeroed(),
            next: MaybeUninit::zeroed(),
            ticks: 0,
            core: CorePeripherals::new(),
            tasks: TaskList::new(),
            #[cfg(feature = "timers")]
            timers: TimerList::new(),
        }
    }

    #[inline(always)]
    pub fn init(&mut self, cs: CritSect) -> ! {
        // Setup of CPU core peripherals
        self.setup_clock();

        self.core.setup();
        self.tasks.setup();

        // Scheduling first process
        unsafe {
            // Sequence to avoid compiler warnings
            let idle = &raw mut IDLE_TASK;
            self.running.write(idle);
            (&mut *idle).setup();
            
            Kernel::start_scheduler(self.running());
        }
        // We should never arrive here, as CPU is under Scheluder control
    }

    #[inline]
    pub const fn add_task(&mut self, task: &'static Task) -> Result<(), ()> {
        self.tasks.add_task(task)
    }

    #[inline]
    pub const fn remove_task(&mut self, task: &'static Task) -> Result<(), ()> {
        self.tasks.remove_task(task)
    }

    #[cfg(feature = "timers")]
    #[inline]
    pub fn new_timer(&mut self, period: Duration, callback: fn(), active: bool, mode: TimerMode) -> Result<&Timer, ()> {
        let res = self.timers.add_timer(period, callback, mode);
        if let Ok(tim) = res {
            if active == true {
                tim.start();
            }
        }

        res
    }
    
    #[cfg(feature = "timers")]
    #[inline]
    pub fn remove_timer(&mut self, slot: usize) -> Result<(), ()> {
        self.timers.remove_timer(slot)
    }

    #[inline]
    const fn running(&self) -> &Task {
        unsafe { &*self.running.as_ptr().read() }
    }

    #[inline]
    const fn running_mut(&self) -> &mut Task {
        unsafe { &mut *(self.running.assume_init_read() as *mut Task) }
    }

    #[inline]
    const fn next(&self) -> &Task {
        unsafe { &*self.next.as_ptr().read() }
    }

    #[inline]
    pub(crate) fn inc_system_ticks(&mut self) {
        self.ticks += 1;
        self.tasks.tick_sleeping();

        #[cfg(feature = "timers")]
        self.timers.tick_timers();
    }

    pub(crate) fn schedule_next(&mut self, cs: CritSect) {
        match (self.running().prio, self.tasks.next_waiting()) {
            // New task to be scheduled
            (run, Ok(next)) if next != run => {
                self.next = MaybeUninit::new(self.tasks.get_ref(next));
                //self.core.sleep_on_exit(false);
                cs.deactivate();
                self.request_context_switch();
            }

            // Nothing to do: idle task. If we are already running Idle Task, simply exit
            (run, Err(())) if run != IDLE_PRIO => {

                // TODO: verify if sleep_on_exit is usable
                // Activate bit and make a SysCall::Sleep (!) to enter an handler to sleep!
                //self.core.sleep_on_exit(true);    

                self.next = MaybeUninit::new(&raw const IDLE_TASK);
                cs.deactivate();
                self.request_context_switch();
            }

            // Same task to execute
            _ => {}
        }
    }

    #[no_mangle]
    pub(crate) fn switch_to_next(&mut self) {
        unsafe {
            self.running().context.save();
            self.running_mut().update_watermark();
                    
            self.running = self.next;
            self.next = MaybeUninit::new(&raw const IDLE_TASK);
            self.running().context.load();
        }
    }
}

//*********************************************************************************************************************
// SEMAPHORES, RENDEZVOUS and MUTEXes
//*********************************************************************************************************************

#[derive(Debug)]
pub struct Semaphore {
    locked: AtomicBitVec,
}

impl Default for Semaphore {
    fn default() -> Self {
        Self::new()
    }
}

impl Semaphore {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBitVec::new(),
        }
    }

    pub fn acquire(&self, task: &Task) {
        let cs = CritSect::activate();
        self.acquire_cs(task, cs);
    }

    #[inline]
    fn acquire_cs(&self, task: &Task, cs: CritSect) {
        self.locked.set(task.prio);
        KERNEL.access(&cs).tasks.stop(task.prio);
        KERNEL.access(&cs).schedule_next(cs);
    }

    pub fn wait(&self, task: &mut Task, timeout: ms) -> Result<(), ()>{
        let cs = CritSect::activate();
        self.wait_cs(task, timeout, cs)
    }

    #[inline]
    fn wait_cs(&self, task: &mut Task, timeout: ms, cs: CritSect) -> Result<(), ()>{
        self.locked.set(task.prio);
        task.semaphore.set(Some(self));
        KERNEL.access(&cs).tasks.sleep(task.prio, timeout.into());
        KERNEL.access(&cs).schedule_next(cs);

        if KERNEL.read().tasks.sleep_time[task.prio] != 0 {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn release(&self) {
        let cs = CritSect::activate();
        self.release_cs(cs);
    }

    #[inline]
    fn release_cs(&self, cs: CritSect) {
        if let Ok(id) = self.locked.find_highest_set() {
            self.locked.clear(id);
            KERNEL.access(&cs).tasks.idle(id);
            KERNEL.access(&cs).schedule_next(cs);
        }
    }
}

pub struct Rendezvous {
    mask: AtomicBitVec,
    arrived: AtomicBitVec,
}

impl Rendezvous {
    pub const fn new(mask: u32) -> Self {
        Self {
            mask: AtomicBitVec::init(mask),
            arrived: AtomicBitVec::new()
        }
    }

    /// Functions should check which Tasks are waiting for a Rendezvous and unlock those
    /// that where left off this new mask... Rigth now we simply don't allow to modify
    /// the mask, using more of a "const setup" logic.
    /// Maybe in future we should implement all required logic to handle a mask change.
    /* pub */ fn set_mask(&self, mask: BitVec) {
        let cs = CritSect::activate();
        self.mask.write_raw(mask.raw());
        cs.deactivate();
    }

    pub fn meet(&self, task: &Task) {
        let cs = CritSect::activate();
        let id = task.prio;
        self.arrived.set(id);
        
        if self.arrived.superset_of(&self.mask) {
            KERNEL.access(&cs).tasks.ready |= self.arrived.raw().into();
            KERNEL.access(&cs).tasks.sleeping &= (!self.arrived.raw()).into();

            // All arrived, empty the BitVec
            self.arrived.reset();
            KERNEL.access(&cs).schedule_next(cs);
        } else {
            KERNEL.access(&cs).tasks.stop(id);
            KERNEL.access(&cs).schedule_next(cs);
        }
    }

}


pub struct Mutex<T> {
    locker: Cell<NullablePtr<Task>>,
    resource: UnsafeCell<T>,
    sem: Semaphore,
}

unsafe impl<T> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locker: Cell::new(NullablePtr::null()),
            resource: UnsafeCell::new(value),
            sem: Semaphore::new(),
        }
    }

    pub fn acquire(&self, task: &Task) -> &mut T {
        let locker = unsafe { &*self.locker.as_ptr() };
        if locker.is_non_null() {
            self.sem.acquire(task);
        }
        let cs = CritSect::activate();
        self.locker.set(NullablePtr::new(task));
        cs.deactivate();
        unsafe { &mut *self.resource.get() }
    }

    pub fn release(&self, task: &Task) {
        let cs = CritSect::activate();
        let locker = unsafe { &*self.locker.as_ptr() };
        if let Some(locked) = locker.get() {
            if locked.prio == task.prio {
                self.locker.set(NullablePtr::null());
                self.sem.release_cs(cs);
            }
        }
    }
}




//*********************************************************************************************************************
// DATASTREAM: QUEUES and STREAM BUFFERS
//*********************************************************************************************************************

/// Queue. Data is passed by-copy and not by-reference.
pub struct Queue<T: Sized + Copy, const SIZE: usize> {
    push: Semaphore,
    pop: Semaphore,
    head: Cell<usize>,
    tail: Cell<usize>,
    cnt: Cell<usize>,
    buff: [Cell<MaybeUninit<T>>; SIZE],

    #[cfg(feature = "buffers_watermark")]
    watermark: Cell<usize>,

    // Size = 20B/24B + T * SIZE
}

/// Implemented because of Critical Sections used inside all methods
/// Critical Sections disallow any IRQ from firing, thus granting
/// that no one could access internal fields meanwhile we have a
/// shared access to them.
/// THIS IS VALID ONLY IN ONE-CORE SYSTEMS!
unsafe impl<T, const SIZE: usize> Sync for Queue<T, SIZE>
where
    T: Sized + Copy  {}

impl<T, const SIZE: usize> Default for Queue<T, SIZE> 
where
    T: Sized + Copy 
{
    fn default() -> Self {
        Self::new()
    }
}


impl<T, const SIZE: usize> Queue<T, SIZE>
where
    T: Sized + Copy,
{
    pub const fn new() -> Self {
        Self {
            push: Semaphore::new(),
            pop: Semaphore::new(),
            head: Cell::new(0),
            tail: Cell::new(0),
            cnt: Cell::new(0),
            buff: [const { Cell::new(MaybeUninit::zeroed()) }; SIZE],

            #[cfg(feature = "buffers_watermark")]
            watermark: Cell::new(0),
        }
    }

    /// Adds an element to Queue, waiting till space is available
    pub fn push(&self, task: &Task, data: T) {
        while self.cnt.get() >= SIZE {
            self.push.acquire(task);
        }

        let cs = CritSect::activate();
        let mut end = self.head.get();
        self.buff[end].set(MaybeUninit::new(data));
        end += 1;
        self.cnt.update(|x| x + 1);
        #[cfg(feature = "buffers_watermark")]
        self.watermark.update(|w| w.max(self.cnt.get()));
        
        if end >= SIZE {
            end = 0;
        }
        self.head.set(end);

        self.pop.release_cs(cs);
    }

    /// Adds an element to Queue; if there is no space available, waits for indicated timeout
    pub fn push_timeout(&self, task: &mut Task, data: T, timeout: ms) -> Result<(), ()> {
        while self.cnt.get() >= SIZE {
            self.push.wait(task, timeout)?;
        }

        let cs = CritSect::activate();
        let mut end = self.head.get();
        self.buff[end].set(MaybeUninit::new(data));
        end += 1;
        self.cnt.update(|c| c + 1);
        #[cfg(feature = "buffers_watermark")]
        self.watermark.update(|w| w.max(self.cnt.get()));

        if end >= SIZE {
            end = 0;
        }
        self.head.set(end);

        self.pop.release_cs(cs); 
        Ok(())
    }

    /// Adds an element to Queue only if there is space available
    pub fn push_dropping(&self, data: T) -> Result<(), ()> {
        if self.cnt.get() >= SIZE {
            return Err(());
        }
        
        let cs = CritSect::activate();
        let mut end = self.head.get();
        self.buff[end].set(MaybeUninit::new(data));
        end += 1;
        self.cnt.update(|c| c + 1);
        #[cfg(feature = "buffers_watermark")]
        self.watermark.update(|w| w.max(self.cnt.get()));

        if end >= SIZE {
            end = 0;
        }
        self.head.set(end);

        self.pop.release_cs(cs);
        Ok(())
    }

    /// Takes an element from Queue, waiting till an element is available
    pub fn pop(&self, task: &Task) -> T {
        while self.cnt.get() == 0 {
            self.pop.acquire(task);
        }

        let cs = CritSect::activate();
        let mut start = self.tail.get();
        let res = unsafe { self.buff[start].get().assume_init() };
        start += 1;
        if start >= SIZE {
            start = 0;
        }
        self.tail.set(start);
        self.cnt.update(|c| c - 1);

        self.push.release_cs(cs);
        res
    }

    /// Takes an element from Queue; if there is no element available, waits for indicated timeout
    pub fn pop_timeout(&self, task: &mut Task, timeout: ms) -> Result<T, ()> {
        while self.cnt.get() == 0 {
            self.pop.wait(task, timeout)?;
        }

        let cs = CritSect::activate();
        let mut start = self.tail.get();
        let res = unsafe { self.buff[start].get().assume_init() };
        start += 1;
        if start >= SIZE {
            start = 0;
        }
        self.tail.set(start);
        self.cnt.update(|c| c - 1);

        self.push.release_cs(cs);
        Ok(res)
    }

    /// Takes an element from Queue only if there is an element available
    pub fn pop_available(&self) -> Option<T> {
        if self.cnt.get() == 0 {
            return None;
        }

        let cs = CritSect::activate();
        let mut start = self.tail.get();
        let res = unsafe { self.buff[start].get().assume_init() };
        start += 1;
        if start >= SIZE {
            start = 0;
        }
        self.tail.set(start);
        self.cnt.update(|c| c - 1);

        self.push.release_cs(cs);
        Some(res)
    }

    /// Get queued element count
    #[inline]
    pub fn count(&self) -> usize {
        self.cnt.get()
    }

    #[cfg(feature = "buffers_watermark")]
    /// Returns the maximum number of elements saved into Queue during its lifetime
    #[inline]
    pub fn watermark(&self) -> usize {
        self.watermark.get()
    }

    /// Empty the Queue, resetting it to zero
    #[inline]
    pub fn clear(&self) {
        let cs = CritSect::activate();
        self.tail.set(0);
        self.head.set(0);
        self.cnt.set(0);
        cs.deactivate();
    }
}

/// This structure is similar to a Queue, but it works with slices of data.
/// Stream Buffers have a special functionality trigger (TRG) level: this permit
/// to unlock reading tasks only when a TRG amount of objects has been written into
/// the stream, reducing context switching as required by your application.
/// NOTE: Remember that data is passed by-copy, and **NOT** by reference!!!
/// 
/// Stream Buffers are useful when receiving data from UARTs, SPIs, I2Cs...
/// You can add data from an ISR using '''write_dropping()''' function,
/// and you can get those data from the task with any "read" method you like.
pub struct StreamBuffer<T: Sized + Copy, const SIZE: usize, const TRG: usize> {
    write: Semaphore,
    read: Semaphore,
    head: Cell<usize>,
    tail: Cell<usize>,
    cnt: Cell<usize>,
    buff: [Cell<MaybeUninit<T>>; SIZE],

    #[cfg(feature = "buffers_watermark")]
    watermark: Cell<usize>,

    // Size = 20B/24B + T * SIZE
}

/// Implemented because of Critical Sections used inside all methods.
/// Critical Sections disallow any IRQ from firing, thus granting
/// that no one could access internal fields meanwhile we have a
/// shared access to them.
/// THIS IS VALID ONLY IN ONE-CORE SYSTEMS!
unsafe impl<T, const SIZE: usize, const TRG: usize> Sync for StreamBuffer<T, SIZE, TRG>
where
    T: Sized + Copy  {}


impl<T, const SIZE: usize, const TRG: usize> Default for StreamBuffer<T, SIZE, TRG> 
where
    T: Sized + Copy 
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const SIZE: usize, const TRG: usize> StreamBuffer<T, SIZE, TRG> 
where 
    T: Sized + Copy
{
    pub const fn new() -> Self {
        if TRG > SIZE {
            panic!("Stream Buffer 'trigger' higher than 'size'!!!");
        }

        Self {
            write: Semaphore::new(),
            read: Semaphore::new(),
            tail: Cell::new(0),
            head: Cell::new(0),
            cnt: Cell::new(0),
            buff: [const { Cell::new(MaybeUninit::zeroed()) }; SIZE],

            #[cfg(feature = "buffers_watermark")]
            watermark: Cell::new(0),
        }
    }

    /// Writes a slice of elements into Stream Buffer, blocking till write is fully completed.
    /// The function blocks the task when there is no space available.
    /// When number of elements written to the Stream Buffer is greater than TRG (trigger),
    /// a blocked reading task will be unlocked.
    pub fn write(&self, task: &Task, slice: &[T]) {
        let mut space;
        let mut to_write: usize = slice.len();

        while to_write != 0  {
            // Checks if same space is available
            while {
                space = SIZE - self.cnt.get();
                space == 0
            }  
            {
                self.write.acquire(task);
            }

            let cs = CritSect::activate();
            let writable = usize::min(space, to_write);
            let write = (SIZE - self.head.get()).min(writable);
            let wrapping = writable - write;

            for id in 0..write {
                self.buff[id + self.head.get()].set(MaybeUninit::new(slice[id]));
            }
            
            if wrapping != 0 {
                for id in 0..wrapping {
                    self.buff[id].set(MaybeUninit::new(slice[id + write]));
                }
                self.head.set(wrapping);
            } else {
                self.head.update(|h| h + writable);
            }
            self.cnt.update(|c| c + writable);
            #[cfg(feature = "buffers_watermark")]
            self.watermark.update(|w| w.max(self.cnt.get()));

            // Updates total written with actual writable bytes.
            // Maybe [slice] is bigger than [space], so we must 
            // start all over again to reach slice's end
            to_write -= writable;

            if self.cnt.get() >= TRG {
                self.read.release_cs(cs);
            } else {
                cs.deactivate();
            }
        }
    }

    /// Writes a slice of elements into Stream Buffer, blocking till write is fully completed.
    /// If there is no space available, waits for indicated timeout.
    /// The function blocks the task when there is no space available.
    /// When number of elements written to the Stream Buffer is greater than TRG (trigger),
    /// a blocked reading task will be unlocked.
    pub fn write_timeout(&self, task: &mut Task, slice: &[T], timeout: ms) -> Result<(), usize> {
        let mut space;
        let mut to_write: usize = slice.len();

        while to_write != 0  {
            // Checks if same space is available
            while {
                space = SIZE - self.cnt.get();
                space == 0
            }  
            {
                if let Err(()) = self.write.wait(task, timeout) {
                    return Err(slice.len() - to_write);
                }
            }

            let cs = CritSect::activate();
            let writable = usize::min(space, to_write);
            let write = (SIZE - self.head.get()).min(writable);
            let wrapping = writable - write;

            for id in 0..write {
                self.buff[id + self.head.get()].set(MaybeUninit::new(slice[id]));
            }
            
            if wrapping != 0 {
                for id in 0..wrapping {
                    self.buff[id].set(MaybeUninit::new(slice[id + write]));
                }
                self.head.set(wrapping);
            } else {
                self.head.update(|h| h + writable);
            }
            self.cnt.update(|c| c + writable);
            #[cfg(feature = "buffers_watermark")]
            self.watermark.update(|w| w.max(self.cnt.get()));

            // Updates total written with actual writable bytes.
            // Maybe [slice] is bigger than [space], so we must 
            // start all over again to reach slice's end
            to_write -= writable;

            if self.cnt.get() >= TRG {
                self.read.release_cs(cs);
            } else {
                cs.deactivate();
            }
        }
        Ok(())
    }

    /// Writes a slice of elements into Stream Buffer, writing only the elements that fit into free space available.
    /// When number of elements written to the Stream Buffer is greater than TRG (trigger),
    /// a blocked reading task will be unlocked.
    pub fn write_dropping(&self, slice: &[T]) -> Result<usize, usize> {
        let space = SIZE - self.cnt.get();

        // No space available, just returns
        if space == 0 {
            return Err(0);
        }
        
        let cs = CritSect::activate();
        let writable = usize::min(space, slice.len());
        let write = (SIZE - self.head.get()).min(writable);
        let wrapping = writable - write;

        for id in 0..write {
            self.buff[id + self.head.get()].set(MaybeUninit::new(slice[id]));
        }
        
        if wrapping != 0 {
            for id in 0..wrapping {
                self.buff[id].set(MaybeUninit::new(slice[id + write]));
            }
            self.head.set(wrapping);
        } else {
            self.head.update(|h| h + writable);
        }

        self.cnt.update(|c| c + writable);
        #[cfg(feature = "buffers_watermark")]
        self.watermark.update(|w| w.max(self.cnt.get()));

        if self.cnt.get() >= TRG {
            self.read.release_cs(cs);
        } else {
            cs.deactivate();
        }

        if writable == slice.len() {
            Ok(writable)
        } else {
            Err(writable)
        }
    }

    /// Reads requested number of elements from the Stream Buffer. It stops till 'slice' is full.
    /// It blocks when there are no elements to read.
    /// TRG has no effect in reading.
    pub fn read(&self, task: &Task, slice: &mut [T]) {
        let mut to_read: usize = slice.len();

        while to_read != 0  {
            while self.cnt.get() == 0 {
                self.read.acquire(task);
            }

            let cs = CritSect::activate();
            let readable = usize::min(to_read, self.cnt.get());
            let read = (SIZE - self.tail.get()).min(readable);
            let remainder = readable - read;
    
            for id in 0..read {
                slice[id] = unsafe { self.buff[id + self.tail.get()].get().assume_init() };
            }
            
            if remainder != 0 {
                for id in 0..remainder {
                    slice[id + read] = unsafe { self.buff[id].get().assume_init() };
                }
                self.tail.set(remainder);
            } else {
                self.tail.set(self.tail.get() + readable);
            }
    
            self.cnt.set(self.cnt.get() - readable);
            to_read -= readable;

            self.write.release_cs(cs);
        }
    }

    /// Reads available number of elements from the Stream Buffer.
    /// It returns when 'slice' is full or when there are no more elements into Strem Buffer.
    /// TRG has no effect in reading.
    pub fn read_available(&self, slice: &mut [T]) -> usize {
        let cs = CritSect::activate();
        if self.cnt.get() == 0 {
            return 0;
        }

        let readable = usize::min(slice.len(), self.cnt.get());
        let read = (SIZE - self.tail.get()).min(readable);
        let remainder = readable - read;

        for id in 0..read {
            slice[id] = unsafe { self.buff[id + self.tail.get()].get().assume_init() };
        }
        
        if remainder != 0 {
            for id in 0..remainder {
                slice[id + read] = unsafe { self.buff[id].get().assume_init() };
            }
            self.tail.set(remainder);
        } else {
            self.tail.set(self.tail.get() + readable);
        }

        self.cnt.set(self.cnt.get() - readable);
        
        self.write.release_cs(cs);
        readable
    }

    /// Reads available number of elements from the Stream Buffer.
    /// It returns when 'slice' is full or when timeout has expired.
    /// TRG has no effect in reading.    
    pub fn read_timeout(&self, task: &mut Task, slice: &mut [T], timeout: ms) -> Result<(), usize> {
        let mut to_read: usize = slice.len();

        while to_read != 0  {
            while self.cnt.get() == 0 {
                if let Err(()) = self.read.wait(task, timeout) {
                    return Err(slice.len() - to_read);
                }
            }

            let cs = CritSect::activate();
            let readable = usize::min(to_read, self.cnt.get());
            let read = (SIZE - self.tail.get()).min(readable);
            let remainder = readable - read;
    
            for id in 0..read {
                slice[id] = unsafe { self.buff[id + self.tail.get()].get().assume_init() };
            }
            
            if remainder != 0 {
                for id in 0..remainder {
                    slice[id + read] = unsafe { self.buff[id].get().assume_init() };
                }
                self.tail.set(remainder);
            } else {
                self.tail.set(self.tail.get() + readable);
            }
    
            self.cnt.set(self.cnt.get() - readable);
            to_read -= readable;

            self.write.release_cs(cs);
        }
        
        Ok(())
    }

    /// Returns the number of elements saved into Stream Buffer
    #[inline]
    pub fn count(&self) -> usize {
        self.cnt.get()
    }

    #[cfg(feature = "buffers_watermark")]
    /// Returns the maximum number of elements saved into Stream Buffer during its lifetime
    #[inline]
    pub fn watermark(&self) -> usize {
        self.watermark.get()
    }

    /// Empty the Stream Buffer, resetting it to zero
    #[inline]
    pub fn clear(&self) {
        let cs = CritSect::activate();
        self.tail.set(0);
        self.head.set(0);
        self.cnt.set(0);
        cs.deactivate();
    }
}