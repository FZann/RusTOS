use core::arch::asm;


use crate::kernel::SysCallType;
use crate::kernel::scheduler::{Scheduler, SCHEDULER};


use crate::kernel::CriticalSection;
use crate::make_peripheral;

use super::processes::Process;
use super::registers::Peripheral;


const SCB_ICSR_PENDSVSET: usize = 1 << 28;

/// Stack frame hardware salvata dai Cortex-M
/// Permette di visualizzare i valori dei registri durante l'ultimo errore
#[repr(C)]
pub struct ExceptionFrame {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r12: u32,
    lr: u32,
    pc: u32,
    xpsr: u32,
}


#[derive(Clone, Copy)]
pub(crate) enum Interrupts {
    SVCall = 11,
    PendSV = 14,
    SysTick = 15,
}


pub(crate) enum IntPrio {
    Max = 0,
    Pri01 = 0x10,
    Pri02 = 0x20,
    Pri03 = 0x30,
    Pri04 = 0x40,
    Pri05 = 0x50,
    Pri06 = 0x60,
    Pri07 = 0x70,
    Pri08 = 0x80,
    Pri09 = 0x90,
    Pri10 = 0xA0,
    Pri11 = 0xB0,
    Pri12 = 0xC0,
    Pri13 = 0xD0,
    Pri14 = 0xE0,
    Min = 0xF0,
}

impl IntPrio {
    fn value(self) -> usize {
        self as usize
    }
}

impl Interrupts {
    fn number(self) -> u16 {
        self as u16
    }
}

pub enum ClockSource {
    ExternalClock = 0,
    CoreClock = 1 << 2,
}




/// Struttura dati effettiva sottostante allo ZST di accesso
#[repr(C)]
pub struct SysTickRegs {
    crs: u32,
    rvr: u32,
    cvr: u32,
    calib: u32,
}

impl SysTickRegs {
    const ENABLE: u32 = 1;
    const TICKINT: u32 = 1 << 1;
    const CLKSOURCE: u32 = 1 << 2;
    //const SKEW: u32 = 1 << 30;
    const TENMS_MASK: u32 = 0x00FF_FFFF;

    pub fn start(&mut self) {
        self.crs |= Self::ENABLE;
    }

    pub fn stop(&mut self) {
        self.crs &= !Self::ENABLE;
    }

    pub fn set_clocksource(&mut self, cksrc: ClockSource) -> &mut Self {
        self.crs &= !Self::CLKSOURCE;
        self.crs |= cksrc as u32;
        self
    }

    pub fn int_enable(&mut self) -> &mut Self {
        self.crs |= Self::TICKINT;
        self
    }

    pub fn init(&mut self) {
        self.stop();
        let reload = self.get_calibration();
        self.set_reload(reload).zero_count();
        self.set_clocksource(ClockSource::CoreClock).int_enable().start();
    }

    pub fn zero_count(&mut self) -> &mut Self {
        self.cvr = 0;
        self
    }

    pub fn set_reload(&mut self, reload: u32) -> &mut Self {
        self.rvr = reload;
        self
    }

    pub fn get_calibration(&mut self) -> u32 {
        // let skew = !((self.cvr & Self::SKEW) == Self::SKEW);
        let tenms = self.calib & Self::TENMS_MASK;
        tenms
    }
}


/// Struttura dati effettiva sottostante allo ZST di accesso
#[repr(C)]
pub struct NVICRegs {
    iser: [usize; 8],
    void1: [usize; 24],
    icer: [usize; 8],
    ispr: [usize; 8],
    void2: [usize; 24],
    icpr: [usize; 8],
    iabr: [usize; 8],
    void3: [usize; 32],
    ipr: [usize; 60],
    stir: usize,
}

impl NVICRegs {
    pub fn enable_interrupt(&mut self, int: Interrupts) {
        let n = int.number();
        match n {
            0 ..= 31 =>  self.iser[0] |= 1 << n,
            _ => (),
        };
    }


    pub fn disable_interrupt(&mut self, int: Interrupts) {
        let n = int.number();
        match n {
            0 ..= 31 =>  self.icer[0] |= 1 << n,
            _ => (),
        };
    }

    pub fn pend_interrupt(&mut self, int: Interrupts) {
        let n = int.number();
        match n {
            0 ..= 31 =>  self.ispr[0] |= 1 << n,
            _ => (),
        };
    }

    pub fn clear_interrupt(&mut self, int: Interrupts) {
        let n = int.number();
        match n {
            0 ..= 31 =>  self.icpr[0] |= 1 << n,
            _ => (),
        };
    }

    pub fn is_interrupt_active(&self, int: Interrupts) -> bool {
        let n = int.number();
        match n {
            0 ..= 31 =>  (self.icpr[0] & 1 << n) != 0,
            _ => false,
        }
    }

    pub fn set_interrupt_prio(&mut self, int: Interrupts, prio: IntPrio) {
        let n = (int.number() >> 2) as usize; // Divide per 4
        self.ipr[n] = prio.value() << (8 * n);
    }

}

make_peripheral!(SysTickTimer: 0xE000_E010, SysTickRegs);
make_peripheral!(NVIC: 0xE000_E100, NVICRegs);


#[doc(hidden)]
#[derive(Clone, Copy)]
pub union Vector {
    handler: unsafe extern "C" fn(),
    reserved: usize,
}

#[no_mangle]
#[link_section = ".vector_table_arm_vectors"]
static __ARM_VECTORS: [Vector; 15] = [
    // Exception 1: Reset Vector
    Vector { handler: __ENTRY },
    // Exception 2: Non Maskable Interrupt
    Vector {
        handler: NonMaskableInt,
    },
    // Exception 3: Hard Fault Interrupt.
    Vector {
        handler: HardFaultTrampoline,
    },
    // Exception 4: Memory Management Interrupt [not on Cortex-M0 variants].
    #[cfg(not(armv6m))]
    Vector {
        handler: MemoryManagement,
    },
    #[cfg(armv6m)]
    Vector { reserved: 0 },
    // Exception 5: Bus Fault Interrupt [not on Cortex-M0 variants].
    #[cfg(not(armv6m))]
    Vector { handler: BusFault },
    #[cfg(armv6m)]
    Vector { reserved: 0 },
    // Exception 6: Usage Fault Interrupt [not on Cortex-M0 variants].
    #[cfg(not(armv6m))]
    Vector {
        handler: UsageFault,
    },
    #[cfg(armv6m)]
    Vector { reserved: 0 },
    // Exception 7: Secure Fault Interrupt [only on Armv8-M].
    #[cfg(armv8m)]
    Vector {
        handler: SecureFault,
    },
    #[cfg(not(armv8m))]
    Vector { reserved: 0 },
    // 8-10: Reserved
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    // Exception 11: SV Call Interrupt.
    Vector { handler: SVCall },
    // Exception 12: Debug Monitor Interrupt [not on Cortex-M0 variants].
    #[cfg(not(armv6m))]
    Vector {
        handler: DebugMonitor,
    },
    #[cfg(armv6m)]
    Vector { reserved: 0 },
    // 13: Reserved
    Vector { reserved: 0 },
    // Exception 14: Pend SV Interrupt [not on Cortex-M0 variants].
    Vector { handler: PendSV },
    // Exception 15: System Tick Interrupt.
    Vector { handler: SysTick },
];


#[no_mangle]
pub extern "C" fn NonMaskableInt() {}

#[no_mangle]
pub extern "C" fn DebugMonitor() {}

#[no_mangle]
pub extern "C" fn UsageFault() {}

#[no_mangle]
pub extern "C" fn BusFault() {}

#[no_mangle]
pub extern "C" fn MemoryManagement() {}

#[no_mangle]
pub extern "C" fn SecureFault() {}

#[no_mangle]
pub extern "C" fn SysTick() {
    let cs = CriticalSection::activate();
    SCHEDULER.get(&cs).inc_system_ticks();
    SCHEDULER.get(&cs).schedule_next();
}

#[no_mangle]
#[link_section = ".vector_table_interrupts"]
static __INTERRUPTS: [Vector; 240] = [Vector { reserved: 0 }; 240];

pub unsafe fn request_context_switch() {
    let scb: *mut usize = 0xE000_ED04 as *mut usize;

    (*scb) |= SCB_ICSR_PENDSVSET;
}

#[inline(always)]
pub fn interrupt_disable() {
    unsafe {
        asm!("cpsid i");
    }
}

#[inline(always)]
pub fn interrupt_enable() {
    unsafe {
        asm!("cpsie i");
    }
}

#[inline(always)]
pub fn nop() {
    unsafe {
        asm!("nop");
    }
}


#[naked]
#[no_mangle]
#[link_section = ".os_entry"]
pub unsafe extern "C" fn __ENTRY() {
    asm!(
        /* Copy the data segment initializers from flash to SRAM */
        "ldr    r0, =ld_data_start",
        "ldr    r1, =ld_data_end",
        "ldr    r2, =ld_data",
        "movs   r3, #0",
        "b	    1f",
        /* Loads the data segment */
        "0:",
        "ldr    r4, [r2, r3]",
        "str    r4, [r0, r3]",
        "adds   r3, r3, #4",
        "1:",
        "adds   r4, r0, r3",
        "cmp    r4, r1",
        "bcc    0b",
        /* Zero fill the bss segment. */
        "ldr    r2, =ld_bss_start",
        "ldr    r4, =ld_bss_end",
        "movs   r3, #0",
        "b      3f",
        "2:",
        "str    r3, [r2]",
        "adds   r2, r2, #4",
        "3:",
        "cmp    r2, r4",
        "bcc    2b",
        /* Set the stack and call the application's entry point.*/
        //"ldr    r0, =ld_stack_start",
        //"mrs    r0, msp",
        "b	    OSEntry",
        options(noreturn)
    );
}

#[naked]
#[no_mangle]
#[link_section = ".os_errorhandler"]
pub unsafe extern "C" fn HardFaultTrampoline() {
    asm!(
        // Ottiene la &Process running
        "ldr    r3, =SCHEDULER",
        "ldr    r2, [r3, #0]",
        "ldr    r3, [r3, #4]",

        // Test se siamo in contesto privilegiato o in thread
        "mov    r0, lr",
        "mov    r1, #4",
        "tst    r0, r1",
        
        // Branch per il contesto
        "bne    0f",
        "mrs    r0, MSP",
        "mov    r1, #2",
        "b      OSFault",
        "0:",
        "mrs    r0, PSP",
        "mov    r1, #1",
        "b      OSFault",
        options(noreturn)
    );
}

#[naked]
#[no_mangle]
pub unsafe extern "C" fn PendSV() {
    asm!(
        // Il layout in memoria di &dyn Process è:
        // [RAM data : *usize; vtable : *usize]
        // Sono due puntatori, non uno solo. Tenere a mente questo.

        // R3: &Scheduler
        // R2: &dyn Process
        // R1: Start Stack Pointer/Watermark
        // R0: value of StackPointers

        /* Salvataggio del contesto attuale */
        "cpsid	i",
        "mrs    r0, psp",           // Take PSP value out to r0
        "stmfd  r0!, {{r4-r11}}",   // Save Context
        "ldr    r3, =SCHEDULER",    // Get &Scheduler
        "ldr    r2, [r3, #0]",      // Get running &dyn Process to switch out
        "str	r0, [r2]",          // Save PSP value in &StackPointer (same as &dyn Process)
        // Calcola il watermark
        "ldr    r1, [r2, #4]",      // &Start Stack Pointer (ref)
        "sub    r1, r1, r0",        // Ottiene il numero di bytes nella stack (r1 = SP Start - SP attuale)
        "lsr    r1, r1, #2",        // Divide per 4 e ottiene il numero di parole (Watermark)
        "ldr    r0, [r2, #8]",      // Ottiene il vecchio Watermark
        "cmp    r0, r1",            // Old Water > New Water??
        "it     lt",                // Abilita le istruzioni condizionali per "minore di"
        "strlt  r1, [r2, #8]",      // Salva il valore nel WaterMark solo se il vecchio è minore del nuovo
        "isb",

        /* Caricamento del nuovo contesto */
        "bl     switch_to_next",
        
        // Carica la nuova stack
        "ldr    r3, =SCHEDULER",    // Get &Scheduler
        "ldr    r2, [r3, #0]",      // Get running &dyn Process' StackPointer to switch out
        "ldr    r0, [r2]",          // Get value of StackPointer
        "ldmfd  r0!, {{r4-r11}}",   // Load Context
        "str    r0, [r2]",          // Saves new StackPointer value in &dyn Process
        "msr	psp, r0",           // Moves StackPointer in PSP
        // Instruction Syncro Barrier per sicurezza
        "isb",
        /* Ritorno al thread, con PSP e in modo non privilegiato */
        "ldr    lr, =0xFFFFFFFD",
        "cpsie	i",
        "bx     lr",
        options(noreturn)
    );
}

#[no_mangle]
#[inline(always)]
unsafe extern "C" fn switch_to_next() {
    let s = unsafe { SCHEDULER.unsafe_get() };
    s.running = s.next;
    s.next = None;
}


#[naked]
#[no_mangle]
pub unsafe extern "C" fn load_first_process() -> ! {
    asm!(
        // R3: &Scheduler
        // R2: &PCB, running or next
        // R0: value of StackPointers, running or next
        /* Caricamento del nuovo contesto */
        "ldr    r3, =SCHEDULER",  // Get &Scheduler
        "ldr    r2, [r3, #0]",    // Get &PCB's StackPointer to run
        "ldr    r0, [r2]",        // Get value of StackPointer
        "ldmfd  r0!, {{r4-r11}}", // Load Context
        "str    r0, [r2]",        // Saves new Stackpointer value in &PCB
        "msr	psp, r0",         // Moves r0 in PSP
        "isb",
        /* Ritorno al thread, con PSP e in modo non privilegiato */
        "ldr    lr, =0xFFFFFFFD",
        "cpsie	i",
        "bx     lr",
        options(noreturn)
    );
}

pub(crate) fn idle_task(_task: &mut dyn Process) -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

#[inline(always)]
pub fn svc(sys_call: SysCallType) {
    unsafe {
        SCHEDULER.unsafe_get().sys_call = sys_call;

        match sys_call {
            SysCallType::Nop => (),
            SysCallType::ProcessIdle(_) =>      asm!("svc    1"),
            SysCallType::ProcessSleep(_, _) =>  asm!("svc    2"),
            SysCallType::ProcessStop(_) =>      asm!("svc    3"),
            SysCallType::StartScheduler =>      asm!("svc    4"),
        }
    }
}





/// Un accesso safe qui non serve, perché siamo al massimo possibile della priorità
/// dell'NVIC. Questo codice non può essere interrotto da nulla.
#[no_mangle]
pub extern "C" fn SVCall() {
    let s = unsafe { SCHEDULER.unsafe_get() };

    match s.sys_call {
        SysCallType::Nop => (),
        SysCallType::StartScheduler => {
            let nvic = unsafe { NVIC::regs() };
            nvic.enable_interrupt(Interrupts::SVCall);
            nvic.enable_interrupt(Interrupts::PendSV);
            nvic.enable_interrupt(Interrupts::SysTick);
            nvic.set_interrupt_prio(Interrupts::SVCall, IntPrio::Max);
            nvic.set_interrupt_prio(Interrupts::SysTick, IntPrio::Pri01);
            nvic.set_interrupt_prio(Interrupts::PendSV, IntPrio::Min);

            let systick = unsafe { SysTickTimer::regs() };
            systick.init();

            s.start();  
        }

        SysCallType::ProcessIdle(prio) => s.process_idle(prio),
        SysCallType::ProcessStop(prio) => s.process_stop(prio),
        SysCallType::ProcessSleep(prio, ticks) => s.process_sleep(prio, ticks),
    };
}
