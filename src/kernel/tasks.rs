use core::{cell::Cell, mem::transmute};

use crate::bitvec::BitVec;
use crate::kernel::{SysCallType, Ticks};
use crate::kernel::Syncable;


#[no_mangle]
//pub static mut SCHEDULER: Mutex<Preemptive> = Mutex::new(Preemptive::new());
pub static mut KERNEL: Kernel = Kernel::new();
//pub static mut SCHEDULER: Preemptive = Preemptive::new();
pub static mut IDLE_TASK: Task<40> = Task::new(super::idle_task, 200);

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
    processes: [Option<&'p dyn Process>; 32],
    schedulable: BitVec,
    sleeping: BitVec,
}

unsafe impl<'p> Sync for Kernel<'p> {}
impl<'p> Syncable for Kernel<'p> {}

impl<'p> Kernel<'p> {
    pub const fn new() -> Self {
        Self {
            sys_call: SysCallType::Nop,
            processes: [None; 32],
            schedulable: BitVec::new(),
            sleeping: BitVec::new(),
            running: None,
            next: None,
        }
    }

    pub fn start(&mut self) -> ! {
        /* Scheduling first process */
        self.running = match self.schedulable.find_first_set() {
            Ok(id) => self.processes[id],
            Err(_) => unsafe { Some(&IDLE_TASK) },
        };

        unsafe {
            IDLE_TASK.setup();
            crate::kernel::load_first_process();
            /* Qui non dovremmo mai arrivare, in quanto la CPU è sotto controllo dello scheduler */
        }
    }

    fn running_id(&self) -> usize {
        self.running.unwrap().prio()
    }

    pub(crate) fn running_idle(&mut self) {
        self.process_idle(self.running_id());
    }

    pub(crate) fn running_stop(&mut self) {
        self.process_stop(self.running_id());
    }

    pub(crate) fn running_sleep(&mut self, ticks: Ticks) {
        self.process_sleep(self.running_id(), ticks);
    }

    pub(crate) fn process_idle(&mut self, prio: usize) {
        if self.processes[prio].is_some() {
            self.schedulable.set(prio);
            self.sleeping.clear(prio);
            self.schedule_next();
        }
    }

    pub(crate) fn process_stop(&mut self, prio: usize) {
        if self.processes[prio].is_some() {
            self.schedulable.clear(prio);
            self.sleeping.clear(prio);
            self.schedule_next();
        }
    }

    pub(crate) fn process_sleep(&mut self, prio: usize, ticks: Ticks) {
        if let Some(pcb) = self.processes[prio] {
            self.schedulable.clear(prio);
            self.sleeping.set(prio);
            pcb.set_ticks(ticks);
            self.schedule_next();
        }
    }

    /// I tick di sleeping di un task vengono diminuiti ad ogni tick
    /// di sistema, fino all'azzeramento.
    /// A questo punto il task torna schedulabile.
    pub(crate) fn inc_system_ticks(&mut self) {
        for id in self.sleeping.into_iter() {
            let task = self.processes[id].unwrap();
            if task.decrement_ticks() == 0 {
                self.schedulable.set(id);
                self.sleeping.clear(id);
            }
        }
    }

    /// Questa funzione è eseguita nell'interrupt SysTick, per ricercare il prossimo task da avviare.
    /// Se c'è un nuovo task la funzione triggera l'interrupt di PendSV, dove avviene lo switch.
    /// Altrimenti lancia l'idle task, che mette in sleep la CPU
    pub(crate) fn schedule_next(&mut self) {
        /* Con una singola clz troviamo subito il prossimo processo schedulabile */
        match (self.running_id(), self.schedulable.find_first_set()) {
            (run, Ok(next)) if run != next => {
                self.next = self.processes[next];
                cortex_m::peripheral::SCB::set_pendsv();
            }

            // Non c'è un task da schedulare!
            (_, Err(_)) => {
                self.next = unsafe { Some(&IDLE_TASK) };
                cortex_m::peripheral::SCB::set_pendsv();
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
            self.schedulable.set(prio);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn remove_process(&mut self, prio: usize) -> Result<(), ()> {
        self.processes[prio].take();
        self.schedulable.clear(prio);
        self.sleeping.clear(prio);
        Ok(())
    }
}


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
        unsafe {
            KERNEL.cs(|sched| sched.process_idle(self.prio()));
        }
    }

    fn stop(&mut self) {
        unsafe {
            KERNEL.cs(|sched| sched.process_stop(self.prio()));
        }
    }

    fn sleep(&mut self, ticks: Ticks) {
        self.set_ticks(ticks);
        unsafe {
            KERNEL.cs(|sched| sched.process_sleep(self.prio(), ticks));
        }
    }
}
