use core::cmp::Ordering;
use core::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

use crate::kernel::{SystemTicks, KERNEL};

use super::Ticks;

#[derive(Clone, Copy)]
pub struct Instant {
    t: SystemTicks,
}

impl Instant {
    pub fn now() -> Instant {
        let mut t = 0;
        KERNEL.with(|_, k| t = k.ticks);
        Instant { t }
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Self::Output {
        Instant {
            t: self.t.saturating_add(rhs.t.into()),
        }
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, rhs: Duration) {
        self.t = self.t.saturating_add(rhs.t.into());
    }
}

impl Sub for Instant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        Duration {
            t: self.t.saturating_sub(rhs.t) as u32,
        }
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, rhs: Duration) -> Self::Output {
        Instant {
            t: self.t.saturating_sub(rhs.t.into()),
        }
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, rhs: Duration) {
        self.t = self.t.saturating_sub(rhs.t.into());
    }
}

impl Eq for Instant {}
impl PartialEq for Instant {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t
    }
}

impl PartialOrd for Instant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.t.partial_cmp(&other.t)
    }
}

impl Ord for Instant {
    fn cmp(&self, other: &Self) -> Ordering {
        self.t.cmp(&other.t)
    }
}

#[derive(Clone, Copy)]
pub struct Duration {
    t: Ticks,
}

impl Duration {
    pub const fn new(t: Ticks) -> Self {
        Duration { t }
    }

    #[inline]
    pub const fn ticks(&self) -> Ticks {
        self.t
    }
}

impl Into<Ticks> for Duration {
    fn into(self) -> Ticks {
        self.t
    }
}

#[derive(Clone, Copy)]
pub struct CountDown {
    period: Duration,
    cnt: Ticks,
}

impl CountDown {
    pub const fn new(period: Duration) -> Self {
        Self {
            period,
            cnt: period.t,
        }
    }

    #[inline]
    pub const fn decrement(&mut self) {
        self.cnt = self.cnt.saturating_sub(1);
    }

    #[inline]
    pub const fn is_expired(&self) -> bool {
        self.cnt == 0
    }

    #[inline]
    pub const fn reload(&mut self) {
        self.cnt = self.period.t;
    }
}

pub struct Deadline {
    period: Duration,
    deadline: Instant,
}

impl Deadline {
    pub fn new(period: Duration) -> Self {
        Self {
            period,
            deadline: Instant::now() + period,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.deadline >= Instant::now()
    }

    pub fn reload(&mut self) {
        self.deadline += self.period;
    }
}

pub struct MHz(usize);

impl MHz {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    pub const  fn period_ns(&self) -> ns {
        ns(1_000 / self.0)
    }
}

impl From<kHz> for MHz {
    fn from(value: kHz) -> Self {
        Self(value.0 / 1_000)
    }
}

impl From<Hz> for MHz {
    fn from(value: Hz) -> Self {
        Self(value.0 / 1_000_000)
    }
}

impl Div for MHz {
    type Output = Hz;

    fn div(self, rhs: Self) -> Self::Output {
        Hz(self.0 / rhs.0)
    }
}

impl Div<kHz> for MHz {
    type Output = kHz;

    fn div(self, rhs: kHz) -> Self::Output {
        let mhz: Hz = self.into();
        let hz: Hz = rhs.into();
        (mhz / hz).into()
    }
}

impl Div<Hz> for MHz {
    type Output = Hz;

    fn div(self, rhs: Hz) -> Self::Output {
        let mhz: Hz = self.into();
        let hz: Hz = rhs.into();
        (mhz / hz).into()
    }
}

#[allow(non_camel_case_types)]
pub struct kHz(usize);

impl kHz {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    pub const  fn period_us(&self) -> us {
        us(1_000 / self.0)
    }

    pub const  fn period_ns(&self) -> ns {
        ns(1_000_000 / self.0)
    }
}

impl From<MHz> for kHz {
    fn from(value: MHz) -> Self {
        Self(value.0 * 1_000)
    }
}

impl From<Hz> for kHz {
    fn from(value: Hz) -> Self {
        Self(value.0 / 1_000)
    }
}

impl Div for kHz {
    type Output = Hz;

    fn div(self, rhs: Self) -> Self::Output {
        let mhz: Hz = self.into();
        let hz: Hz = rhs.into();
        (mhz / hz).into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hz(usize);

impl Hz {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    pub const fn period_ms(&self) -> ms {
        ms(1_000 / self.0)
    }

    pub const  fn period_us(&self) -> us {
        us(1_000_000 / self.0)
    }

    pub const  fn period_ns(&self) -> ns {
        ns(1_000_000_000 / self.0)
    }
}

impl From<Hz> for usize {
    fn from(value: Hz) -> Self {
        value.0
    }
}

impl From<u16> for Hz {
    fn from(value: u16) -> Self {
        Self(value.into())
    }
}

impl From<u32> for Hz {
    fn from(value: u32) -> Self {
        Self(value as usize)
    }
}

impl From<usize> for Hz {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<MHz> for Hz {
    fn from(value: MHz) -> Self {
        Self(value.0 * 1_000_000)
    }
}

impl From<kHz> for Hz {
    fn from(value: kHz) -> Self {
        Self(value.0 * 1_000)
    }
}

impl Add for Hz {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Hz(self.0 + rhs.0)
    }
}

impl Sub for Hz {
    type Output = Self;
    
    fn sub(self, rhs: Self) -> Self::Output {
        Hz(self.0.saturating_sub(rhs.0))
    }
}

impl Mul for Hz {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Hz(self.0 * rhs.0)
    }
}

impl Div for Hz {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Hz(self.0 / rhs.0)
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct ms(usize);

impl ms {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }
}

impl Add for ms {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        ms(self.0 + rhs.0)
    }
}

impl Add<us> for ms {
    type Output = Self;

    fn add(self, rhs: us) -> Self::Output {
        ms(self.0.saturating_add(rhs.0 / 1000))
    }
}

impl Sub for ms {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        ms(self.0.saturating_sub(rhs.0))
    }
}

impl Sub<us> for ms {
    type Output = Self;

    fn sub(self, rhs: us) -> Self::Output {
        ms(self.0.saturating_sub(rhs.0 / 1000))
    }
}

impl From<ms> for usize {
    fn from(value: ms) -> Self {
        value.0
    }
}

impl From<ms> for u32 {
    fn from(value: ms) -> Self {
        value.0 as u32
    }
}

#[allow(non_camel_case_types)]
pub struct us(usize);

impl us {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }
}

impl Add for us {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        us(self.0 + rhs.0)
    }
}

impl Add<ms> for us {
    type Output = Self;

    fn add(self, rhs: ms) -> Self::Output {
        us(self.0.saturating_add(rhs.0 * 1000))
    }
}

impl Sub for us {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        us(self.0.saturating_sub(rhs.0))
    }
}

impl Sub<ms> for us {
    type Output = Self;

    fn sub(self, rhs: ms) -> Self::Output {
        us(self.0.saturating_sub(rhs.0 * 1000))
    }
}

#[allow(non_camel_case_types)]
pub struct ns(usize);

impl ns {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }
}

impl Add for ns {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        ns(self.0 + rhs.0)
    }
}

impl Add<us> for ns {
    type Output = Self;

    fn add(self, rhs: us) -> Self::Output {
        ns(self.0.saturating_add(rhs.0 * 1_000))
    }
}

impl Add<ms> for ns {
    type Output = Self;

    fn add(self, rhs: ms) -> Self::Output {
        ns(self.0.saturating_add(rhs.0 * 1_000_000))
    }
}

impl Sub for ns {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        ns(self.0.saturating_sub(rhs.0))
    }
}

impl Sub<us> for ns {
    type Output = Self;

    fn sub(self, rhs: us) -> Self::Output {
        ns(self.0.saturating_sub(rhs.0 * 1_000))
    }
}

impl Sub<ms> for ns {
    type Output = Self;

    fn sub(self, rhs: ms) -> Self::Output {
        ns(self.0.saturating_sub(rhs.0 * 1_000_000))
    }
}