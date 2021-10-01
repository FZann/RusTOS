use crate::kernel::processes::PCB;

use super::processes::Process;

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

pub trait Scheduler {
    fn start(&self) -> !;
    fn run_next(&mut self);
    fn add_process(&mut self, process: PCB) -> Result<(), ()>;
    fn remove_process(&mut self, prio: u8) -> Result<(), ()>;
}

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
}

/// Lo Scheduler tiene in memoria anche le variabili che servono per completare
/// un context switch. In questo modo evito di usare una serie di unsafe per
/// la modifica dei valori, perché non risultano statici allo scheduler stesso
#[repr(C)]
pub struct Preemptive<'pcb> {
    /// L'accesso a questa variabile avviene anche via assembly! Non modificare la dichiarazione!
    running_stack_ptr: usize,
    /// L'accesso a questa variabile avviene anche via assembly! Non modificare la dichiarazione!
    next_stack_ptr: usize,

    running_proccess: Option<&'pcb PCB>,
    list: ProcessList,
}

impl<'stack> Preemptive<'stack> {
    pub const fn new() -> Self {
        Self {
            running_stack_ptr: 0,
            next_stack_ptr: 0,
            running_proccess: None,
            list: ProcessList::new(),
        }
    }
}

impl<'stack> Scheduler for Preemptive<'stack> {
    fn start(&self) -> ! {

        
        
        
        
        loop {

        }
    }

    /// Questa funzione è eseguita nell'interrupt SysTick, per ricercare il prossimo task da avviare.
    /// Al termine, la funzione triggera l'interrupt di PendSV, dove avviene lo switch.
    fn run_next(&mut self) {
        if let Some(next) = self.list.find_next_ready() {
            self.next_stack_ptr = 0;
            //next.run();
        } else {
            self.next_stack_ptr = 0;
        }
    }

    fn add_process(&mut self, process: PCB) -> Result<(), ()> {
        self.list.add(process)?;
        Ok(())
    }

    fn remove_process(&mut self, _prio: u8) -> Result<(), ()> {
        todo!()
    }
}
