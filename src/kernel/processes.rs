use core::cell::Cell;

use crate::kernel::Ticks;

use super::{SysCallType, SystemCall};

pub type TaskHandle = fn(&mut dyn Process) -> !;
type StackPointer = usize;

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
    fn wait(&mut self, wait: &dyn Waitable);
}

pub trait Waitable {
    
}

/// **PCB**
///
/// Process Control Block per un dispositivo ARM Cortex-M4.
#[repr(C)]
pub struct Task<const WORDS: usize> {
    /* !!! --------------------- !!! */
    // L'accesso a queste variabili avviene anche via assembly! Non modificare la dichiarazione!
    sp: StackPointer,
    
    /// Stack pointer iniziale. Serve per far ripartire il task in caso di errori
    sp_start: StackPointer,   

    /// Massimo riempimento della stack, rappresentato in numero di parole (sp_start - sp)
    sp_watermark: StackPointer,  
    /* !!! --------------------- !!! */

    stack: [usize; WORDS],
    task: TaskHandle,
    prio: usize,
    pub(crate) ticks: Cell<Ticks>,
}

impl<const WORDS: usize> Task<WORDS> {
    pub const fn new(task: TaskHandle, prio: usize) -> Self {
        if WORDS < 32 {
            panic!("Stack troppo piccola!");
        };

        Self {
            sp: 0,
            sp_start: 0,
            sp_watermark: 0,
            stack: [0; WORDS],
            task,
            prio,
            ticks: Cell::new(0),
        }
    }
}

impl<const WORDS: usize> Process for Task<WORDS> {
    fn setup(&mut self) {
        // Questo permette di avere una reference alla struttura "task" nella funzione stessa
        let pointer: [usize; 2] = unsafe { core::mem::transmute(self as &dyn Process) };

        self.stack[WORDS - 01] = 1 << 24; // xPSR - Thumb state attivo
        self.stack[WORDS - 02] = self.task as usize; // PC
        self.stack[WORDS - 03] = 0xFFFFFFFD; // LR
        self.stack[WORDS - 04] = 0xC; // R12
        self.stack[WORDS - 05] = 0x3; // R3
        self.stack[WORDS - 06] = 0x2; // R2
        self.stack[WORDS - 07] = pointer[1]; // R1
        self.stack[WORDS - 08] = pointer[0]; // R0

        // Software stack; non è strettamente necessaria, serve più per debug
        self.stack[WORDS - 09] = 0xB; // R11
        self.stack[WORDS - 10] = 0xA; // R10
        self.stack[WORDS - 11] = 0x9; // R9
        self.stack[WORDS - 12] = 0x8; // R8
        self.stack[WORDS - 13] = 0x7; // R7
        self.stack[WORDS - 14] = 0x6; // R6
        self.stack[WORDS - 15] = 0x5; // R5
        self.stack[WORDS - 16] = 0x4; // R4

        // Impostazione dello stack pointer, della stack iniziale e del watermark
        self.sp = &self.stack[WORDS - 16] as *const usize as usize;
        self.sp_start = &self.stack[WORDS - 01] as *const usize as usize;
        self.sp_watermark = 0;
    }

    #[inline(always)]
    fn handle(&self) -> TaskHandle {
        self.task
    }

    #[inline(always)]
    fn prio(&self) -> usize {
        self.prio
    }

    #[inline(always)]
    fn sp(&self) -> StackPointer {
        self.sp
    }

    #[inline(always)]
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
        SystemCall(SysCallType::ProcessIdle(self.prio));
    }

    fn stop(&mut self) {
        SystemCall(SysCallType::ProcessStop(self.prio));
    }

    fn sleep(&mut self, ticks: Ticks) {
        SystemCall(SysCallType::ProcessSleep(self.prio, ticks));
    }

    fn wait(&mut self, wait: &dyn Waitable) {
        todo!()
    }

}
