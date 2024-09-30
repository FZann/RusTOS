pub(crate) mod core_peripherals;

use core::arch::asm;
use core::task::Context;

use crate::kernel::tasks::{Kernel, KERNEL};
use crate::kernel::tasks::StackPointer;
use crate::kernel::SysCallType;
use crate::kernel::CritSect;

use super::ExecContext;

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
    let cs = CritSect::activate();
    KERNEL.get(&cs).inc_system_ticks();
    KERNEL.get(&cs).schedule_next();
}

#[no_mangle]
#[link_section = ".vector_table_interrupts"]
static __INTERRUPTS: [Vector; 240] = [Vector { reserved: 0 }; 240];

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
        "ldr    r3, =KERNEL",
        "ldr    r2, [r3, #0]",
        "ldr    r3, [r3, #4]",

        "mov    r0, lr",
        "msr    CONTROL, r1",       // Test se siamo in contesto privilegiato o in thread

        // Gestione dell'errore da parte di Rust
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

        // r0: &Scheduler
        // r1: &dyn Process
        // r2: Start Stack Pointer/Watermark
        // r3: value of StackPointers

        /* Salvataggio del contesto attuale */
        "cpsid	i",
        "mrs    r3, psp",           // Take PSP value out to r3
        "stmfd  r3!, {{r4-r11}}",   // Save Context
        "ldr    r0, =KERNEL",       // Get &Scheduler
        "ldr    r1, [r0, #0]",      // Get running &dyn Process to switch out
        "str	r3, [r1]",          // Save PSP value in &StackPointer (same as &dyn Process)
        
        // Calcola il watermark - Fatto in Rust!
        //"ldr    r2, [r1, #4]",      // &Start Stack Pointer (ref)
        //"sub    r2, r2, r3",        // Ottiene il numero di bytes nella stack (r2 = SP Start - SP attuale)
        //"lsr    r2, r2, #2",        // Divide per 4 e ottiene il numero di parole (Watermark)
        //"ldr    r3, [r1, #8]",      // Ottiene il vecchio Watermark
        //"cmp    r3, r2",            // Old Water > New Water??
        //"it     lt",                // Abilita le istruzioni condizionali per "minore di"
        //"strlt  r2, [r1, #8]",      // Salva il valore nel WaterMark solo se il vecchio è minore del nuovo
        
        /* Caricamento del nuovo contesto */
        "bl     switch_to_next",
        
        // Carica la nuova stack
        //"ldr    r0, =KERNEL",       // Get &Scheduler
        "ldr    r1, [r0, #0]",      // Get running &dyn Process' StackPointer to switch out
        "ldr    r3, [r1]",          // Get value of StackPointer
        "ldmfd  r3!, {{r4-r11}}",   // Load Context
        "str    r3, [r1]",          // Saves new StackPointer value in &dyn Process
        "msr	psp, r3",           // Moves StackPointer in PSP
        // Instruction Syncro Barrier per sicurezza
        "isb",
        /* Ritorno al thread, con PSP e in modo non privilegiato */
        "ldr    lr, =0xFFFFFFFD",
        "cpsie	i",
        "bx     lr",
        options(noreturn)
    );
}



pub(crate) fn idle_task(_task: &mut dyn crate::kernel::tasks::Process) -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}



impl<'p> Kernel<'p> {

    #[inline(always)]
    pub(crate) fn request_context_switch(&self) {
        if Kernel::get_context().is_privileged() {
            let scb: *mut usize = 0xE000_ED04 as *mut usize;
            unsafe { (*scb) |= SCB_ICSR_PENDSVSET };
        } else {
            SystemCall(SysCallType::ContextSwith);
        }
    }

    #[naked]
    #[no_mangle]
    pub(crate) unsafe extern "C" fn load_first_process(&self) -> ! {
        asm!(
            // R0: &Scheduler - dovuto alle AAPCS
            // R2: &dyn Process, running or next
            // R3: value of StackPointers, running or next
            /* Caricamento del nuovo contesto */
            "ldr    r2, [r0, #0]",    // Get &PCB's StackPointer to run
            "ldr    r3, [r2]",        // Get value of StackPointer
            "ldmfd  r3!, {{r4-r11}}", // Load Context
            "str    r3, [r2]",        // Saves new Stackpointer value in &PCB
            "msr	psp, r3",         // Moves r0 in PSP
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
    unsafe extern "C" fn switch_to_next<'k>(&mut self, sp: &'k mut StackPointer) -> &Self {
        sp.update_watermark();
        self.running = self.next;
        self.next = None;
        self
    }

    #[inline(always)]
    pub(crate) extern "C" fn get_context() -> ExecContext {
        let val: usize;
        unsafe {
            asm!(
                "msr    CONTROL, {out}",
                out = out(reg) val,
            );
            val.into()
        }
    }

}



#[allow(non_snake_case)]
#[inline(always)]
pub fn SystemCall(sys_call: SysCallType) {
    unsafe {
        let cs = CritSect::activate();
        KERNEL.get(&cs).sys_call = sys_call;
        drop(cs);

        match sys_call {
            SysCallType::Nop => (),
            SysCallType::StartScheduler =>      asm!("svc   1"),
            SysCallType::ContextSwith =>        asm!("svc   2"),
        }
    }
}


/// Un accesso safe qui non serve, perché siamo al massimo possibile della priorità
/// dell'NVIC. Questo codice non può essere interrotto da nulla.
#[no_mangle]
pub extern "C" fn SVCall() {
    let cs = CritSect::activate();
    let k = KERNEL.get(&cs);

    match k.sys_call {
        SysCallType::Nop => (),
        SysCallType::StartScheduler => {
            k.start();  
        }

        SysCallType::ContextSwith => k.request_context_switch(),
    };
}