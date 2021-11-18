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
        let vec: usize = self.into();
        let res: usize;
        unsafe {
            asm!(
                "clz    {1}, {0}",
                in(reg) vec,
                out(reg) res,
            );
        }
        if res < 32 {
            Ok(res)
        } else {
            Err(())
        }
    }
}

use crate::kernel::scheduler::SCHEDULER;

#[naked]
#[no_mangle]
pub unsafe extern "C" fn PendSV() {
    asm!(
        // Questo interrupt richiede che gli vengano passati i puntatori al TCB
        // del task attuale e al nuovo task, in maniera tale da gestire il context switch.
        // Il passaggio avviene tramite l'uso di due variabili static mut, che saranno
        // modificate preventivamente dall'algoritmo di scheduling.

        /* Salvataggio del contesto attuale */
        // 1. Metti il Process Stack usize in r0
        // 2. Store Multiple usando l'indirizzo puntato da r0. Modifica il valore di r0
        // 3. Carico il valore di r0 nella variabile "running" dello Scheduler
        // 4. Instruction Synchronization Barrier, come da manuale ARM
        "mrs    r0, psp",
        "stmfd  r0!, {{r4-r11}}",
        "ldr    r1, =SCHEDULER",
        "str	r0, [r1]",
        "isb",
        /* Check per determinare se c'è un nuovo processo da caricare */
        "ldr    r0, [r1, #4]",
        // Viene comparato a 0 perché è lo scheduler che azzera il puntatore, nel caso non ci sia un task pronto
        //"cmp    r0, #0",
        // Se non ci sono task da caricare, la CPU viene messa in sleep
        "beq     sleep_cpu",
        /* Caricamento del nuovo contesto */
        // 1. Load Multiple usando l'indirizzo puntato da r0. Modifica il valore di r0
        // 2. Carico il valore di r0 nella variabile "next" dello Scheduler
        // 3. Carico il valore di r0 nel Process Stack usize
        // 4. Instruction Synchronization Barrier, come da manuale ARM
        "ldr    r0, [r1, #4]",
        "ldmfd  r0!, {{r4-r11}}",
        "str    r0, [r1, #4]",
        "msr	psp, r0",
        "isb",
        /* Ritorno al thread, con PSP e in modo non privilegiato */
        "ldr    r0, =0xFFFFFFFD",
        "bx     r0",
        //sym SCHEDULER,
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

#[naked]
#[no_mangle]
pub unsafe extern "C" fn SVCall() {
    asm!("svc   0", options(noreturn));
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
