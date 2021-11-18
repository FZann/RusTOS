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

impl From<TaskHandle> for usize {
    fn from(handle: TaskHandle) -> Self {
        handle.0 as usize
    }
}

pub struct Ticks(usize);

impl Ticks {
    pub const fn new(ticks: usize) -> Self {
        Ticks(ticks)
    }

    pub fn ticks(&self) -> usize {
        self.0
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn decrement(&mut self) {
        self.0 -= 1;
    }
}

pub struct Stack {
    stack: &'static mut [usize],
    sp: StackPointer,
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

    fn sleep(ticks: Ticks);
    fn stop();
}

/// **PCB**
///
/// Process Control Block per un dispositivo ARM Cortex-M4.
pub struct PCB {
    stack: Stack,
    task: TaskHandle,
    prio: u8,
    pub(crate) sleep: Ticks,
}

impl Process for PCB {
    fn new(task: TaskHandle, stack: &'static mut [usize], prio: u8) -> Self {
        Self {
            task,
            stack: Stack::new(stack, task),
            prio,
            sleep: Ticks(0),
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

    fn sleep(ticks: Ticks) {}

    fn stop() {}
}
