use core::marker::PhantomData;
use core::mem::MaybeUninit;

use crate::bitvec::BitVec;
use crate::kernel::{SysCallType, Ticks, CorePeripherals};

use crate::kernel::CritCell;

use super::{CritSect, SystemCall};


#[no_mangle]
pub static KERNEL: CritCell<Kernel> = CritCell::new(Kernel::new());
pub static mut IDLE_TASK: Task<32> = Task::new(super::idle_task, 200);

pub type Task = fn(&mut TCB) -> !;

/// Lo Scheduler tiene in memoria anche le variabili che servono per completare
/// un context switch. In questo modo evito di usare una serie di unsafe per
/// la modifica dei valori, perché non risultano statici allo scheduler stesso
#[repr(C)]
pub struct Kernel<'p> {
    /* !!! --------------------- !!! */
    // L'accesso a queste variabili avviene anche via assembly! Non modificare la dichiarazione!
    // Il fatto di usare &dyn Process implica una dimensione di due words dei campi running e next.
    // Questo si deve riflettere nell'assembly, usando i giusti offset.
    pub(crate) running: MaybeUninit<*const TCB<'p>>,
    pub(crate) next: MaybeUninit<*const TCB<'p>>,
    /* !!! --------------------- !!! */
    pub(crate) sys_call: SysCallType,

    /// Ticks totali da quando il sistema è partito
    ticks: Ticks,

    /// Periferiche del core
    core: CorePeripherals,  

    /// Lista processi
    processes: [MaybeUninit<TCB<'p>>; BitVec::BITS],
    ready: BitVec,
    sleeping: BitVec,
}

impl<'p> Kernel<'p> {
    pub const fn new() -> Self {
        Self {
            running: MaybeUninit::zeroed(),
            next: MaybeUninit::zeroed(),
            sys_call: SysCallType::Nop,
            ticks: 0,
            core: CorePeripherals::new(),
            processes: [const { MaybeUninit::zeroed() }; BitVec::BITS],
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
        }
            
            /* Qui non dovremmo mai arrivare, in quanto la CPU è sotto controllo dello scheduler */
    }

    #[inline]
    fn running(&self) -> &TCB {
        unsafe { & *self.running.assume_init() }
    }

    #[inline]
    fn get_task(&mut self, prio: usize) -> &mut TCB<'p> {
        unsafe { self.processes[prio].assume_init_mut() }
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
        let task = self.get_task(prio);
        task.set_ticks(ticks);
        self.ready.clear(prio);
        self.sleeping.set(prio);
        self.schedule_next();
    }

    /// I tick di sleeping di un task vengono diminuiti ad ogni tick
    /// di sistema, fino all'azzeramento.
    /// A questo punto il task torna schedulabile.
    pub(crate) fn inc_system_ticks(&mut self) {
        self.ticks += 1;

        for id in self.sleeping.into_iter() {
            let task = self.get_task(id);
            if  task.decrement_ticks() == 0 {
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
        match (self.running().prio(), self.ready.find_higher_set()) {
            (run, Ok(next)) if run != next => {

                self.next = MaybeUninit::new(self.get_task(next));
                self.request_context_switch();                    
            }

            // Non c'è un task da schedulare!
            (_, Err(_)) => {
                // TODO: implementa lo sleep e rimuovi totalmente IDLE_TASK
                self.core.sleep_on_exit(true);

                self.next = unsafe { MaybeUninit::new(&IDLE_TASK) };
                self.request_context_switch();                    
            }
            // Entriamo in questa casistica se run.prio() == self.schedulable.first_set().id
            // Quindi usciamo senza fare nulla
            _ => {}
        }
    }

    pub fn new_task(&mut self, task: Task, prio: usize, words: usize) {
        let stack;
        let remain;
        unsafe {
            (stack, remain) = STACK.split_at(words);
            
        } 
        self.processes[prio].write(TCB::new(task, prio, stack));
        
        self.get_task(prio).setup();
        self.ready.set(prio);
    }

    pub fn remove_process(&mut self, prio: usize) {
        self.processes[prio] = MaybeUninit::zeroed();
        self.ready.clear(prio);
        self.sleeping.clear(prio);
    }
}


#[repr(C)]
pub(crate) struct Stack<'s> {
    ptr: usize,
    start: usize,
    watermark: usize,
    data: *const [usize],
    lifetime: PhantomData<&'s [usize]>,
}

impl<'s> Stack<'s> {
    pub(crate) const fn new(data: &'s [usize]) -> Self {
        Self {
            ptr: 0,
            start: 0,
            watermark: 0,
            data,
            lifetime: PhantomData,
        }
    }

    pub(crate) fn update_watermark(&mut self) {
        let words = (self.start - self.ptr) >> 2;
        if words > self.watermark {
            self.watermark = words;
        }
    }

    pub(crate) fn get_stack(&self) -> &mut [usize] {
        unsafe { &mut *(self.data as *mut [usize]) }
    }
}

/// **TCB**
///
/// Task Control Block per un dispositivo ARM Cortex-M4.
#[repr(C)]
pub struct TCB<'t> {
    /* !!! --------------------- !!! */
    // L'accesso a queste variabili avviene anche via assembly! Non modificare la dichiarazione!
    stack: Stack<'t>,
    /* !!! --------------------- !!! */
    task: Task,
    ticks: Ticks,
    prio: usize,
}

impl<'t> TCB<'t> {
    pub(crate) const fn new(task: Task, prio: usize, stack: &'t [usize]) -> Self {
        Self {
            stack: Stack::new(stack),
            task,
            prio,
            ticks: 0,
        }
    }

    pub(crate) fn setup(&mut self) {
        let pointer: usize = unsafe { core::mem::transmute(&mut *self) };
        let stack = self.stack.get_stack();
        let len = stack.len();

        stack[len - 01] = 1 << 24; // xPSR - Thumb state attivo
        stack[len - 02] = self.task as usize; // PC
        stack[len - 03] = 0xFFFFFFFD; // LR
        stack[len - 04] = 0xC; // R12
        stack[len - 05] = 0x3; // R3
        stack[len - 06] = 0x2; // R2
        stack[len - 07] = 0x1; // R1
        stack[len - 08] = pointer; // R0

        // Software stack; non è strettamente necessaria, serve più per debug
        stack[len - 09] = 0xB; // R11
        stack[len - 10] = 0xA; // R10
        stack[len - 11] = 0x9; // R9
        stack[len - 12] = 0x8; // R8
        stack[len - 13] = 0x7; // R7
        stack[len - 14] = 0x6; // R6
        stack[len - 15] = 0x5; // R5
        stack[len - 16] = 0x4; // R4

        let sp = (&stack[len - 16] as *const usize) as usize;
        let start = (&stack[len - 01] as *const usize) as usize;
        self.stack.ptr = sp;
        self.stack.start = start;
    }


    pub fn handle(&self) -> Task {
        self.task
    }

    pub fn prio(&self) -> usize {
        self.prio
    }

    pub(crate) fn set_ticks(&mut self, ticks: Ticks) {
        self.ticks = ticks;
    }

    pub(crate) fn decrement_ticks(&mut self) -> Ticks {
        self.ticks = self.ticks.saturating_sub(1);
        self.ticks
    }

    pub fn idle(&mut self) {
        KERNEL.with(|k| k.process_idle(self.prio));
    }
    

    pub fn stop(&mut self) {
        KERNEL.with(|k| k.process_stop(self.prio));
    }

    pub fn sleep(&mut self, ticks: Ticks) {
        KERNEL.with(|k| k.process_sleep(self.prio, ticks));
    }
}
