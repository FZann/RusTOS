use crate::kernel::Ticks;
use crate::kernel::{SysCallType, SystemCall};

/// Wrapper per avere la type-safety.
#[derive(Clone, Copy)]
pub struct StackPointer(*const usize);

impl StackPointer {
    pub const fn new() -> Self {
        Self(0 as *const usize)
    }
}

impl From<*const usize> for StackPointer {
    fn from(sp: *const usize) -> Self {
        StackPointer(sp)
    }
}

impl From<usize> for StackPointer {
    fn from(sp: usize) -> Self {
        StackPointer(sp as *const usize)
    }
}

unsafe impl Sync for StackPointer {}
unsafe impl Send for StackPointer {}

#[derive(Clone, Copy)]
pub struct TaskHandle(fn() -> !);

impl TaskHandle {
    pub fn new(task: fn() -> !) -> Self {
        Self(task)
    }
}

impl From<TaskHandle> for usize {
    fn from(handle: TaskHandle) -> Self {
        handle.0 as usize
    }
}

#[repr(C)]
pub struct Stack {
    sp: StackPointer,
    stack: &'static mut [usize],
}

impl Stack {
    pub const fn allocate<const LEN: usize>() -> [usize; LEN] {
        [0usize; LEN]
    }

    pub fn new(stack: &'static mut [usize], handle: TaskHandle) -> Self {
        let len = stack.len();

        // Precarichiamo la stack con l'handle del Task
        // Hardware stack
        stack[len - 01] = 1 << 24; // xPSR - Thumb state attivo
        stack[len - 02] = handle.into(); // PC
        stack[len - 03] = 0xFFFFFFFD; // LR
        stack[len - 04] = 0xC; // R12
        stack[len - 05] = 0x3; // R3
        stack[len - 06] = 0x2; // R2
        stack[len - 07] = 0x1; // R1

        // Software stack
        stack[len - 09] = 0xB; // R11
        stack[len - 10] = 0xA; // R10
        stack[len - 11] = 0x9; // R9
        stack[len - 12] = 0x8; // R8
        stack[len - 13] = 0x7; // R7
        stack[len - 14] = 0x6; // R6
        stack[len - 15] = 0x5; // R5
        stack[len - 16] = 0x4; // R4

        // Calcolo dello stack usize
        // Dovremmo partire da *base* + len, ma avendo già caricato
        // una stack frame, dobbiamo sottrarre 16.
        let sp: StackPointer = unsafe { stack.as_ptr().add(len - 16) }.into();

        Self { stack, sp }
    }

    pub fn sp(&self) -> StackPointer {
        self.sp
    }

    pub fn sp_ref(&self) -> &StackPointer {
        &self.sp
    }
}

pub trait Process {
    fn new(task: TaskHandle, stack: &'static mut [usize], prio: u8) -> Self
    where
        Self: Sized;
    fn handle(&self) -> TaskHandle;
    fn prio(&self) -> u8;
    fn set_prio(&mut self, prio: u8);

    fn stack_pointer(&self) -> StackPointer;
    fn stack_ptr_ref(&self) -> &StackPointer;

    fn set_ticks(&self, ticks: Ticks);
    fn get_ticks(&self) -> Ticks;

    fn idle() {
        SystemCall(SysCallType::ProcessIdle);
    }

    fn sleep(ticks: Ticks) {
        SystemCall(SysCallType::ProcessSleep(ticks));
    }

    fn stop() {
        SystemCall(SysCallType::ProcessStop);
    }
}

/// **PCB**
///
/// Process Control Block per un dispositivo ARM Cortex-M4.
/// In questo caso i ticks indicano i tick in valore assoluto.
/// Questo perché i valori di Sleep sono calcolati aggiungendo
/// il tempo di Sleep al valore di ticks attuali.
/// Questa tecnica mi permette di risparmiare il decremento
/// dei tick di sleeping e di effettuare unicamente il check
/// di verifica con l'if.

pub struct PCB {
    stack: Stack,
    task: TaskHandle,
    prio: u8,
    ticks: Ticks,
}

impl Process for PCB {
    fn new(task: TaskHandle, stack: &'static mut [usize], prio: u8) -> Self {
        Self {
            task,
            stack: Stack::new(stack, task),
            prio,
            ticks: Ticks::new(0),
        }
    }

    fn handle(&self) -> TaskHandle {
        self.task
    }

    fn prio(&self) -> u8 {
        self.prio
    }

    fn set_prio(&mut self, prio: u8) {
        self.prio = prio;
    }

    fn stack_pointer(&self) -> StackPointer {
        self.stack.sp()
    }
    fn stack_ptr_ref(&self) -> &StackPointer {
        self.stack.sp_ref()
    }

    fn set_ticks(&self, ticks: Ticks) {
        self.ticks.set(ticks.value());
    }

    fn get_ticks(&self) -> Ticks {
        self.ticks.clone()
    }
}
