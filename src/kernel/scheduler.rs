use crate::kernel::processes::{Process, ProcessState};
use crate::kernel::{BitVec, SysCallType, Ticks};

#[no_mangle]
pub static mut SCHEDULER: Preemptive = Preemptive::new();

pub trait Scheduler<'p> {
    fn start(&mut self) -> !;

    fn process_idle(&mut self, prio: usize);
    fn process_stop(&mut self, prio: usize);
    fn process_sleep(&mut self, prio: usize, ticks: Ticks);

    fn inc_system_ticks(&mut self);
    fn schedule_next(&mut self);
    fn add_process(&mut self, process: &'p dyn Process) -> Result<(), ()>;
    fn remove_process(&mut self, prio: usize) -> Result<(), ()>;
}

/// Lo Scheduler tiene in memoria anche le variabili che servono per completare
/// un context switch. In questo modo evito di usare una serie di unsafe per
/// la modifica dei valori, perché non risultano statici allo scheduler stesso
#[repr(C)]
pub struct Preemptive<'p> {
    /* !!! --------------------- !!! */
    // L'accesso a queste variabili avviene anche via assembly! Non modificare la dichiarazione!
    // Il fatto di usare Option<&dyn Process> implica una dimensione di due words dei campi running e next.
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
    const NONE: Option<&'p dyn Process> = None;

    pub const fn new() -> Self {
        Self {
            running: None,
            next: None,
            sys_call: SysCallType::Nop,
            processes: [Self::NONE; 32],
            schedulable: BitVec::new(),
            sleeping: BitVec::new(),
        }
    }
}

impl<'p> Scheduler<'p> for Preemptive<'p> {
    fn start(&mut self) -> ! {
        /* Scheduling first process */
        let id = self.schedulable.first_set().unwrap();
        self.running = self.processes[id];

        unsafe {
            crate::kernel::load_first_process();
            /* Qui non dovremmo mai arrivare, in quanto la CPU è sotto controllo dello scheduler */
        }
    }

    fn process_idle(&mut self, prio: usize) {
        if let Some(pcb) = self.processes[prio] {
            pcb.set_state(ProcessState::Idle);
            self.schedulable.set(prio);
            self.sleeping.clear(prio);
        }
    }

    fn process_stop(&mut self, prio: usize) {
        if let Some(pcb) = self.processes[prio] {
            pcb.set_state(ProcessState::Stopped);
            self.schedulable.clear(prio);
            self.sleeping.clear(prio);
        }
    }

    /// I tick di sleeping di un task vengono diminuiti ad ogni tick
    /// di sistema, fino all'azzeramento. 
    /// A questo punto il task torna schedulabile.
    fn process_sleep(&mut self, prio: usize, ticks: Ticks) {
        if let Some(pcb) = self.processes[prio] {
            pcb.set_state(ProcessState::Sleeping(ticks));
            self.schedulable.clear(prio);
            self.sleeping.set(prio);
        }
    }

    fn inc_system_ticks(&mut self) {
        let mut sleeping = self.sleeping;
        while let Ok(id) = sleeping.first_set() {
            let task = self.processes[id].unwrap();
            task.decrement_ticks();
            if let ProcessState::Idle = task.get_state() {
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
        match (self.schedulable.first_set(), self.running) {
            (Ok(id), Some(run)) if run.prio() != id as u8 => {
                self.next = self.processes[id];
                cortex_m::peripheral::SCB::set_pendsv();
            },
            _ => {
                // TODO: inserire lo sleep automatico, magari senza idle task
                panic!("CASINO ATROCE! Siamo senza idle task.");
                // crate::kernel::sleep_cpu();
            }
        }
    }

    fn add_process(&mut self, process: &'p dyn Process) -> Result<(), ()> {
        match (process.handle(), process.sp()) {
            (Some(_), Some(_)) => (),
            (_, _) => panic!("Processo non setuppato!"),
        };

        let prio = process.prio() as usize;

        match self.processes[prio] {
            Some(_) => Err(()),
            None => {
                self.processes[prio] = Some(process);
                self.schedulable.set(prio);
                Ok(())
            }
        }
    }

    fn remove_process(&mut self, prio: usize) -> Result<(), ()> {
        self.processes[prio].take();
        self.schedulable.clear(prio);
        Ok(())
    }
}
