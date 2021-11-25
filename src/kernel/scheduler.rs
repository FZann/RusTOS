use crate::kernel::processes::{Process, StackPointer, Ticks, PCB};
use core::{cell::Cell, intrinsics::unreachable};
use crate::kernel::assembly::SystemCall;

use super::SysCallType;


#[no_mangle]
pub static mut SCHEDULER: Preemptive = Preemptive::new();

pub trait Scheduler {
    fn start(&self) -> !;

    fn process_idle(&self, prio: u8);
    fn process_stop(&self, prio: u8);
    fn process_sleep(&self, prio: u8, ticks: Ticks);

    fn inc_system_ticks(&self) -> IncToken;
    fn run_next(&mut self, _token: IncToken) -> !;
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

    pub fn value(&self) -> usize {
        self.vec.get()
    }
}

impl core::ops::BitOr for BooleanVector {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            vec: Cell::new(self.value() | rhs.value())
        }
    }
}


/// La struttura tiene insieme i processi e i loro stati correlati
/// In questo modo ho creato un pezzetto dello scheduler.
/// I processi non sono a conoscenza del loro stato (idle, sleep, run...),
/// ma lo scheduler sì, tramite questa struttura.
struct ProcessList {
    processes: [Option<PCB>; 32],
    running: BooleanVector,
    idling: BooleanVector,
    sleeping: BooleanVector,
    stopped: BooleanVector,
}

impl ProcessList {
    /// Serve per poter usare la fn new() -> Self
    const NONE: Option<PCB> = None;

    pub const fn new() -> Self {
        Self {
            processes: [Self::NONE; 32],
            running: BooleanVector::new(),
            idling: BooleanVector::new(),
            sleeping: BooleanVector::new(),
            stopped: BooleanVector::new(),
        }
    }

    pub fn find_next_ready(&self) -> Option<&PCB> {
        /*
        Il processo che può partire è uno in idle, oppure lo stesso che già sta girando.
        Tramite un banale OR sommo i bit dei due vettori che rappresentano gli stati
        suddetti dei processi in lista.
        Poi cerco il primo ad '1': la sua posizione nel vettore indica l'indirizzo
        al quale troverò il processo da dare in pasto alla CPU.
         */

        let runnable = self.running.clone() | self.idling.clone();
        let next = runnable.find_first_set();

        match next {
            Ok(prio) => self.processes[prio].as_ref(),
            Err(_) => None,
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
        self.idling.set(prio);

        self.stopped.clear(prio);
        self.sleeping.clear(prio);
        self.running.clear(prio);
    }

    pub fn set_stop(&self, prio: u8) {
        self.stopped.set(prio);

        self.idling.clear(prio);
        self.sleeping.clear(prio);
        self.running.clear(prio);
    }

    pub fn set_sleeping(&self, prio: u8) {
        self.sleeping.set(prio);

        self.idling.clear(prio);
        self.stopped.clear(prio);
        self.running.clear(prio);
    }

    pub fn get_process_ref(&self, prio: usize) -> Option<&PCB> {
        self.processes[prio].as_ref()
    }

    /// Looppa su tutti gli indirizzi della lista, lanciando la funzione f su tutti gli elementi.
    pub fn foreach(&self, f: impl Fn(&PCB)) {
        for index in 0..=31usize {
            if let Some(pcb) = &self.processes[index] {
                f(pcb);
            }
        }
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
        if let Some(pcb) = self.list.get_process_ref(prio as usize) {
            self.list.idling.clear(pcb.prio());
            self.list.sleeping.clear(pcb.prio());
        }
    }

    /// I ticks di sleeping di un task non rappresentano i tick rimanenti alla scadenza,
    /// ma il valore assoluto che il sistema deve raggiungere per riattivare il processo.
    /// Questo elimina tutte le operazioni di sottrazione a tutti i contatori dei ticks di
    /// tutti i processi.
    fn process_sleep(&self, prio: u8, ticks: Ticks) {
        if let Some(pcb) = self.list.get_process_ref(prio as usize) {
            pcb.set_ticks(ticks + self.ticks.clone());
            self.list.set_sleeping(prio);
        }
    }

    fn inc_system_ticks(&self) -> IncToken {
        self.ticks.increment();
        
        // Settiamo come idle quei task la cui soglia di ticks è
        // stata superata dal conteggio dei ticks del sistema
        self.list.foreach( 
            |pcb| if pcb.get_ticks() == self.ticks {
                self.list.set_idle(pcb.prio());
            }
        );

        IncToken
    }

    /// Questa funzione è eseguita nell'interrupt SysTick, per ricercare il prossimo task da avviare.
    /// Al termine, la funzione triggera l'interrupt di PendSV, dove avviene lo switch.
    fn run_next(&mut self, _token: IncToken) -> ! {
        if let Some(pcb) = self.list.find_next_ready() {
            self.next_stack_ptr = pcb.stack_pointer();
        } else {
            self.next_stack_ptr = 0.into();
        }

        unsafe {
            SystemCall(SysCallType::ContextSwitch);
            unreachable();
        }
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
