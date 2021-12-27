extern "C" {
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
        /* Call the application's entry point.*/
        "b	    OSEntry",
        options(noreturn)
    );
}

impl crate::kernel::scheduler::BooleanVector {
    pub fn find_first_set(&self) -> Result<usize, ()> {
        let vec: usize = self.value();
        if vec != 0 {
            let res: usize;
            unsafe {
                asm!(
                    "clz    {1}, {0}",
                    in(reg) vec,
                    out(reg) res,
                );
            }
            // Devo sottrarre per ottenere il valore corretto
            Ok(31 - res)
        } else {
            Err(())
        }
    }
}

/// Usato nella funzione successiva di PendSV
use crate::kernel::scheduler::SCHEDULER;
use crate::kernel::SysCallType;

use super::processes::Process;
use super::scheduler::Scheduler;

use core::arch::asm;

#[naked]
#[no_mangle]
pub unsafe extern "C" fn PendSV() {
    asm!(
        // R3: &Scheduler
        // R2: &PCBs, running or next
        // R0: value of StackPointers, running or next

        /* Salvataggio del contesto attuale */
        "cpsid	i",
        "mrs    r0, psp",         // Take PSP value out to r0
        "stmfd  r0!, {{r4-r11}}", // Save Context
        "ldr    r3, =SCHEDULER",  // Get &Scheduler
        "ldr    r2, [r3, #4]", // Get running &PCB's StackPointer to switch out (#4 due to Option)
        "str	r0, [r2]",        // Save PSP value in &PCB (same as &StackPointer, 'cause of repr(C))
        "isb",
        /* Check per determinare se c'è un nuovo processo da caricare */
        //"bl     Supervisor",       // <--- Da fare (serve?)


        /* Caricamento del nuovo contesto */
        "ldr    r2, [r3, #12]", // Get next &PCB's StackPointer to switch in (#12 due to Option)
        "str    r2, [r3, #4]",  // Save &PCB as running
        // Azzera il "next stack pointer"
        "mov    r1, #0",
        "str    r1, [r3, #8]",    // Clear next &PCB (seen as None in Rust side)
        "str    r1, [r3, #12]",   // Clear next &PCB (seen as None in Rust side)
        "ldr    r0, [r2]",        // Get value of StackPointer
        "ldmfd  r0!, {{r4-r11}}", // Load Context
        "str    r0, [r2]",        // Saves new StackPointer value in &PCB
        "msr	psp, r0",            // Moves StackPointer in PSP
        "isb",
        /* Ritorno al thread, con PSP e in modo non privilegiato */
        "ldr    lr, =0xFFFFFFFD",
        "cpsie	i",
        "bx     lr",
        options(noreturn)
    );
}

/*
#[no_mangle]
pub unsafe extern "C" fn Supervisor(stack: &ExceptionFrame) {
    let ciao = stack;
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
        "ldr    r2, [r3, #4]",    // Get &PCB's StackPointer to run (#4 due to Option)
        "ldr    r0, [r2]",        // Get value of StackPointer
        "ldmfd  r0!, {{r4-r11}}", // Load Context
        "str    r0, [r2]",        // Saves new Stackpointer value in &PCB
        "msr	psp, r0",            // Moves r0 in PSP
        "isb",
        /* Ritorno al thread, con PSP e in modo non privilegiato */
        "ldr    lr, =0xFFFFFFFD",
        "bx     lr",
        options(noreturn)
    );
}

#[no_mangle]
extern "C" fn sleep_cpu() {
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
        match SCHEDULER.sys_call {
            SysCallType::Nop => (),
            SysCallType::ProcessIdle => asm!("svc    1"),
            SysCallType::ProcessSleep(_) => asm!("svc    2"),
            SysCallType::ProcessStop => asm!("svc    3"),
            SysCallType::StartScheduler => asm!("svc    4"),
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
        "mrs    r0, MSP",
        "b      OSFault",
        "0:",
        "mrs    r0, PSP",
        "b      OSFault",
        options(noreturn)
    );
}

#[no_mangle]
pub extern "C" fn SVCall() {
    unsafe {
        match &SCHEDULER.sys_call {
            SysCallType::Nop => (),
            SysCallType::StartScheduler => SCHEDULER.start(),
            SysCallType::ProcessIdle => {
                let prio = (*SCHEDULER.process_running.get().unwrap()).prio();
                SCHEDULER.process_idle(prio);
            }
            SysCallType::ProcessStop => {
                let prio = (*SCHEDULER.process_running.get().unwrap()).prio();
                SCHEDULER.process_stop(prio);
            }
            SysCallType::ProcessSleep(ticks) => {
                let prio = (*SCHEDULER.process_running.get().unwrap()).prio();
                SCHEDULER.process_sleep(prio, ticks.clone());
            }
        }
    }
}
