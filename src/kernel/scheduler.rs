use crate::kernel::processes::PCB;

use super::processes::{Process, Ticks};

pub struct BooleanVector {
    vec: usize,
}

impl BooleanVector {
    pub const fn new() -> Self {
        BooleanVector { vec: 0 }
    }

    pub fn read(&self, bit: u8) -> bool {
        self.vec & (1 << bit) == (1 << bit)
    }

    pub fn set(&mut self, bit: u8) {
        self.vec |= 1 << bit
    }

    pub fn clear(&mut self, bit: u8) {
        self.vec &= !(1 << bit)
    }
}

impl From<BooleanVector> for usize {
    fn from(bv: BooleanVector) -> Self {
        bv.vec
    }
}

impl From<&BooleanVector> for usize {
    fn from(bv: &BooleanVector) -> Self {
        bv.vec
    }
}

/// Il suo scopo è quello di forzare un inc_system_ticks prima di chiamare run_next
pub struct IncToken;

pub trait Scheduler {
    fn start(&self) -> !;
    
    fn process_idle(&mut self, prio: u8);
    fn process_stop(&mut self, prio: u8);
    fn process_sleep(&mut self, prio: u8, ticks: Ticks);

    fn inc_system_ticks(&mut self) -> IncToken;
    fn run_next(&mut self, _token: IncToken);
    fn add_process(&mut self, process: PCB) -> Result<(), ()>;
    fn remove_process(&mut self, prio: u8) -> Result<(), ()>;
}

/// La struttura tiene insieme i processi e i loro stati correlati
/// In questo modo ho creato un pezzetto dello scheduler.
/// I processi non sono a conoscenza del loro stato (idle, sleep, run...),
/// ma lo scheduler sì, tramite questa struttura.
struct ProcessList {
    processes: [Option<PCB>; 32],
    paused: BooleanVector,
    running: BooleanVector,
    stopped: BooleanVector,
    sleeping: BooleanVector,
}

impl<'stack> ProcessList {
    /// Serve per poter usare la fn new() -> Self
    const NONE: Option<PCB> = None;

    pub const fn new() -> Self {
        Self {
            processes: [Self::NONE; 32],
            paused: BooleanVector::new(),
            running: BooleanVector::new(),
            stopped: BooleanVector::new(),
            sleeping: BooleanVector::new(),
        }
    }

    pub fn find_next_ready(&self) -> Option<&PCB> {
        /* Usando l'istruzione 'clz' in assembly, otteniamo direttamente l'indice del processo
           con il quale accedere all'array dei PCB. Con una singola istruzione otteniamo immediatamente
           ciò che ci serve!
         */ 
        let running = self.running.find_first_set();
        let next = self.paused.find_first_set();

        match (running, next) {
            (Ok(run_id), Ok(next_id)) if run_id <= next_id => self.processes[run_id].as_ref(),
            (Ok(run_id), Ok(next_id)) if run_id > next_id => self.processes[next_id].as_ref(),
            (Ok(run_id), Err(_)) => self.processes[run_id].as_ref(),
            (Err(_), Ok(next_id)) => self.processes[next_id].as_ref(),
            (Err(_), Err(_)) => None,
            _ => None,
        }
    }

    pub fn add(&mut self, process: PCB) -> Result<(), ()> {
        let prio: usize = process.prio() as usize;
        
        match self.processes[prio] {
            Some(_) => Err(()),
            None => {
                self.processes[prio] = Some(process);
                Ok(())
            }
        }

    }

    pub fn remove(&mut self, _prio: u8) -> Result<(), ()> {
        Ok(())
    }

    pub fn set_idle(&mut self, prio: u8) {
        self.paused.set(prio);

        self.running.clear(prio);
        self.stopped.clear(prio);
        self.sleeping.clear(prio);
    }

    pub fn set_stopped(&mut self, prio: u8) {
        self.stopped.set(prio);
        
        self.paused.clear(prio);
        self.running.clear(prio);
        self.sleeping.clear(prio);
    }

    pub fn set_sleeping(&mut self, prio: u8) {
        self.sleeping.set(prio);

        self.paused.clear(prio);
        self.running.clear(prio);
        self.stopped.clear(prio);
    }

    pub fn get_process_ref(&self, prio: usize) -> Option<&PCB> {
        self.processes[prio].as_ref()
    }
}

/// Lo Scheduler tiene in memoria anche le variabili che servono per completare
/// un context switch. In questo modo evito di usare una serie di unsafe per
/// la modifica dei valori, perché non risultano statici allo scheduler stesso
#[repr(C)]
pub struct Preemptive<'pcb> {
    /* !!! --------------------- !!! */
    // L'accesso a queste variabili avviene anche via assembly! Non modificare la dichiarazione!
    running_stack_ptr: usize,
    next_stack_ptr: usize,
    running_process: Option<&'pcb PCB>,
    /* !!! --------------------- !!! */

    list: ProcessList,
    ticks: Ticks,
}

impl<'stack> Preemptive<'stack> {
    pub const fn new() -> Self {
        Self {
            running_stack_ptr: 0,
            next_stack_ptr: 0,
            running_process: None,
            list: ProcessList::new(),
            ticks: Ticks::new(0),
        }
    }
}

impl<'stack> Scheduler for Preemptive<'stack> {
    fn start(&self) -> ! {

        
        
        
        
        loop {

        }
    }

    fn process_idle(&mut self, prio: u8) {
        self.list.set_idle(prio);
    }   

    fn process_stop(&mut self, prio: u8) {
        self.list.set_stopped(prio);
    }

    fn process_sleep(&mut self, prio: u8, ticks: Ticks) {
        self.list.set_sleeping(prio);
        self.list.get_process_ref(prio as usize).map(|pcb| pcb.sleep(ticks));
    }

    fn inc_system_ticks(&mut self) -> IncToken {
        self.ticks.increment();
        IncToken
    }

    /// Questa funzione è eseguita nell'interrupt SysTick, per ricercare il prossimo task da avviare.
    /// Al termine, la funzione triggera l'interrupt di PendSV, dove avviene lo switch.
    fn run_next(&mut self, _token: IncToken) {
        todo!();
    }

    fn add_process(&mut self, process: PCB) -> Result<(), ()> {
        self.list.add(process)?;
        Ok(())
    }

    fn remove_process(&mut self, _prio: u8) -> Result<(), ()> {
        todo!()
    }
}
