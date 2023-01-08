use core::cell::Cell;

use crate::kernel::Ticks;

pub type TaskHandle = Option<fn() -> !>;
type StackPointer<'sp> = Option<&'sp usize>;

#[derive(Clone, Copy)]
pub enum ProcessState {
    Idle,
    Running,
    Stopped,
    Sleeping(Ticks),
    Waiting,
}

pub trait Process {
    fn handle(&self) -> TaskHandle;
    fn prio(&self) -> u8;

    fn set_state(&self, state: ProcessState);
    fn get_state(&self) -> ProcessState;
    fn sp(&self) -> StackPointer;
    fn decrement_ticks(&self);
}

/// **PCB**
///
/// Process Control Block per un dispositivo ARM Cortex-M4.
#[repr(C)]
pub struct Task<'sp, const WORDS: usize> {
    /* !!! --------------------- !!! */
    // L'accesso a queste variabili avviene anche via assembly! Non modificare la dichiarazione!
    sp: StackPointer<'sp>,
    /* !!! --------------------- !!! */
    stack: [usize; WORDS],
    task: TaskHandle,
    state: Cell<ProcessState>,
    prio: u8,
}

impl<'sp, const WORDS: usize> Task<'sp, WORDS> {
    pub const fn allocate(prio: u8) -> Self {
        if WORDS <= 32 {
            panic!("Stack troppo piccola!");
        };

        Self {
            sp: None,
            stack: [0; WORDS],
            task: None,
            prio,
            state: Cell::new(ProcessState::Idle),
        }
    }

    pub fn setup(&'sp mut self, task: fn() -> !) -> &Self {
        self.task = Some(task);

        self.stack[WORDS - 01] = 1 << 24; // xPSR - Thumb state attivo
        self.stack[WORDS - 02] = task as usize; // PC
        self.stack[WORDS - 03] = 0xFFFFFFFD; // LR
        self.stack[WORDS - 04] = 0xC; // R12
        self.stack[WORDS - 05] = 0x3; // R3
        self.stack[WORDS - 06] = 0x2; // R2
        self.stack[WORDS - 07] = 0x1; // R1

        // Software stack; non è strettamente necessaria, serve più per debug
        self.stack[WORDS - 09] = 0xB; // R11
        self.stack[WORDS - 10] = 0xA; // R10
        self.stack[WORDS - 11] = 0x9; // R9
        self.stack[WORDS - 12] = 0x8; // R8
        self.stack[WORDS - 13] = 0x7; // R7
        self.stack[WORDS - 14] = 0x6; // R6
        self.stack[WORDS - 15] = 0x5; // R5
        self.stack[WORDS - 16] = 0x4; // R4

        self.sp = Some(&self.stack[WORDS - 16]);
        self
    }
}

impl<'sp, const WORDS: usize> Process for Task<'sp, WORDS> {
    fn handle(&self) -> TaskHandle {
        self.task
    }

    fn prio(&self) -> u8 {
        self.prio
    }

    fn sp(&self) -> StackPointer {
        self.sp
    }

    fn set_state(&self, state: ProcessState) {
        self.state.set(state);
    }

    fn get_state(&self) -> ProcessState {
        self.state.get()
    }

    fn decrement_ticks(&self) {
        if let ProcessState::Sleeping(ticks) = self.state.get() {
            let ticks = ticks - 1;
            if ticks == 0 {
                self.state.set(ProcessState::Idle);
            } else {
                self.state.set(ProcessState::Sleeping(ticks));
            }
        }
    }
}
