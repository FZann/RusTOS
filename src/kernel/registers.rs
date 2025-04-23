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

use core::marker::PhantomData;

use crate::kernel::CritSect;


#[derive(Clone, Copy)]
pub struct BitMask<const BITS: usize, const POS: usize>;

impl<const BITS: usize, const POS: usize> BitMask<BITS, POS> {
    pub const fn new() -> Self {
        if POS as u32 >= usize::BITS {
            panic!("Bit position out of bounds");
        }

        BitMask
    }

    pub const fn mask(&self) -> usize {
        let res = 0;
        res
    }

    pub const fn pos(&self) -> usize {
        POS
    }
}


pub(crate) struct RO<const ADR: usize, const OFF: usize>;

impl<const ADR: usize, const OFF: usize> RO<ADR, OFF> {
    const PTR: *mut usize = (ADR + OFF) as *mut usize;

    pub const fn new() -> Self {
        Self
    }

    pub const fn addr(&self) -> usize {
        ADR + OFF
    }
    
    #[inline(always)]
    pub const fn ptr(&self) -> *const usize {
        Self::PTR
    }

    #[inline(always)]
    pub fn read(&self) -> usize {
        unsafe { Self::PTR.read_volatile() }
    }

    #[inline(always)]
    pub fn read_bit(&self, bit: usize) -> bool {
        let r = self.read();
        (r & (1 << bit)) != 0
    }

    #[inline(always)]
    pub fn check(&self, mask: usize) -> bool {
        let r = self.read();
        (r & mask) != 0
    }
}

pub(crate) struct RW<const ADR: usize, const OFF: usize>;

impl<const ADR: usize, const OFF: usize> RW<ADR, OFF> {
    const PTR: *mut usize = (ADR + OFF) as *mut usize;

    pub const fn new() -> Self {
        Self
    }

    pub const fn addr(&self) -> usize {
        ADR + OFF
    }

    #[inline(always)]
    pub const fn ptr(&self) -> *const usize {
        Self::PTR
    }

    #[inline(always)]
    pub fn read(&self) -> usize {
        unsafe { Self::PTR.read_volatile() }
    }

    #[inline(always)]
    pub fn read_bit(&self, bit: usize) -> bool {
        let r = self.read();
        (r & (1 << bit)) != 0
    }

    #[inline(always)]
    pub fn check(&self, mask: usize) -> bool {
        let r = self.read();
        (r & mask) != 0
    }

    #[inline(always)]
    pub fn write(&self, mask: usize) {
        unsafe {
            Self::PTR.write_volatile(mask);
        }
    }

    #[inline(always)]
    pub fn write_bit(&self, bit: usize, val: bool) {
        let r = self.read();
        let not: usize = (!val).into();
        let v: usize = val.into();
        // Branch-less setting/clearing del bit
        self.write(r & !(not << bit));
        self.write(r | v << bit);
    }

    #[inline(always)]
    pub fn modify<F>(&self, f: F)
        where F: FnOnce(usize) -> usize
    {
        self.write(f(self.read()));
    }

    #[inline(always)]
    pub fn set(&self, mask: usize) {
        let r = self.read();
        self.write(r | mask);
    }

    #[inline(always)]
    pub fn set_bit(&self, bit: usize) {
        let r = self.read();
        self.write(r | (1 << bit));
    }

    #[inline(always)]
    pub fn clear(&self, mask: usize) {
        let r = self.read();
        self.write(r & !mask);
    }

    #[inline(always)]
    pub fn clear_bit(&self, bit: usize) {
        let r = self.read();
        self.write(r & !(1 << bit));
    }
}


pub(crate) struct WO<const ADR: usize, const OFF: usize>;

impl<const ADR: usize, const OFF: usize> WO<ADR, OFF> {
    const PTR: *mut usize = (ADR + OFF) as *mut usize;

    pub const fn new() -> Self {
        Self
    }

    pub const fn addr(&self) -> usize {
        ADR + OFF
    }
    
    #[inline(always)]
    pub const fn ptr(&self) -> *const usize {
        Self::PTR
    }

    #[inline(always)]
    pub fn write(&self, val: usize) {
        unsafe {
            Self::PTR.write_volatile(val);
        }
    }

    #[inline(always)]
    pub fn set_bit(&self, bit: usize) {
        self.write(1 << bit);
    }
}


pub(crate) struct RWArea<const ADR: usize, const OFF: usize, const WORDS: usize>;

impl<const ADR: usize, const OFF: usize, const WORDS: usize> RWArea<ADR, OFF, WORDS> {
    const PTR: *mut usize = (ADR + OFF) as *mut usize;

    pub const fn new() -> Self {
        Self
    }

    #[inline(always)]
    pub const fn ptr(&self) -> *const usize {
        Self::PTR
    }

    #[inline(always)]
    pub fn read(&self, word: usize) -> usize {
        unsafe { Self::PTR.add(word).read_volatile() }
    }

    #[inline(always)]
    pub fn read_bit(&self, word: usize, bit: usize) -> bool {
        let r = self.read(word);
        (r & (1 << bit)) != 0
    }

    #[inline(always)]
    pub fn write(&self, word: usize, val: usize) {
        unsafe {
            Self::PTR.add(word).write_volatile(val);
        }
    }

    #[inline(always)]
    pub fn modify<const W: usize, F>(&self, word: usize, f: F)
        where F: FnOnce(usize) -> usize
    {
        self.write(word, f(self.read(word)));
    }

    #[inline(always)]
    pub fn set(&self, word: usize, mask: usize) {
        let r = self.read(word);
        self.write(word, r | mask);
    }

    #[inline(always)]
    pub fn set_bit(&self, word: usize, bit: usize) {
        let r = self.read(word);
        self.write(word, r | (1 << bit));
    }

    #[inline(always)]
    pub fn clear(&self, word: usize, mask: usize) {
        let r = self.read(word);
        self.write(word, r & !mask);
    }

    #[inline(always)]
    pub fn clear_bit(&self, word: usize, bit: usize) {
        let r = self.read(word);
        self.write(word, r & !(1 << bit));
    }
}


pub(crate) struct RWChannels<const ADR: usize, const OFF: usize, const NUM: usize, T>(PhantomData<T>);


impl<const ADR: usize, const OFF: usize, const NUM: usize, T> RWChannels<ADR, OFF, NUM, T> {
    const PTR: *mut T = (ADR + OFF) as *mut T;

    pub const fn new() -> Self {
        Self(PhantomData)
    }

    #[inline(always)]
    pub const fn ptr(&self) -> *const T {
        Self::PTR
    }

    pub const fn channel(&self, ch: usize) -> &mut T {
        unsafe { &mut *Self::PTR.add(ch) }
    }
}

pub(crate) trait Peripheral {
    type Registers;
    const ADR: usize;
    
    #[inline(always)]
    fn regs<'r>() -> &'r mut Self::Registers {
        unsafe { core::mem::transmute(Self::ADR) }
    }

    #[inline(always)]
    fn access<'r>(&self, _cs: &CritSect) -> &'r mut Self::Registers {
        Self::regs()
    }

    #[inline(always)]
    fn access_unchecked<'r>(&self) -> &'r mut Self::Registers {
        Self::regs()
    }

    /// Esecuzione di una funzione racchiusa in una critical section
    fn with(mut f: impl FnMut(&CritSect, &mut Self::Registers)) {
        let cs = CritSect::activate();
        f(&cs, Self::regs());
        drop(cs);
    }
}



