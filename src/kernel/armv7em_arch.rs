use core::arch::asm;

use crate::kernel::SysCallType;
use crate::kernel::scheduler::{Scheduler, SCHEDULER};

//use cortex_m::Peripherals;

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
enum Interrupts {
    SVCall = 11,
    PendSV = 14,
    SysTick = 15,
}

impl Interrupts {
    fn number(self) -> u16 {
        self as u16
    }
}


/*
#[allow(dead_code)]
extern "C" {
    //#[link_section = ".static_kernel_variables"]
    //static ld_stack_start: u32;
    #[link_section = ".static_kernel_variables"]
    static ld_data_start: u32;
    #[link_section = ".static_kernel_variables"]
    static ld_data_end: u32;
    #[link_section = ".static_kernel_variables"]
    static ld_data: u32;
    #[link_section = ".static_kernel_variables"]
    static ld_bss_start: u32;
    #[link_section = ".static_kernel_variables"]
    static ld_bss_end: u32;
}
*/

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
#[link_section = ".vector_table_interrupts"]
static __INTERRUPTS: [Vector; 240] = [Vector { reserved: 0 }; 240];

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
    unsafe {
        SCHEDULER.inc_system_ticks();
        SCHEDULER.schedule_next();
    }
}


const SCB_ICSR_PENDSVSET: usize = 1 << 28;

pub unsafe fn request_context_switch() {
    let scb: *mut usize = 0xE000_ED04 as *mut usize;

    (*scb) |= SCB_ICSR_PENDSVSET;
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
pub unsafe extern "C" fn PendSV() {
    asm!(
        // Il layout in memoria di &dyn Process è:
        // [RAM data : *usize; vtable : *usize]
        // Sono due puntatori. A noi serve il primo puntatore verso la memoria RAM
        // Nel codice seguente effettueremo questo puntamento per cambiare contesto

        // R3: &Scheduler
        // R2: &dyn Process, running or next
        // R0: value of StackPointers

        /* Salvataggio del contesto attuale */
        "cpsid	i",
        "mrs    r0, psp",           // Take PSP value out to r0
        "stmfd  r0!, {{r4-r11}}",   // Save Context
        "ldr    r3, =SCHEDULER",    // Get &Scheduler
        "ldr    r2, [r3, #0]",      // Get running &dyn Process' StackPointer to switch out
        "str	r0, [r2]",          // Save PSP value in &dyn Process (same as &StackPointer because of repr(C))
        "isb",
        /* Caricamento del nuovo contesto */
        //"bl     switch_to_next",
        "ldr    r2, [r3, #8]",      // Get next &dyn Process' StackPointer to switch in
        "str    r2, [r3, #0]",      // Save &dyn Process' data as running

        // Azzera il "next &dyn Process"
        "mov    r1, #0",
        "str    r1, [r3, #8]",      // Clear next &dyn Process (seen as None in Rust side)
        "str    r1, [r3, #12]",     // Clear next &dyn Process (seen as None in Rust side)
        // Carica la nuova stack
        "ldr    r0, [r2]",          // Get value of StackPointer
        "ldmfd  r0!, {{r4-r11}}",   // Load Context
        "str    r0, [r2]",          // Saves new StackPointer value in &dyn Process
        "msr	psp, r0",           // Moves StackPointer in PSP
        "isb",
        /* Ritorno al thread, con PSP e in modo non privilegiato */
        "ldr    lr, =0xFFFFFFFD",
        "cpsie	i",
        "bx     lr",
        options(noreturn)
    );
}

/*
NON VA, LOL! Probabilmente non salva i registri perché è già in contesto privilegiato
e corrompe i puntamenti dell'assembly che viene dopo.

/// Serve per cambiare i task con codice Rust, per maggiore sicurezza
#[no_mangle]
pub unsafe extern "C" fn switch_to_next() {
    SCHEDULER.running = SCHEDULER.next;
    SCHEDULER.next = None;
}
*/

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

pub(crate) fn idle_task() -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

#[inline(always)]
pub fn svc(sys_call: SysCallType) {
    unsafe {
        SCHEDULER.sys_call = sys_call;
    }
    unsafe {
        match sys_call {
            SysCallType::Nop => (),
            SysCallType::ProcessIdle =>     asm!("svc    1"),
            SysCallType::ProcessSleep(_) => asm!("svc    2"),
            SysCallType::ProcessStop =>     asm!("svc    3"),
            SysCallType::StartScheduler =>  asm!("svc    4"),
        }
    }
}

#[naked]
#[no_mangle]
#[link_section = ".os_errorhandler"]
pub unsafe extern "C" fn HardFaultTrampoline() {
    asm!(
        "mov    r0, lr",
        "mov    r1, #4",
        "tst    r0, r1",
        "bne    0f",
        "mov    r0, #2",
        "mrs    r1, MSP",
        "b      OSFault",
        "0:",
        "mov    r0, #1",
        "mrs    r1, PSP",
        "b      OSFault",
        options(noreturn)
    );
}

const SYST_CSR_ENABLE: usize = 1 << 0;
const SYST_CSR_TICKINT: usize = 1 << 1;
const SYST_CSR_CLKSOURCE: usize = 1 << 2;

#[no_mangle]
pub extern "C" fn SVCall() {
    unsafe {

        match SCHEDULER.sys_call {
            SysCallType::Nop => (),
            SysCallType::StartScheduler => {
                //let mut p = cortex_m::Peripherals::take().unwrap();

                let syst: *mut usize = 0xE000_E010 as *mut usize;
                (*syst) |= SYST_CSR_CLKSOURCE;
                (*syst.offset(1)) = *syst.offset(3);
                (*syst) |= SYST_CSR_TICKINT | SYST_CSR_ENABLE;
               
                /*
                let sys_tick = &mut p.SYST;
                let reload = cortex_m::peripheral::SYST::get_ticks_per_10ms();
                sys_tick.set_reload(reload);
                sys_tick.enable_interrupt();
                sys_tick.enable_counter();
                */

                //let nv: *mut usize = 0xE000_E100 as *mut usize;

                //let nvic = &mut p.NVIC;
                //nvic.set_priority(Interrupts::SVCall, 0);
                //nvic.set_priority(Interrupts::SysTick, 1);
                //nvic.set_priority(Interrupts::PendSV, 255);

                SCHEDULER.start();
            }

            SysCallType::ProcessIdle => SCHEDULER.running_idle(),
            SysCallType::ProcessStop => SCHEDULER.running_stop(),
            SysCallType::ProcessSleep(ticks) => SCHEDULER.running_sleep(ticks),
        };
    }
}

