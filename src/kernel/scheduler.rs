use crate::kernel::processes::{Process, StackPointer, Ticks, PCB};
use core::cell::Cell;

#[no_mangle]
pub static mut SCHEDULER: Preemptive = Preemptive::new();

pub trait Scheduler {
    fn start(&self) -> !;

    fn process_idle(&self, prio: u8);
    fn process_stop(&self, prio: u8);
    fn process_sleep(&mut self, prio: u8, ticks: Ticks);

    fn inc_system_ticks(&mut self) -> IncToken;
    fn run_next(&self, _token: IncToken);
    fn add_process(&mut self, process: PCB) -> Result<(), ()>;
    fn remove_process(&mut self, prio: u8) -> Result<(), ()>;
}

#[derive(Clone)]
pub struct BooleanVector {
    vec: Cell<usize>,
}

impl BooleanVector {
    pub const fn new() -> Self {
        BooleanVector { vec: Cell::new(0) }
    }

    pub fn read(&self, bit: u8) -> bool {
        self.vec.get() & (1 << bit) == (1 << bit)
    }

    pub fn set(&self, bit: u8) {
        let mut vec = self.vec.get();
        vec |= 1 << bit;
        self.vec.set(vec);
    }

    pub fn clear(&self, bit: u8) {
        let mut vec = self.vec.get();
        vec &= !(1 << bit);
        self.vec.set(vec);
    }

    pub fn get(&self) -> usize {
        self.vec.get()
    }
}

impl From<usize> for BooleanVector {
    fn from(vec: usize) -> Self {
        Self {
            vec: Cell::new(vec),
        }
    }
}

impl From<BooleanVector> for usize {
    fn from(bv: BooleanVector) -> Self {
        bv.vec.get()
    }
}

impl From<&BooleanVector> for usize {
    fn from(bv: &BooleanVector) -> Self {
        bv.vec.get()
    }
}

/// La struttura tiene insieme i processi e i loro stati correlati
/// In questo modo ho creato un pezzetto dello scheduler.
/// I processi non sono a conoscenza del loro stato (idle, sleep, run...),
/// ma lo scheduler sì, tramite questa struttura.
struct ProcessList {
    processes: [Option<PCB>; 32],
    active: BooleanVector,
    sleeping: BooleanVector,
}

impl ProcessList {
    /// Serve per poter usare la fn new() -> Self
    const NONE: Option<PCB> = None;

    pub const fn new() -> Self {
        Self {
            processes: [Self::NONE; 32],
            active: BooleanVector::new(),
            sleeping: BooleanVector::new(),
        }
    }

    pub fn find_next_ready(&self) -> Option<&PCB> {
        /* Usando l'istruzione 'clz' in assembly, otteniamo direttamente l'indice del processo
          con il quale accedere all'array dei PCB. Con una singola istruzione otteniamo immediatamente
          ciò che ci serve!
        */
        let next: BooleanVector = (self.active.get() & !self.sleeping.get()).into();
        let result = next.find_first_set();

        if let Ok(index) = result {
            self.processes[index].as_ref()
        } else {
            None
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

    pub fn remove(&mut self, prio: u8) -> Result<(), ()> {
        self.processes[prio as usize].take();
        Ok(())
    }

    pub fn set_idle(&self, prio: u8) {
        self.active.set(prio);
        self.sleeping.clear(prio);
    }

    pub fn set_stopped(&self, prio: u8) {
        self.active.clear(prio);
        self.sleeping.clear(prio);
    }

    pub fn set_sleeping(&self, prio: u8) {
        self.sleeping.set(prio);
    }

    pub fn get_process_ref(&self, prio: usize) -> Option<&PCB> {
        self.processes[prio].as_ref()
    }
}

/// Lo Scheduler tiene in memoria anche le variabili che servono per completare
/// un context switch. In questo modo evito di usare una serie di unsafe per
/// la modifica dei valori, perché non risultano statici allo scheduler stesso
#[repr(C)]
pub struct Preemptive {
    /* !!! --------------------- !!! */
    // L'accesso a queste variabili avviene anche via assembly! Non modificare la dichiarazione!
    running_stack_ptr: StackPointer,
    next_stack_ptr: StackPointer,
    /* !!! --------------------- !!! */
    list: ProcessList,
    ticks: Ticks,
    running: Option<*const PCB>,
}

unsafe impl Sync for Preemptive {}

/// Il suo scopo è quello di forzare un inc_system_ticks prima di chiamare run_next
pub struct IncToken;

impl Preemptive {
    pub const fn new() -> Self {
        Self {
            running_stack_ptr: StackPointer::new(),
            next_stack_ptr: StackPointer::new(),
            list: ProcessList::new(),
            ticks: Ticks::new(0),
            running: None,
        }
    }
}

impl Scheduler for Preemptive {
    fn start(&self) -> ! {
        loop {}
    }

    fn process_idle(&self, prio: u8) {
        self.list.set_idle(prio);
    }

    fn process_stop(&self, prio: u8) {
        if let Some(pcb) = self.list.processes[prio as usize].as_ref() {
            self.list.active.clear(pcb.prio());
            self.list.sleeping.clear(pcb.prio());
        }
    }

    fn process_sleep(&mut self, prio: u8, ticks: Ticks) {
        if let Some(pcb) = self.list.processes[prio as usize].as_mut() {
            pcb.sleep = ticks;
            self.list.sleeping.set(pcb.prio());
        }
    }

    fn inc_system_ticks(&mut self) -> IncToken {
        self.ticks.increment();
        for opt_pcb in self.list.processes.iter() {
            if let Some(pcb) = opt_pcb {}
        }
        IncToken
    }

    /// Questa funzione è eseguita nell'interrupt SysTick, per ricercare il prossimo task da avviare.
    /// Al termine, la funzione triggera l'interrupt di PendSV, dove avviene lo switch.
    fn run_next(&self, _token: IncToken) {
        todo!();
    }

    fn add_process(&mut self, process: PCB) -> Result<(), ()> {
        self.list.add(process)?;
        Ok(())
    }

    fn remove_process(&mut self, prio: u8) -> Result<(), ()> {
        self.list.remove(prio)?;
        Ok(())
    }
}
