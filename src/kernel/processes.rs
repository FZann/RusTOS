use crate::kernel::Ticks;
use core::{cell::Cell, mem::transmute};

use super::{
    scheduler::{Scheduler, SCHEDULER},
    Syncable,
};

pub type TaskHandle = fn() -> !;
type StackPointer<'sp> = Option<&'sp usize>;

pub trait Process {
    fn setup(&mut self);
    fn prio(&self) -> usize;
    fn sp(&self) -> StackPointer;
    fn handle(&self) -> TaskHandle;

    fn set_ticks(&self, ticks: Ticks);
    fn decrement_ticks(&self) -> Ticks;

    fn idle(&mut self);
    fn stop(&mut self);
    fn sleep(&mut self, ticks: Ticks);
}

/// **PCB**
///
/// Process Control Block per un dispositivo ARM Cortex-M4.
#[repr(C)]
pub struct Task<'task, const WORDS: usize> {
    /* !!! --------------------- !!! */
    // L'accesso a queste variabili avviene anche via assembly! Non modificare la dichiarazione!
    sp: StackPointer<'task>,
    /* !!! --------------------- !!! */
    stack: [usize; WORDS],
    task: TaskHandle,
    ticks: Cell<Ticks>,
    prio: u8,
}

unsafe impl<'task, const WORDS: usize> Sync for Task<'task, WORDS> {}
impl<'task, const WORDS: usize> Syncable for Task<'task, WORDS> {}

impl<'task, const WORDS: usize> Task<'task, WORDS> {
    pub const fn new(task: TaskHandle, prio: u8) -> Self {
        if WORDS <= 32 {
            panic!("Stack troppo piccola!");
        };

        Self {
            sp: None,
            stack: [0; WORDS],
            task,
            prio,
            ticks: Cell::new(0),
        }
    }
}

impl<'task, const WORDS: usize> Process for Task<'task, WORDS> {
    fn setup(&mut self) {
        self.stack[WORDS - 01] = 1 << 24; // xPSR - Thumb state attivo
        self.stack[WORDS - 02] = self.task as usize; // PC
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

        let sp = &self.stack[WORDS - 16];
        unsafe {
            self.sp = Some(transmute(sp));
        }
    }

    fn handle(&self) -> TaskHandle {
        self.task
    }

    fn prio(&self) -> usize {
        self.prio as usize
    }

    fn sp(&self) -> StackPointer {
        self.sp
    }

    fn set_ticks(&self, ticks: Ticks) {
        self.ticks.set(ticks);
    }

    fn decrement_ticks(&self) -> Ticks {
        let ticks = self.ticks.get();
        if ticks > 0 {
            self.set_ticks(ticks - 1);
        }
        ticks
    }

    fn idle(&mut self) {
        SCHEDULER.cs(|sched| sched.process_idle(self.prio()));
    }

    fn stop(&mut self) {
        SCHEDULER.cs(|sched| sched.process_stop(self.prio()));
    }

    fn sleep(&mut self, ticks: Ticks) {
        self.set_ticks(ticks);
        SCHEDULER.cs(|sched| sched.process_sleep(self.prio(), ticks));
    }
}
