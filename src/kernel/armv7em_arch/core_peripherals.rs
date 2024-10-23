use core::marker::PhantomData;
use crate::peripherals::Peripheral;

crate::make_peripheral!(SysTickTimer: 0xE000_E010);
crate::make_peripheral!(NVIC: 0xE000_E100);

pub(crate) struct RO;
pub(crate) struct RW;
pub(crate) struct WO;

pub(crate) trait Access {}

impl Access for RO {}
impl Access for RW {}
impl Access for WO {}

pub(crate) struct Reg<const ADR: usize, T: Access>(PhantomData<T>);

impl<const ADR: usize, T: Access> Reg<ADR, T> {
    const PTR: *mut usize = ADR as *mut usize;

    const fn new() -> Self {
        Reg(PhantomData)
    }
}

impl<const ADR: usize> Reg<ADR, RO> {
    pub fn read(&self) -> usize {
        unsafe { *Self::PTR }
    }
}

impl<const ADR: usize> Reg<ADR, RW> {
    pub fn read(&self) -> usize {
        unsafe { Self::PTR.read_volatile() }
    }

    pub fn write(&self, val: usize) {
        unsafe { Self::PTR.write_volatile(val); }
    }
}

impl<const ADR: usize> Reg<ADR, WO> {
    pub fn write(&self, val: usize) {
        unsafe { Self::PTR.write_volatile(val); }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Interrupts {
    NonMaskableInt = 2,

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


pub(crate) struct CorePeripherals {
    systick: PhantomData<SysTickTimer>,
    nvic: PhantomData<NVIC>,
    scb: SCB,
    mpu: PhantomData<u32>,
    fpu: PhantomData<u32>,

    primask: PhantomData<u32>,
    faultmask: PhantomData<u32>,
    basepri: PhantomData<u32>,
    control: PhantomData<u32>
}

impl CorePeripherals {
    pub const fn new() -> Self {
        Self {
            systick: PhantomData,
            nvic: PhantomData,
            scb: SCB::new(),
            mpu: PhantomData,
            fpu: PhantomData,
            primask: PhantomData,
            faultmask: PhantomData,
            basepri: PhantomData,
            control: PhantomData,
        }
    }

    pub fn setup_os(&self) {
        let nvic = unsafe { NVIC::regs() };
        nvic.enable_interrupt(Interrupts::SVCall);
        nvic.enable_interrupt(Interrupts::PendSV);
        nvic.enable_interrupt(Interrupts::SysTick);
        nvic.set_interrupt_prio(Interrupts::SVCall, IntPrio::Max);
        nvic.set_interrupt_prio(Interrupts::SysTick, IntPrio::Max);
        nvic.set_interrupt_prio(Interrupts::PendSV, IntPrio::Min);
        
        let systick = unsafe { SysTickTimer::regs() };
        systick.init();
    }
    
    pub(crate) fn sleep_on_exit(&self, sleep: bool) {
        self.scb.scr.sleep_on_exit(sleep);
    }
}

/// Struttura dati effettiva sottostante allo ZST di accesso
#[repr(C)]
pub struct SysTickTimer {
    crs: u32,
    rvr: u32,
    cvr: u32,
    calib: u32,
}

impl SysTickTimer {
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
pub struct NVIC {
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

impl NVIC {
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

pub struct SCB {
    actrl: Reg<0xE000E008, RW>,
    cpuid: Reg<0xE000ED00, RO>,
    icsr: Reg<0xE000ED04, RW>,
    vtor: Reg<0xE000ED08, RW>,
    aircr: Reg<0xE000ED0C, RW>,
    scr: Reg<0xE000ED10, RW>,
    ccr: Reg<0xE000ED14, RW>,
    shpr1: Reg<0xE000ED18, RW>,
    shpr2: Reg<0xE000ED1C, RW>,
    shpr3: Reg<0xE000ED20, RW>,
    shcrs: Reg<0xE000ED24, RW>,
    cfsr: Reg<0xE000ED28, RW>,
    hfsr: Reg<0xE000ED2C, RW>,
    mmar: Reg<0xE000ED34, RW>,
    bfar: Reg<0xE000ED38, RW>,
    afsr: Reg<0xE000ED3C, RW>,
}

impl SCB {
    const fn new() -> Self {
        Self {
            actrl: Reg::new(),
            cpuid: Reg::new(),
            icsr: Reg::new(),
            vtor: Reg::new(),
            aircr: Reg::new(),
            scr: Reg::new(),
            ccr: Reg::new(),
            shpr1: Reg::new(),
            shpr2: Reg::new(),
            shpr3: Reg::new(),
            shcrs: Reg::new(),
            cfsr: Reg::new(),
            hfsr: Reg::new(),
            mmar: Reg::new(),
            bfar: Reg::new(),
            afsr: Reg::new(),
        }
    }
}

impl Reg<0xE000ED10, RW> {
    pub(crate) fn sleep_on_exit(&self, sleep: bool) {
        let val = (sleep as usize) << 1;
        self.write(val);
    }
}
