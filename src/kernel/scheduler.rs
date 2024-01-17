use crate::kernel::processes::{Process, Task};
use crate::kernel::{BitVec, SysCallType, Ticks};
use crate::kernel::request_context_switch;

use super::SyncCell;

#[no_mangle]
//pub static mut SCHEDULER: Mutex<Preemptive> = Mutex::new(Preemptive::new());
//pub static SCHEDULER: Preemptive = Preemptive::new();
pub static mut SCHEDULER: Preemptive = Preemptive::new();
pub static mut IDLE_TASK: Task<40> = Task::new(super::idle_task, 200);

pub trait Scheduler<'p> {
    fn start(&mut self) -> !;

    fn running_id(&self) -> usize;
    fn running_idle(&mut self);
    fn running_stop(&mut self);
    fn running_sleep(&mut self, ticks: Ticks);

    fn process_idle(&mut self, prio: usize);
    fn process_stop(&mut self, prio: usize);
    fn process_sleep(&mut self, prio: usize, ticks: Ticks);

    fn inc_system_ticks(&mut self);
    fn schedule_next(&mut self);
    fn add_process(&mut self, process: &'p mut dyn Process) -> Result<(), ()>;
    fn remove_process(&mut self, prio: usize) -> Result<(), ()>;
}

/// Lo Scheduler tiene in memoria anche le variabili che servono per completare
/// un context switch. In questo modo evito di usare una serie di unsafe per
/// la modifica dei valori, perché non risultano statici allo scheduler stesso
#[repr(C)]
pub struct Preemptive<'p> {
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

unsafe impl<'p> Sync for Preemptive<'p> {}

impl<'p> Preemptive<'p> {
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
}

impl<'p> Scheduler<'p> for Preemptive<'p> {
    fn start(&mut self) -> ! {
        /* Scheduling first process */
        self.running = match self.schedulable.first_set() {
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

    fn running_idle(&mut self) {
        self.process_idle(self.running_id());
    }

    fn running_stop(&mut self) {
        self.process_stop(self.running_id());
    }

    fn running_sleep(&mut self, ticks: Ticks) {
        self.process_sleep(self.running_id(), ticks);
    }

    fn process_idle(&mut self, prio: usize) {
        if self.processes[prio].is_some() {
            self.schedulable.set(prio);
            self.sleeping.clear(prio);
            self.schedule_next();
        }
    }

    fn process_stop(&mut self, prio: usize) {
        if self.processes[prio].is_some() {
            self.schedulable.clear(prio);
            self.sleeping.clear(prio);
            self.schedule_next();
        }
    }

    fn process_sleep(&mut self, prio: usize, ticks: Ticks) {
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
    fn inc_system_ticks(&mut self) {
        let mut sleeping = self.sleeping.clone();
        while let Ok(id) = sleeping.first_set() {
            let task = self.processes[id].unwrap();
            if task.decrement_ticks() == 0 {
                self.schedulable.set(id);
                self.sleeping.clear(id);
            }
            sleeping.clear(id);
        }
    }

    /// Questa funzione è eseguita nell'interrupt SysTick, per ricercare il prossimo task da avviare.
    /// Se c'è un nuovo task la funzione triggera l'interrupt di PendSV, dove avviene lo switch.
    /// Altrimenti lancia l'idle task, che mette in sleep la CPU
    fn schedule_next(&mut self) {
        /* Con una singola clz troviamo subito il prossimo processo schedulabile */
        match (self.running_id(), self.schedulable.first_set()) {
            (run, Ok(next)) if run != next => {
                self.next = self.processes[next];
                unsafe { request_context_switch(); }
            }

            // Non c'è un task da schedulare!
            (_, Err(_)) => {
                self.next = unsafe { Some(&IDLE_TASK) };
                unsafe { request_context_switch(); }
            }
            // Entriamo in questa casistica se run.prio() == self.schedulable.first_set().id
            // Quindi usciamo senza fare nulla
            _ => {}
        }
    }

    fn add_process(&mut self, process: &'p mut dyn Process) -> Result<(), ()> {
        let prio = process.prio();

        match self.processes[prio] {
            Some(_) => Err(()),
            None => {
                process.setup();
                self.processes[prio] = Some(process);
                self.schedulable.set(prio);
                Ok(())
            }
        }
    }

    fn remove_process(&mut self, prio: usize) -> Result<(), ()> {
        self.processes[prio].take();
        self.schedulable.clear(prio);
        self.sleeping.clear(prio);
        Ok(())
    }
}
