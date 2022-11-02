use crate::kernel::processes::{Process, ProcessState};
use crate::kernel::{BitVec, BitVector, SysCallType, Ticks};

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
    // Il fatto di usare Option<&P> implica una dimensione di una singola word dei campi running e next.
    // Questo si deve riflettere nell'assembly, usando i giusti offset.
    pub(crate) running: Option<&'p dyn Process>,
    pub(crate) next: Option<&'p dyn Process>,
    /* !!! --------------------- !!! */
    pub(crate) sys_call: SysCallType,
    processes: [Option<&'p dyn Process>; 32],
    schedulable: BitVec,
    ticks: Ticks,
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
            schedulable: 0,
            ticks: 0,
        }
    }
}

impl<'p> Scheduler<'p> for Preemptive<'p> {
    fn start(&mut self) -> ! {
        /* Scheduling first process */
        let id = self.schedulable.first_set().unwrap();
        self.running = self.processes[id];

        unsafe {
            crate::kernel::armv7em_arch::load_first_process();
            /* Qui non dovremmo mai arrivare, in quanto la CPU è sotto controllo dello scheduler */
        }
    }

    fn process_idle(&mut self, prio: usize) {
        if let Some(pcb) = self.processes[prio] {
            pcb.set_state(ProcessState::Idle);
            self.schedule_next();
        }
    }

    fn process_stop(&mut self, prio: usize) {
        if let Some(pcb) = self.processes[prio] {
            pcb.set_state(ProcessState::Stopped);
            self.schedule_next();
        }
    }

    /// I ticks di sleeping di un task non rappresentano i tick rimanenti alla scadenza,
    /// ma il valore assoluto che il sistema deve raggiungere per riattivare il processo.
    /// Questo elimina tutte le operazioni di sottrazione a tutti i contatori dei ticks di
    /// tutti i processi.
    fn process_sleep(&mut self, prio: usize, ticks: Ticks) {
        if let Some(pcb) = self.processes[prio] {
            pcb.set_state(ProcessState::Sleeping(ticks));
            self.schedule_next();
        }
    }

    fn inc_system_ticks(&mut self) {
        self.ticks = self.ticks + 1;

        // Loop su tutti gli elementi non-null per decrementarne i ticks
        for maybe_task in self.processes {
            if let Some(task) = maybe_task {
                task.decrement_ticks();
                if let ProcessState::Idle = task.get_state() {
                    self.schedulable.set(task.prio() as usize);
                }
            }
        }
    }

    /// Questa funzione è eseguita nell'interrupt SysTick, per ricercare il prossimo task da avviare.
    /// Se c'è un nuovo task la funzione triggera l'interrupt di PendSV, dove avviene lo switch.
    /// Altrimenti lancia l'idle task, che mette in sleep la CPU
    fn schedule_next(&mut self) {
        let mut runnable = self.schedulable;

        while let Ok(id) = runnable.first_set() {
            match self.processes[id].unwrap().get_state() {
                ProcessState::Idle | ProcessState::Running => {
                    self.next = self.processes[id];
                    cortex_m::peripheral::SCB::set_pendsv();
                    return;
                }
                _ => (),
            }
            runnable.clear(id);
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
