//! RusTOS - Rust Real Time Operating System 
//! Copyright (C) 2025 - Fabio Zanin - fabio.zanin93@outlook.com
//! 
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License.
//! 
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU General Public License for more details.
//! 
//! You should have received a copy of the GNU General Public License
//! along with this program.  If not, see <https://www.gnu.org/licenses/>.


use core::arch::{asm, naked_asm};
use core::marker::PhantomData;

use crate::hw::CPU_FREQUENCY;
use crate::kernel::Task;
use crate::kernel::CritSect;
use crate::kernel::SysCallType;
use crate::kernel::ExecContext;
use crate::kernel::Vector;
use crate::kernel::{Kernel, KERNEL};
use crate::kernel::registers::*;

/// Stack frame hardware saved by Cortex-M CPUs
/// Permits to visualize register values before last exception
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

    #[cfg(feature = "fpu_enabled")]
    fpu_regs: [u32; 16],
    #[cfg(feature = "fpu_enabled")]
    fpscr: u32,
    #[cfg(feature = "fpu_enabled")]
    reserved: u32,
}

/// TODO: use this to save Task context inside TCB
#[derive(Debug)]
#[repr(C)]
pub struct CpuContext {
    r4: u32,
    r5: u32,
    r6: u32,
    r7: u32,
    r8: u32,
    r9: u32,
    r10: u32,
    r11: u32,
    sp: u32,
    #[cfg(armv8m)]
    psplim: u32,
}

impl CpuContext {
    pub const fn new() -> Self {
        Self {
            r4: 4,
            r5: 5,
            r6: 6,
            r7: 7,
            r8: 8,
            r9: 9,
            r10: 10,
            r11: 11,
            sp : 0,
        }
    }

    #[inline(always)]
    pub(crate) unsafe fn save(&self) {
        #[cfg(armv6m)]
        asm!(
            "stm    {0}, {{r4-r7}}",
            "mov    r4, r8",
            "mov    r5, r9",
            "mov    r6, r10",
            "mov    r7, r11",
            "stm    {1}, {{r4-r7}}",
            "mrs    r1, psp",
            "str    r1, [{2}]",
            in(reg) &self.r4,
            in(reg) &self.r8,
            in(reg) &self.sp,
        );
        #[cfg(not(armv6m))]
        asm!(
            "stm    {0}, {{r4-r11}}",
            "mrs    r1, psp",
            "str    r1, [{1}]",
            in(reg) &self.r4,
            in(reg) &self.sp,
        );
    }

    #[inline(always)]
    pub(crate) unsafe fn load(&self) {
        #[cfg(armv6m)]
        asm!(
            "ldm    {0}, {{r4-r7}}",
            "mov    r8, r4",
            "mov    r9, r5",
            "mov    r10, r6",
            "mov    r11, r7",
            "ldm    {1}, {{r4-r7}}",
            "ldr    r1, [{2}]",
            "msr    r1, psp",
            in(reg) &self.r8,
            in(reg) &self.r4,
            in(reg) &self.sp,
        );
        #[cfg(not(armv6m))]
        asm!(
            "ldm    {0}, {{r4-r11}}",
            "ldr    r1, [{1}]",
            "msr    r1, psp",
            in(reg) &self.r4,
            in(reg) &self.sp,
        );
        #[cfg(armv8m)]
        asm!(
            "ldr    r2, [{0}]",
            "msr    r2, psplim",
            in(reg) &self.psplim,
        );
    }
}

#[cfg(has_fpu)]
#[cfg(feature = "fpu_enabled")]
pub struct FpuContext {
    s16: u32,
    s17: u32,
    s18: u32,
    s19: u32,
    s20: u32,
    s21: u32,
    s22: u32,
    s23: u32,
    s24: u32,
    s25: u32,
    s26: u32,
    s27: u32,
    s28: u32,
    s29: u32,
    s30: u32,
    s31: u32,
}

#[cfg(has_fpu)]
#[cfg(feature = "fpu_enabled")]
impl FpuContext {
    pub const fn new() -> Self {
        Self {
            s16: 0,
            s17: 0,
            s18: 0,
            s19: 0,
            s20: 0,
            s21: 0,
            s22: 0,
            s23: 0,
            s24: 0,
            s25: 0,
            s26: 0,
            s27: 0,
            s28: 0,
            s29: 0,
            s30: 0,
            s31: 0,
        }
    }
}

#[cfg(feature = "mpu_enabled")]
pub struct MpuContext {
    s0: u32,
}


//*********************************************************************************************************************
// EXCEPTIONS CPU VECTORS
//*********************************************************************************************************************
#[derive(Clone, Copy)]
pub enum Exceptions {
    NonMaskableInt = 2,

    HardFault = 3,

    #[cfg(not(armv6m))]
    MemoryManagement = 4,

    #[cfg(not(armv6m))]
    BusFault = 5,

    #[cfg(not(armv6m))]
    UsageFault = 6,

    #[cfg(armv8m)]
    SecureFault = 7,

    SVCall = 11,

    #[cfg(not(armv6m))]
    DebugMonitor = 12,

    PendSV = 14,

    SysTick = 15,
}

impl Exceptions {
    fn number(self) -> u16 {
        self as u16
    }
}

#[repr(C)]
pub enum IntPrio {
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
        handler: HardFault,
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

#[unsafe(naked)]
#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "C" fn __ENTRY() {
    naked_asm!(
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
        "b	    OSEntry",
    );
}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn NonMaskableInt() {
    loop {

    }
}

#[unsafe(naked)]
#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "C" fn HardFault() {
    naked_asm!(
        // Ottiene la &TCB running
        "ldr    r3, =KERNEL",
        "ldr    r1, [r3, #0]",
        "mrs    r0, PSP",
        // Gestione dell'errore da parte di Rust
        "b      OSHardFault",
    );
}

#[unsafe(naked)]
#[no_mangle]
#[cfg(not(armv6m))]
#[allow(non_snake_case)]
unsafe extern "C" fn MemoryManagement() {
    naked_asm!(
        // Ottiene la &TCB running
        "ldr    r3, =KERNEL",
        "ldr    r1, [r3, #0]",
        "mrs    r0, PSP",
        // Gestione dell'errore da parte di Rust
        "b      OSMemoryFault",
    );
}

#[unsafe(naked)]
#[no_mangle]
#[cfg(not(armv6m))]
#[allow(non_snake_case)]
unsafe extern "C" fn BusFault() {
    naked_asm!(
        // Ottiene la &TCB running
        "ldr    r3, =KERNEL",
        "ldr    r1, [r3, #0]",
        "mrs    r0, PSP",
        // Gestione dell'errore da parte di Rust
        "b      OSBusFault",
    );
}

#[unsafe(naked)]
#[no_mangle]
#[cfg(not(armv6m))]
#[allow(non_snake_case)]
unsafe extern "C" fn UsageFault() {
    naked_asm!(
        // Ottiene la &TCB running
        "ldr    r3, =KERNEL",
        "ldr    r1, [r3, #0]",
        "mrs    r0, PSP",
        // Gestione dell'errore da parte di Rust
        "b      OSUsageFault",
    );
}

#[unsafe(naked)]
#[no_mangle]
#[cfg(armv8m)]
#[allow(non_snake_case)]
unsafe extern "C" fn SecureFault() {
    naked_asm!(
        // Ottiene la &TCB running
        "ldr    r3, =KERNEL",
        "ldr    r1, [r3, #0]",
        "mrs    r0, PSP",
        // Gestione dell'errore da parte di Rust
        "b      OSSecureFault",
    );
}


/// We are at maximum of NVIC priority, this code can't be interrupted at all.
#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn SVCall() {
    let k = unsafe { KERNEL.access_unsafe() };
    let syscall: SysCallType = k.sys_call;
    match syscall {
        SysCallType::Nop => (),
        SysCallType::StartScheduler => k.start(),
        SysCallType::ContextSwitch => k.core.scb.set_pendsv(),
    };
}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn DebugMonitor() {
    loop {
        
    }
}

#[unsafe(naked)]
#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "C" fn PendSV() {
    naked_asm!(
        "cpsid	i",
        "ldr    r0, =KERNEL",           // Get &Scheduler

        "bl      switch_to_next",

        "ldr    lr, =0xFFFFFFFD",
        "cpsie	i",
        "bx     lr",
    );
}

#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn SysTick() {
    let cs = CritSect::activate();
    KERNEL.access(&cs).inc_system_ticks();
    KERNEL.access(&cs).schedule_next(cs);
}

//***************************************************************************************************************
// KERNEL ASSEMBLY
//***************************************************************************************************************
impl Kernel {
    #[inline(always)]
    pub(crate) fn interrupt_disable() {
        unsafe {
            asm!("cpsid i");
        }
    }
    
    #[inline(always)]
    pub(crate) fn interrupt_enable() {
        unsafe {
            asm!("cpsie i");
        }
    }
    
    #[inline(always)]
    pub(crate) fn nop() {
        unsafe {
            asm!("nop");
        }
    }
    
    #[inline(always)]
    pub(crate) fn core_sleep() {
        unsafe {
            asm!("wfi");
        }
    }

    /// To implement SysCalls with arguments and return values, we should
    /// add CpuContext to TCB before, as that is foundamental to have!
    /// To make arguments, we must save Task state and then we could assing regs 
    /// to syscall's arguments.
    /// To make return values, we must restore state and then assign regs to 
    /// syscall's return values; otherwise we could modify task's saved state
    /// directly, and then do a simple context restore.
    /// SysCalls should be at higher priority than other things (beside other exceptions)
    /// to be sure an other ISR do not interfere with arguments/return registers.
    /// SysCalls could be implemented... but: are they useful?
    /// A project's objective is to have HW SysCalls, but I am thinking hard about that:
    /// HW is readily accessible from Non-Privileged context, so why have SysCall?
    /// We could have driver tasks that R/W directly into peripheral memory without SysCalls.
    /// Perhaps we could implement a "Driver Task List" into the Kernel and parse that before
    /// users tasks... so user could implement its drivers, but in a predefined way.
    /// Don't know. Many things to study and be aware of.
    /// Best thing is to try and see what is it's outcome.
    #[inline(always)]
    #[allow(non_snake_case)]
    pub(crate) fn SystemCall(sys_call: SysCallType) {
        unsafe {
            KERNEL.access_unsafe().sys_call = sys_call;
    
            match sys_call {
                SysCallType::Nop => (),
                SysCallType::StartScheduler => asm!("svc    1"),
                SysCallType::ContextSwitch => asm!("svc  2"),
            }
        }
    }

    #[inline(always)]
    pub(crate) fn request_context_switch(&self) {
        if Kernel::get_context().is_privileged() {
            self.core.scb.set_pendsv();
        } else {
            Kernel::SystemCall(SysCallType::ContextSwitch);
        }
    }


    pub(crate) unsafe fn start_task(task: &Task, _cs: CritSect) -> ! {
        task.load_context();
        asm!(
            // Going back to thread, using PSP and in non-privileged mode
            "ldr    lr, =0xFFFFFFFD",
            "cpsie	i",
            "bx     lr",
            options(noreturn)
        );
    }

    #[inline(always)]
    pub(crate) extern "C" fn get_context() -> ExecContext {
        let val: usize;
        unsafe {
            asm!(
                "mrs    {out}, CONTROL",
                out = out(reg) val,
            );
            val.into()
        }
    }
}

impl Task {
    pub(crate) fn setup(&mut self) {
        let pointer = &raw const *self as usize;
        let stack = unsafe { &mut *(self.stack as *mut [usize]) };
        let len = stack.len();

        stack[len - 01] = 1 << 24; // xPSR - Thumb state active
        stack[len - 02] = self.task as usize; // PC
        stack[len - 03] = 0xFFFFFFFD; // LR
        stack[len - 04] = 0xC; // R12
        stack[len - 05] = 0x3; // R3
        stack[len - 06] = 0x2; // R2
        stack[len - 07] = 0x1; // R1
        stack[len - 08] = pointer; // R0

        // Software stack; this is not strictly necessary, but it is useful for debugging
        stack[len - 09] = 0xB; // R11
        stack[len - 10] = 0xA; // R10
        stack[len - 11] = 0x9; // R9
        stack[len - 12] = 0x8; // R8
        stack[len - 13] = 0x7; // R7
        stack[len - 14] = 0x6; // R6
        stack[len - 15] = 0x5; // R5
        stack[len - 16] = 0x4; // R4

        self.sp = (&stack[len - 16] as *const usize) as usize;
        self.stack_start = (&stack[len - 01] as *const usize) as usize;
        
        #[cfg(armv8m)] 
        {
            self.context.psplim = self.stack as u32;
        }
    }

    #[inline(always)]
    pub(crate) unsafe fn save_context(&self) {
        #[cfg(armv6m)]
        asm!(
            "mrs    r3, psp",               // Take PSP value out to r3
            "stmfd  r3!, {{r4-r7}}",        // Save Context
            "mov    r4, r8",
            "mov    r5, r9",
            "mov    r6, r10",
            "mov    r7, r11",
            "stmfd  r3!, {{r4-r7}}",
            "str	r3, [{0}]",             // Save PSP value in &StackPointer (same as &Task)
            in(reg) &self.sp
        );
        #[cfg(not(armv6m))]
        asm!(
            "mrs    r3, psp",               // Take PSP value out to r3
            "stmfd  r3!, {{r4-r11}}",       // Save Context
            "str	r3, [{0}]",             // Save PSP value in &StackPointer (same as &Task)
            in(reg) &self.sp
        );
    }

    #[inline(always)]
    pub(crate) unsafe fn load_context(&self) {
        #[cfg(armv6m)]
        asm!(
            "ldr    r3, [{0}]",             // Get value of StackPointer
            "ldmfd  r3!, {{r4-r7}}",        // Load Context
            "mov    r11, r7",
            "mov    r10, r6",
            "mov    r9, r5",
            "mov    r8, r4",
            "ldmfd  r3!, {{r4-r7}}",       
            "str    r3, [{0}]",             // Saves new StackPointer value in &Task
            "msr	psp, r3",               // Moves StackPointer in PSP
            in(reg) &self.sp
        );
        #[cfg(not(armv6m))]
        asm!(
            "ldr    r3, [{0}]",             // Get value of StackPointer
            "ldmfd  r3!, {{r4-r11}}",       // Load Context
            "str    r3, [{0}]",             // Saves new Stackpointer value in &PCB
            "msr	psp, r3",               // Moves r3 in PSP
            "isb",
            in(reg) &self.sp
        );
    }

}

//***************************************************************************************************************
// CORE PERIPHERALS
//***************************************************************************************************************

pub enum ClockSource {
    ExternalClock = 0,
    CoreClock = 1 << 2,
}

pub struct CorePeripherals {
    systick: SysTickTimer,
    nvic: NVIC,
    scb: SCB,
    mpu: PhantomData<u32>,
    fpu: PhantomData<u32>,
}

impl CorePeripherals {
    pub const fn new() -> Self {
        Self {
            systick: SysTickTimer::new(),
            nvic: NVIC::new(),
            scb: SCB::new(),
            mpu: PhantomData,
            fpu: PhantomData,
        }
    }

    pub fn setup(&mut self) {
        self.nvic.enable_interrupt(Exceptions::HardFault);
        self.nvic.set_interrupt_prio(Exceptions::HardFault, IntPrio::Max);

        #[cfg(not(armv6m))]
        self.nvic.enable_interrupt(Exceptions::UsageFault);
        #[cfg(not(armv6m))]
        self.nvic.set_interrupt_prio(Exceptions::UsageFault, IntPrio::Pri01);

        #[cfg(not(armv6m))]
        self.nvic.enable_interrupt(Exceptions::MemoryManagement);
        #[cfg(not(armv6m))]
        self.nvic.set_interrupt_prio(Exceptions::MemoryManagement, IntPrio::Pri01);

        #[cfg(not(armv6m))]
        self.nvic.enable_interrupt(Exceptions::BusFault);
        #[cfg(not(armv6m))]
        self.nvic.set_interrupt_prio(Exceptions::BusFault, IntPrio::Pri01);

        #[cfg(armv8m)]
        self.nvic.enable_interrupt(Exceptions::SecureFault);
        #[cfg(armv8m)]
        self.nvic.set_interrupt_prio(Exceptions::SecureFault, IntPrio::Pri01);

        self.nvic.enable_interrupt(Exceptions::SVCall);
        self.nvic.set_interrupt_prio(Exceptions::SVCall, IntPrio::Max);

        self.nvic.enable_interrupt(Exceptions::PendSV);
        self.nvic.set_interrupt_prio(Exceptions::PendSV, IntPrio::Min);
        
        self.nvic.enable_interrupt(Exceptions::SysTick);
        self.nvic.set_interrupt_prio(Exceptions::SysTick, IntPrio::Pri14);

        self.systick.init();
    }

    #[inline]
    pub fn sleep_on_exit(&self, sleep: bool) {
        self.scb.sleep_on_exit(sleep);
    }

    pub fn get_irq_num(&self) -> usize {
        let val: usize;
        unsafe {
            asm!(
                "mrs    {out}, IPSR",
                out = out(reg) val,
            );
            val
        }
    }

    pub fn set_basepri(&self, val: usize) {
        unsafe {
            asm!(
                "msr    BASEPRI, {in}",
                in = in(reg) val,
            );
        }
    }

    pub fn get_basepri(&self) -> usize {
        let val: usize;
        unsafe {
            asm!(
                "mrs    {out}, BASEPRI",
                out = out(reg) val,
            );
            val
        }
    }

}


const SYSTICK_ADR: usize = 0xE000_E010;
struct SysTickTimer {
    crs: RW<SYSTICK_ADR, 0x00>,
    rvr: RW<SYSTICK_ADR, 0x04>,
    cvr: RW<SYSTICK_ADR, 0x08>,
    calib: RW<SYSTICK_ADR, 0x0C>,
}

impl SysTickTimer {
    const ENABLE: usize = 1;
    const TICKINT: usize = 1 << 1;
    const CLKSOURCE: usize = 1 << 2;
    //const SKEW: usize = 1 << 30;
    const TENMS_MASK: usize = 0x00FF_FFFF;

    const fn new() -> Self {
        Self { 
            crs: RW::new(),
            rvr: RW::new(),
            cvr: RW::new(), 
            calib: RW::new(),
         }
    }

    fn start(&mut self) {
        self.crs.set(Self::ENABLE);
    }

    fn stop(&mut self) {
        self.crs.clear(Self::ENABLE);
    }

    fn set_clocksource(&mut self, cksrc: ClockSource) -> &mut Self {
        self.crs.clear(Self::CLKSOURCE);
        self.crs.set(cksrc as usize);
        self
    }

    fn int_enable(&mut self) -> &mut Self {
        self.crs.set(Self::TICKINT);
        self
    }

    fn init(&mut self) {
        self.stop();
        let cpu: crate::kernel::Hz = CPU_FREQUENCY.into();
        let cpu: usize = cpu.into();
        let reload = cpu / 1000;
        
        self.set_reload(reload).zero_count();
        self.set_clocksource(ClockSource::CoreClock)
            .int_enable()
            .start();
    }

    fn zero_count(&mut self) -> &mut Self {
        self.cvr.write(0);
        self
    }

    fn set_reload(&mut self, reload: usize) -> &mut Self {
        self.rvr.write(reload as usize);
        self
    }

    fn get_calibration(&mut self) -> usize {
        // let skew = !((self.cvr & Self::SKEW) == Self::SKEW);
        let tenms = self.calib.read() & Self::TENMS_MASK;
        tenms
    }
    
}


const NVIC_ADR: usize = 0xE000_E100;
struct NVIC {
    iser: RWArea<NVIC_ADR, 0x000, 8>,
    icer: RWArea<NVIC_ADR, 0x080, 8>,
    ispr: RWArea<NVIC_ADR, 0x100, 8>,
    icpr: RWArea<NVIC_ADR, 0x180, 8>,
    iabr: RWArea<NVIC_ADR, 0x200, 8>,
    ipr: RWArea<NVIC_ADR, 0x300, 60>,
    stir: WO<NVIC_ADR, 0xE00>,
}

impl NVIC {
    const fn new() -> Self {
        Self {
            iser: RWArea::new(),
            icer: RWArea::new(),
            ispr: RWArea::new(),
            icpr: RWArea::new(),
            iabr: RWArea::new(),
            ipr: RWArea::new(),
            stir: WO::new(),
        }
    }

    fn enable_interrupt(&mut self, int: Exceptions) {
        let n = int.number() as usize;
        self.iser.set_bit(n >> 5, n & 0x1F);
    }

    fn disable_interrupt(&mut self, int: Exceptions) {
        let n = int.number() as usize;
        self.icer.set_bit(n >> 5, n & 0x1F);
    }

    fn pend_interrupt(&mut self, int: Exceptions) {
        let n = int.number() as usize;
        self.ispr.set_bit(n >> 5, n & 0x1F);
    }

    fn clear_interrupt(&mut self, int: Exceptions) {
        let n = int.number() as usize;
        self.icpr.set_bit(n >> 5, n & 0x1F);
    }

    fn is_interrupt_active(&self, int: Exceptions) -> bool {
        let n = int.number() as usize;
        self.icpr.read_bit(n >> 5, n & 0x1F)
    }

    fn set_interrupt_prio(&mut self, int: Exceptions, prio: IntPrio) {
        let n = int.number() as usize; // Divide per 4
        let pos = (n + n) & 0x1F;
        self.ipr.set(n >> 5, prio.value() << pos);
    }
}

const SCB_ADDR: usize = 0xE000E000;
struct SCB {
    actrl: RW<SCB_ADDR,0x008>,
    cpuid: RO<SCB_ADDR,0xD00>,
    icsr: RW<SCB_ADDR,0xD04>,
    vtor: RW<SCB_ADDR,0xD08>,
    aircr: RW<SCB_ADDR,0xD0C>,
    scr: RW<SCB_ADDR,0xD10>,
    ccr: RW<SCB_ADDR,0xD14>,
    shpr1: RW<SCB_ADDR,0xD18>,
    shpr2: RW<SCB_ADDR,0xD1C>,
    shpr3: RW<SCB_ADDR,0xD20>,
    shcrs: RW<SCB_ADDR,0xD24>,
    cfsr: RW<SCB_ADDR,0xD28>,
    hfsr: RW<SCB_ADDR,0xD2C>,
    mmar: RW<SCB_ADDR,0xD34>,
    bfar: RW<SCB_ADDR,0xD38>,
    afsr: RW<SCB_ADDR,0xD3C>,
}

impl SCB {
    const ICSR_PENDSVSET_MASK: usize = 1 << 28;
    const SCR_SLEEPONEXIT: usize = 1;

    const fn new() -> Self {
        Self {
            actrl: RW::new(),
            cpuid: RO::new(),
            icsr: RW::new(),
            vtor: RW::new(),
            aircr: RW::new(),
            scr: RW::new(),
            ccr: RW::new(),
            shpr1: RW::new(),
            shpr2: RW::new(),
            shpr3: RW::new(),
            shcrs: RW::new(),
            cfsr: RW::new(),
            hfsr: RW::new(),
            mmar: RW::new(),
            bfar: RW::new(),
            afsr: RW::new(),
        }
    }
    
    #[inline]
    fn sleep_on_exit(&self, sleep: bool) {
        self.scr.write_bit(Self::SCR_SLEEPONEXIT, sleep);
    }

    #[inline]
    fn set_pendsv(&self) {
        self.icsr.write(Self::ICSR_PENDSVSET_MASK);
    }

}

