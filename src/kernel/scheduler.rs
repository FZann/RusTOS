use crate::kernel::processes::{Process, ProcessState, PCB};
use crate::kernel::{BooleanVector, Ticks};

use super::SysCallType;

#[no_mangle]
pub static mut SCHEDULER: Preemptive = Preemptive::new();

pub trait Scheduler {
    fn start(&mut self) -> !;

    fn process_idle(&mut self, prio: u8);
    fn process_stop(&mut self, prio: u8);
    fn process_sleep(&mut self, prio: u8, ticks: Ticks);

    fn find_next_ready(&self) -> Option<&PCB>;

    fn inc_system_ticks(&mut self);
    fn run_next(&mut self);
    fn add_process(&mut self, process: PCB) -> Result<(), ()>;
    fn remove_process(&mut self, prio: u8) -> Result<(), ()>;
}
/// Lo Scheduler tiene in memoria anche le variabili che servono per completare
/// un context switch. In questo modo evito di usare una serie di unsafe per
/// la modifica dei valori, perché non risultano statici allo scheduler stesso
#[repr(C)]
pub struct Preemptive {
    /* !!! --------------------- !!! */
    // L'accesso a queste variabili avviene anche via assembly! Non modificare la dichiarazione!
    pub(crate) process_running: Option<*const PCB>,
    pub(crate) process_next: Option<*const PCB>,
    /* !!! --------------------- !!! */
    pub(crate) sys_call: SysCallType,
    processes: [Option<PCB>; 32],
    active: BooleanVector,
    ticks: Ticks,
}

unsafe impl Sync for Preemptive {}

impl Preemptive {
    const NONE: Option<PCB> = None;

    pub const fn new() -> Self {
        Self {
            process_running: None,
            process_next: None,
            sys_call: SysCallType::Nop,
            processes: [Self::NONE; 32],
            active: BooleanVector::new(),
            ticks: 0,
        }
    }
}

impl Preemptive {
    /// Looppa su tutti gli indirizzi della lista, lanciando la funzione f su tutti gli elementi non-null.
    /// Con questa implementazione eseguiamo il minor numero possibile di iterazioni.
    pub fn foreach(&self, f: impl Fn(&PCB)) {
        let tasks = self.active.clone();
        while let Ok(id) = tasks.find_first_set() {
            self.processes[id].as_ref().map(&f);
            tasks.clear(id as u8);
        }
    }
}

impl Scheduler for Preemptive {
    fn start(&mut self) -> ! {
        /* Scheduling first process */
        if let Some(pcb) = self.find_next_ready() {
            /* Qui si DEVE entrare */
            self.process_running = Some(pcb);
        }
        unsafe {
            crate::kernel::armv7em_arch::load_first_process();
            /* Qui non dovremmo mai arrivare, in quanto la CPU è sotto controllo dello scheduler */
        }
    }

    fn process_idle(&mut self, prio: u8) {
        if let Some(pcb) = &self.processes[prio as usize] {
            pcb.set_state(ProcessState::Idle);
            self.run_next();
        }
    }

    fn process_stop(&mut self, prio: u8) {
        if let Some(pcb) = &self.processes[prio as usize] {
            pcb.set_state(ProcessState::Stopped);
            self.run_next();
        }
    }

    /// I ticks di sleeping di un task non rappresentano i tick rimanenti alla scadenza,
    /// ma il valore assoluto che il sistema deve raggiungere per riattivare il processo.
    /// Questo elimina tutte le operazioni di sottrazione a tutti i contatori dei ticks di
    /// tutti i processi.
    fn process_sleep(&mut self, prio: u8, ticks: Ticks) {
        if let Some(pcb) = &self.processes[prio as usize] {
            pcb.set_state(ProcessState::Sleeping(ticks));
            self.run_next();
        }
    }

    fn find_next_ready(&self) -> Option<&PCB> {
        let runnable = self.active.clone();

        while let Ok(id) = runnable.find_first_set() {
            if let Some(next) = self.processes[id].as_ref() {
                match next.get_state() {
                    ProcessState::Idle | ProcessState::Running => {
                        return Some(next);
                    }
                    _ => (),
                }
            }
            runnable.clear(id as u8);
        }
        
        None
    }

    fn inc_system_ticks(&mut self) {
        self.ticks = self.ticks + 1;
        self.foreach(|pcb| {
            pcb.decrement_ticks();
        });
    }

    /// Questa funzione è eseguita nell'interrupt SysTick, per ricercare il prossimo task da avviare.
    /// Se c'è un nuovo task la funzione triggera l'interrupt di PendSV, dove avviene lo switch.
    /// Altrimenti lancia l'idle task, che mette in sleep la CPU
    fn run_next(&mut self) {
        if let Some(next) = self.find_next_ready() {
            self.process_next = Some(next);
            cortex_m::peripheral::SCB::set_pendsv();
        }
    }

    fn add_process(&mut self, process: PCB) -> Result<(), ()> {
        let prio = process.prio() as usize;

        match self.processes[prio] {
            Some(_) => Err(()),
            None => {
                self.processes[prio] = Some(process);
                self.active.set(prio as u8);
                Ok(())
            }
        }
    }

    fn remove_process(&mut self, prio: u8) -> Result<(), ()> {
        self.processes[prio as usize].take();
        self.active.clear(prio);
        Ok(())
    }
}
