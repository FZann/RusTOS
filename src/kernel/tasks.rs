use core::marker::PhantomData;
use core::cell::Cell;

use crate::bitvec::BitVec;
use crate::kernel::{SysCallType, Ticks, CorePeripherals};

use crate::kernel::CritCell;

use super::{CritSect, SystemCall};


#[no_mangle]
//pub static mut SCHEDULER: Mutex<Preemptive> = Mutex::new(Preemptive::new());
pub static KERNEL: CritCell<Kernel> = CritCell::new(Kernel::new());
//pub static mut SCHEDULER: Preemptive = Preemptive::new();
pub static mut IDLE_TASK: Task<32> = Task::new(super::idle_task, 200);

pub type TaskHandle = fn(&mut dyn Process) -> !;

/// Lo Scheduler tiene in memoria anche le variabili che servono per completare
/// un context switch. In questo modo evito di usare una serie di unsafe per
/// la modifica dei valori, perché non risultano statici allo scheduler stesso
#[repr(C)]
pub struct Kernel<'p> {
    /* !!! --------------------- !!! */
    // L'accesso a queste variabili avviene anche via assembly! Non modificare la dichiarazione!
    // Il fatto di usare &dyn Process implica una dimensione di due words dei campi running e next.
    // Questo si deve riflettere nell'assembly, usando i giusti offset.
    pub(crate) running: Option<&'p dyn Process>,
    pub(crate) next: Option<&'p dyn Process>,
    /* !!! --------------------- !!! */
    pub(crate) sys_call: SysCallType,

    /// Ticks totali da quando il sistema è partito
    ticks: u64,

    /// Periferiche del core
    core: CorePeripherals,  

    /// Lista processi
    processes: [Option<&'p dyn Process>; BitVec::BITS],
    ready: BitVec,
    sleeping: BitVec,
}

impl<'p> Kernel<'p> {
    pub const fn new() -> Self {
        Self {
            running: None,
            next: None,
            sys_call: SysCallType::Nop,
            ticks: 0,
            core: CorePeripherals::new(),
            processes: [None; BitVec::BITS],
            ready: BitVec::new(),
            sleeping: BitVec::new(),
        }
    }

    pub fn init(&self, cs: CritSect) -> ! {
        drop(cs);
        SystemCall(SysCallType::StartScheduler);
        unreachable!();
    }

    pub(crate) fn start(&mut self) -> ! {
        // Setup delle periferiche core per far girare l'OS
        self.core.setup_os();
        
        /* Scheduling first process */
        unsafe {
            self.running = Some(&IDLE_TASK);
            IDLE_TASK.setup();
            self.load_first_process();
            /* Qui non dovremmo mai arrivare, in quanto la CPU è sotto controllo dello scheduler */
        }
    }
    fn running(&self) -> &dyn Process {
        self.running.unwrap()
    }

    pub(crate) fn process_idle(&mut self, prio: usize) {
        self.ready.set(prio);
        self.sleeping.clear(prio);
        self.schedule_next();
    }

    pub(crate) fn process_stop(&mut self, prio: usize) {
        self.ready.clear(prio);
        self.sleeping.clear(prio);
        self.schedule_next();
    }

    pub(crate) fn process_sleep(&mut self, prio: usize, ticks: Ticks) {
        if let Some(pcb) = self.processes[prio] {
            self.ready.clear(prio);
            self.sleeping.set(prio);
            pcb.set_ticks(ticks);
            self.schedule_next();
        }
    }

    /// I tick di sleeping di un task vengono diminuiti ad ogni tick
    /// di sistema, fino all'azzeramento.
    /// A questo punto il task torna schedulabile.
    pub(crate) fn inc_system_ticks(&mut self) {
        self.ticks += 1;

        for id in self.sleeping.into_iter() {
            let task = self.processes[id].unwrap();
            if task.decrement_ticks() == 0 {
                self.ready.set(id);
                self.sleeping.clear(id);
            }
        }
    }

    /// Questa funzione è eseguita nell'interrupt SysTick, per ricercare il prossimo task da avviare.
    /// Se c'è un nuovo task la funzione triggera l'interrupt di PendSV, dove avviene lo switch.
    /// Altrimenti lancia l'idle task, che mette in sleep la CPU
    pub(crate) fn schedule_next(&mut self) {
        /* Con una singola clz troviamo subito il prossimo processo schedulabile */
        match (self.running().prio(), self.ready.find_first_set()) {
            (run, Ok(next)) if run != next => {
                self.next = self.processes[next];
                SystemCall(SysCallType::ContextSwith);
            }

            // Non c'è un task da schedulare!
            (_, Err(_)) => {
                // TODO: implementa lo sleep e rimuovi totalmente IDLE_TASK
                self.core.sleep_on_exit(true);
                self.next = unsafe { Some(&IDLE_TASK) };
                SystemCall(SysCallType::ContextSwith);
            }
            // Entriamo in questa casistica se run.prio() == self.schedulable.first_set().id
            // Quindi usciamo senza fare nulla
            _ => {}
        }
    }

    pub fn add_process(&mut self, process: &'p mut dyn Process) -> Result<(), ()> {
        let prio = process.prio();

        if let None = self.processes[prio] {
            process.setup();
            self.processes[prio] = Some(process);
            self.ready.set(prio);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn remove_process(&mut self, prio: usize) -> Result<(), ()> {
        self.processes[prio].take();
        self.ready.clear(prio);
        self.sleeping.clear(prio);
        Ok(())
    }
}


pub trait Process {
    fn setup(&mut self);
    fn set_ticks(&self, ticks: Ticks);
    fn decrement_ticks(&self) -> Ticks;

    fn prio(&self) -> usize;
    fn sp(&self) -> StackPointer;
    fn handle(&self) -> TaskHandle;

    fn idle(&mut self);
    fn stop(&mut self);
    fn sleep(&mut self, ticks: Ticks);
}

#[derive(Clone)]
#[repr(C)]
pub struct StackPointer<'sp> {
    ptr: usize,
    start: usize,
    watermark: usize,
    lifetime: PhantomData<&'sp usize>,
}

impl<'sp> StackPointer<'sp> {
    pub(crate) const fn new() -> Self {
        Self {
            ptr: 0,
            start: 0,
            watermark: 0,
            lifetime: PhantomData,
        }
    }

    pub(crate) fn update_watermark(&mut self) {
        let words = (self.start - self.ptr) >> 2;
        if words > self.watermark {
            self.watermark = words;
        }
    }
}

/// **PCB**
///
/// Process Control Block per un dispositivo ARM Cortex-M4.
#[repr(C)]
pub struct Task<'t, const WORDS: usize> {
    /* !!! --------------------- !!! */
    // L'accesso a queste variabili avviene anche via assembly! Non modificare la dichiarazione!
    sp: StackPointer<'t>,
    /* !!! --------------------- !!! */
    stack: [usize; WORDS],
    task: TaskHandle,
    ticks: Cell<Ticks>,
    prio: usize,
}

impl<'t, const WORDS: usize> Task<'t, WORDS> {
    pub const fn new(task: TaskHandle, prio: usize) -> Self {
        if WORDS < 32 {
            panic!("Stack troppo piccola!");
        };

        Self {
            sp: StackPointer::new(),
            stack: [0; WORDS],
            task,
            prio,
            ticks: Cell::new(0),
        }
    }
}

impl<'t, const WORDS: usize> Process for Task<'t, WORDS> {
    fn setup(&mut self) {
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

        let sp = &self.stack[WORDS - 16] as *const usize as usize;
        let start = &self.stack[WORDS - 01] as *const usize as usize;
        self.sp.ptr = sp;
        self.sp.start = start;
    }


    fn handle(&self) -> TaskHandle {
        self.task
    }

    fn prio(&self) -> usize {
        self.prio as usize
    }

    fn sp(&self) -> StackPointer {
        self.sp.clone()
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
        KERNEL.with(|k| k.process_idle(self.prio));
    }
    

    fn stop(&mut self) {
        KERNEL.with(|k| k.process_stop(self.prio));
    }

    fn sleep(&mut self, ticks: Ticks) {
        KERNEL.with(|k| k.process_sleep(self.prio, ticks));
    }
}
