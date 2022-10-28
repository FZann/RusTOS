use core::cell::Cell;

use crate::kernel::{TaskHandle, Ticks};

/// Wrapper per avere la type-safety.
type StackPointer = &'static usize;

#[derive(Clone, Copy)]
pub enum ProcessState {
    Idle,
    Running,
    Stopped,
    Sleeping(Ticks),
    Waiting,
}

pub trait Process {
    fn new(task: TaskHandle, stack: &'static mut [usize], prio: u8) -> Self
    where
        Self: Sized;
    fn handle(&self) -> TaskHandle;
    fn prio(&self) -> u8;
    fn set_prio(&mut self, prio: u8);

    fn stack_pointer(&self) -> StackPointer;

    fn set_state(&self, state: ProcessState);
    fn get_state(&self) -> ProcessState;

    fn decrement_ticks(&self);
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
#[repr(C)]
pub struct PCB {
    /* !!! --------------------- !!! */
    // L'accesso a queste variabili avviene anche via assembly! Non modificare la dichiarazione!
    sp: StackPointer,
    /* !!! --------------------- !!! */

    stack: &'static [usize],
    task: TaskHandle,
    state: Cell<ProcessState>,
    prio: u8,
}

impl Process for PCB {
    fn new(task: TaskHandle, stack: &'static mut [usize], prio: u8) -> Self {
        let len = stack.len();

        // Precarichiamo la stack con l'handle del Task
        // Hardware stack; necessaria
        stack[len - 01] = 1 << 24; // xPSR - Thumb state attivo
        stack[len - 02] = task as usize; // PC
        stack[len - 03] = 0xFFFFFFFD; // LR
        stack[len - 04] = 0xC; // R12
        stack[len - 05] = 0x3; // R3
        stack[len - 06] = 0x2; // R2
        stack[len - 07] = 0x1; // R1

        // Software stack; non è strettamente necessaria, serve più per debug
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
        let sp: StackPointer = &stack[len - 16];
        Self {
            sp,
            task,
            stack,
            prio,
            state: Cell::new(ProcessState::Idle),
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
