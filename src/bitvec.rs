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

use core::sync::atomic::{AtomicU32, Ordering};
use core::mem::MaybeUninit;
use core::fmt::{Binary, Display, Formatter};

#[cfg(feature = "BitVecUsize")]
type VecType = usize;

#[cfg(feature = "BitVec32")]
type VecType = u32;
#[cfg(feature = "BitVec32")]
type AtomicVecType = AtomicU32;

#[cfg(feature = "BitVec64")]
type VecType = u64;


/// BitVector, alias an unsigned number that is used to store an array of booleans
/// This simple implementation focuses on bit operations and manipulation
#[derive(Debug, Clone, Copy)]
pub struct BitVec(VecType);

impl BitVec {
    /// Get number of bits
    pub const BITS: usize = VecType::BITS as usize;

    /// Highest bit index, counted starting at BIT.0
    pub const HIGHEST_BIT: usize = VecType::BITS as usize - 1;

    /// Mask of all ones
    pub const MASK: VecType = VecType::MAX;

    /// Create a new vector
    pub const fn new() -> Self {
        BitVec(0)
    }

    pub const fn init(vec: VecType) -> Self {
        BitVec(vec)
    }

    /// Sets a single bit in the vector
    #[inline]
    pub const fn set(&mut self, bit: usize) -> &mut Self {
        self.0 = self.0 | (1 << bit);
        self
    }

    /// Clears a single bit in the vector
    #[inline]
    pub const fn clear(&mut self, bit: usize) -> &mut Self {
        self.0 = self.0 & !(1 << bit);
        self
    }

    /// Toggles a single bit in the vector
    #[inline]
    pub const fn toggle(&mut self, bit: usize) -> &mut Self {
        self.0 = self.0 ^ (1 << bit);
        self
    }

    /// Sets first zero bit (LSB)
    #[inline]
    pub const fn set_first_zero(&mut self) -> &mut Self {
        if let Ok(bit) = self.find_first_zero() {
            self.set(bit);
        }
        self
    }

    /// Clears first one bit (LSB)
    #[inline]
    pub const fn clear_first_one(&mut self) -> &mut Self {
        if let Ok(bit) = self.find_first_set() {
            self.clear(bit);
        }
        self
    }

    /// Checks if a bit is set
    #[inline]
    pub const fn check(&self, bit: usize) -> bool {
        (self.0 & (1 << bit)) != 0
    }

    /// Get vector as raw number
    #[inline]
    pub const fn raw(&self) -> VecType {
        self.0
    }

    /// Create a copy with reversed bit order
    #[inline]
    pub const fn reverse(&self) -> Self {
        BitVec(self.0.reverse_bits())
    }

    /// Sets vector to zero
    #[inline]
    pub const fn reset(&mut self) {
        self.0 = 0;
    }

    /// Inverts (toggles) all bits in vector. Can be concatenated with others operations.
    #[inline]
    pub const fn invert(&mut self) -> &mut Self {
        self.0 = self.0 ^ Self::MASK;
        self
    }

    /// Bitand operation. Can be concatenated with others operations.
    #[inline]
    pub const fn and(&mut self, rhs: &Self) -> &mut Self {
        self.0 = self.0 & rhs.0;
        self
    }

    /// Bitor operation. Can be concatenated with others operations.
    #[inline]
    pub const fn or(&mut self, rhs: &Self) -> &mut Self {
        self.0 = self.0 | rhs.0;
        self
    }

    /// Bitxor operation. Can be concatenated with others operations.
    #[inline]
    pub const fn xor(&mut self, rhs: &Self) -> &mut Self {
        self.0 = self.0 ^ rhs.0;
        self
    }

    /// Returns the bits in 'pattern' that are not active in 'self'
    #[inline]
    pub const fn difference(&self, pattern: &Self) -> Self {
        BitVec((self.0 ^ pattern.0) & pattern.0)
    }

    /// Returns true if there is one or more bits setted
    #[inline]
    pub const fn is_populated(&self) -> bool {
        !self.is_empty()
    }

    /// Returns true if vector is empty
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Checks if vector cointais at least one of pattern's bits
    #[inline]
    pub const fn intersects(&self, pattern: &Self) -> bool {
        BitVec(self.0 & pattern.0).is_populated()
    }

    /// Checks if whole pattern is contained into self
    #[inline]
    pub const fn superset_of(&self, pattern: &Self) -> bool {
        self.difference(pattern).is_empty()
    }

    #[inline]
    pub const fn count_ones(&self) -> usize {
        self.0.count_ones() as usize
    }

    #[inline]
    pub const fn count_zeroes(&self) -> usize {
        self.0.count_zeros() as usize
    }

    #[inline]
    pub const fn find_highest_set(&self) -> Result<usize, ()> {
        if self.is_populated() == true {
            Ok(Self::HIGHEST_BIT - self.0.leading_zeros() as usize)
        } else {
            Err(())
        }
    }

    #[inline]
    pub const fn find_first_set(&self) -> Result<usize, ()> {
        if self.is_populated() {
            Ok(self.0.trailing_zeros() as usize)
        } else {
            Err(())
        }
    }

    #[inline]
    pub const fn find_first_zero(&self) -> Result<usize, ()> {
        let ones = self.0.trailing_ones() as usize;
        if ones <= Self::HIGHEST_BIT {
            Ok(ones)
        } else {
            Err(())
        }
    }
}

impl core::ops::BitAnd for BitVec {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        BitVec(self.0 & rhs.0)
    }
}

impl core::ops::BitAndAssign for BitVec {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 = self.0 & rhs.0;
    }
}

impl core::ops::BitOr for BitVec {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        BitVec(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for BitVec {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 = self.0 | rhs.0;
    }
}

impl PartialEq for BitVec {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Display for BitVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Binary for BitVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Binary::fmt(&self.0, f)
    }
}

impl From<VecType> for BitVec {
    fn from(value: VecType) -> Self {
        BitVec(value)
    }
}

impl IntoIterator for &BitVec {
    type Item = usize;

    type IntoIter = BitVecIter;

    fn into_iter(self) -> Self::IntoIter {
        BitVecIter { vec: self.clone() }
    }
}

pub struct BitVecIter {
    vec: BitVec,
}

impl Iterator for BitVecIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(bit) = self.vec.find_highest_set() {
            self.vec.clear(bit);
            Some(bit)
        } else {
            None
        }
    }
}

/// List of elements indexed by a BitVec.
/// I founded this data structure quite usefull in embedded systems,
/// where you may not want to dynamic allocate a standard list.
/// This is able to store a fixed number of elements, one for each bit of the vector.
/// Thanks to the BitVec you can know wich slot is used and how many are used,
/// or you can iterate on the filled slots only.
///
/// Some of the good points: this structure uses very little RAM because no tagged unions (aka Options)
/// are used to know if values are empty; this means one byte saved per element in the list.
/// No fields to keep track of lenght and space used, yet keeping this kind of infos with the BitVec.
///
/// The only thing is that operating at bit-level is somewhat CPU consuming.
/// I didn't do any benchmark, maybe them could be done to get an idea of how performances are.
pub struct BitList<U: Sized> {
    bv: BitVec,
    list: [MaybeUninit<U>; BitVec::BITS],
}

impl<U: Sized> BitList<U> {
    pub const SIZE: usize = BitVec::BITS;

    pub const fn new() -> Self {
        Self {
            bv: BitVec::new(),
            list: [const { MaybeUninit::zeroed() }; BitVec::BITS],
        }
    }

    pub const fn insert(&mut self, element: U) -> Result<usize, U> {
        if let Ok(idx) = self.bv.find_first_zero() {
            self.bv.set(idx);
            self.list[idx] = MaybeUninit::new(element);
            Ok(idx)
        } else {
            Err(element)
        }
    }

    pub const fn insert_at(&mut self, idx: usize, element: U) -> Result<usize, U> {
        let used = self.bv.check(idx);
        if idx < BitVec::BITS && !used {
            self.bv.set(idx);
            self.list[idx] = MaybeUninit::new(element);
            Ok(idx)
        } else {
            Err(element)
        }
    }

    pub const fn get(&self, idx: usize) -> Result<&U, ()> {
        if self.bv.check(idx) == false {
            Err(())
        } else {
            // This is safe as 'bv' grants that list[idx] has been initialized
            let res = unsafe { self.list[idx].assume_init_ref() };
            Ok(res)
        }
    }

    pub const fn get_mut(&mut self, idx: usize) -> Result<&mut U, ()> {
        if self.bv.check(idx) == false {
            Err(())
        } else {
            // This is safe as 'bv' grants that list[idx] has been initialized
            let res = unsafe { self.list[idx].assume_init_mut() };
            Ok(res)
        }
    }

    #[inline]
    pub const unsafe fn get_unchecked(&mut self, idx: usize) -> &mut U {
        self.list[idx].assume_init_mut()
    }

    pub const fn remove(&mut self, idx: usize) -> Result<U, ()> {
        if self.bv.check(idx) == false {
            Err(())
        } else {
            self.bv.clear(idx);
            // This is safe as 'bv' grants that list[idx] has been initialized
            let res = unsafe { self.list[idx].assume_init_read() };
            Ok(res)
        }
    }

    #[inline]
    /// Used prior to fuse two lists together to check if some element will be lost
    pub const fn intersects(&self, rhs: Self) -> bool {
        self.bv.intersects(&rhs.bv)
    }

    /// Fuses two lists together, leaving only one of them
    /// In case of intersection, gives priority to "self" list's elements over rhs
    /// Returns the number of wasted elements from second list
    pub fn fuse(&mut self, rhs: Self) -> usize {
        let fusing = self.bv.difference(&rhs.bv);
        let wasted = (self.bv & rhs.bv).count_ones();

        self.bv |= fusing;
        for id in fusing.into_iter() {
            self.list[id].write(unsafe { rhs.list[id].assume_init_read() });
        }

        wasted
    }

    #[inline]
    pub const fn space_left(&self) -> usize {
        self.bv.count_zeroes()
    }

    #[inline]
    pub const fn space_used(&self) -> usize {
        self.bv.count_ones()
    }

    #[inline]
    pub const fn size(&self) -> usize {
        Self::SIZE
    }
    
}

impl<'a, U: Sized> IntoIterator for &'a BitList<U> {
    type Item = (usize, &'a U);

    type IntoIter = BitListIter<'a, U>;

    fn into_iter(self) -> Self::IntoIter {
        BitListIter {
            vec: self.bv,
            items: self.list.as_slice(),
        }
    }
}

impl<'a, U: Sized> IntoIterator for &'a mut BitList<U> {
    type Item = (usize, &'a mut U);

    type IntoIter = BitListIterMut<'a, U>;

    fn into_iter(self) -> Self::IntoIter {
        BitListIterMut {
            vec: self.bv,
            items: self.list.as_mut_slice(),
        }
    }
}

pub struct BitListIter<'a, U: Sized> {
    vec: BitVec,
    items: &'a [MaybeUninit<U>],
}

impl<'a, U: Sized> Iterator for BitListIter<'a, U> {
    type Item = (usize, &'a U);

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(bit) = self.vec.find_highest_set() {
            self.vec.clear(bit);
            Some((bit, unsafe { self.items[bit].assume_init_ref() }))
        } else {
            None
        }
    }
}

pub struct BitListIterMut<'a, U: Sized> {
    vec: BitVec,
    items: &'a mut [MaybeUninit<U>],
}

impl<'a, U: Sized> Iterator for BitListIterMut<'a, U> {
    type Item = (usize, &'a mut U);

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(bit) = self.vec.find_highest_set() {
            self.vec.clear(bit);
            // Using [transmute] to fix lifetimes
            Some((bit, unsafe { core::mem::transmute(self.items[bit].assume_init_mut()) }))
        } else {
            None
        }
    }
}


#[derive(Debug)]
pub struct AtomicBitVec(AtomicVecType);

impl AtomicBitVec {
    /// Get number of bits
    pub const BITS: usize = VecType::BITS as usize;

    /// Highest bit index, counted starting at BIT.0
    pub const HIGHEST_BIT: usize = VecType::BITS as usize - 1;

    /// Mask of all ones
    pub const MASK: VecType = VecType::MAX;

    /// Create a new vector
    pub const fn new() -> Self {
        AtomicBitVec(AtomicVecType::new(0))
    }

    pub const fn init(vec: VecType) -> Self {
        AtomicBitVec(AtomicVecType::new(vec))
    }

    /// Sets a single bit in the vector
    #[inline]
    pub fn write_raw(&self, val: VecType) -> &Self {
        self.0.store(val, Ordering::Relaxed);
        self
    }

    /// Sets a single bit in the vector
    #[inline]
    pub fn set(&self, bit: usize) -> &Self {
        self.0.fetch_or(1 << bit, Ordering::Relaxed);
        self
    }

    /// Clears a single bit in the vector
    #[inline]
    pub fn clear(&self, bit: usize) -> &Self {
        self.0.fetch_and(!(1 << bit), Ordering::Relaxed);
        self
    }

    /// Toggles a single bit in the vector
    #[inline]
    pub fn toggle(&self, bit: usize) -> &Self {
        self.0.fetch_xor(1 << bit, Ordering::Relaxed);
        self
    }

    /// Sets first zero bit (LSB)
    #[inline]
    pub fn set_first_zero(&self) -> &Self {
        if let Ok(bit) = self.find_first_zero() {
            self.set(bit);
        }
        self
    }

    /// Clears first one bit (LSB)
    #[inline]
    pub fn clear_first_one(&self) -> &Self {
        if let Ok(bit) = self.find_first_set() {
            self.clear(bit);
        }
        self
    }

    /// Checks if a bit is set
    #[inline]
    pub fn check(&self, bit: usize) -> bool {
        self.raw() & (1 << bit) != 0
    }

    /// Get vector as raw number
    #[inline]
    pub fn raw(&self) -> VecType {
        self.0.load(Ordering::Relaxed)
    }

    /// Create a copy with reversed bit order
    #[inline]
    pub fn reverse(&self) -> Self {
        AtomicBitVec(self.raw().reverse_bits().into())
    }

    /// Sets vector to zero
    #[inline]
    pub fn reset(&self) {
        self.0.store(0, Ordering::Relaxed);
    }

    /// Inverts (toggles) all bits in vector. Can be concatenated with others operations.
    #[inline]
    pub fn invert(&self) -> &Self {
        self.0.fetch_xor(Self::MASK, Ordering::Relaxed);
        self
    }

    /// Bitand operation. Can be concatenated with others operations.
    #[inline]
    pub fn and(&self, rhs: &Self) -> &Self {
        self.0.fetch_and(rhs.raw(), Ordering::Relaxed);
        self
    }

    /// Bitand operation. Can be concatenated with others operations.
    #[inline]
    pub fn and_raw(&self, rhs: VecType) -> &Self {
        self.0.fetch_and(rhs, Ordering::Relaxed);
        self
    }

    /// Bitor operation. Can be concatenated with others operations.
    #[inline]
    pub fn or(&self, rhs: &Self) -> &Self {
        self.0.fetch_or(rhs.raw(), Ordering::Relaxed);
        self
    }

    /// Bitor operation. Can be concatenated with others operations.
    #[inline]
    pub fn or_raw(&self, rhs: VecType) -> &Self {
        self.0.fetch_or(rhs, Ordering::Relaxed);
        self
    }

    /// Bitxor operation. Can be concatenated with others operations.
    #[inline]
    pub fn xor(&self, rhs: &Self) -> &Self {
        self.0.fetch_xor(rhs.raw(), Ordering::Relaxed);
        self
    }

    /// Bitxor operation. Can be concatenated with others operations.
    #[inline]
    pub fn xor_raw(&self, rhs: VecType) -> &Self {
        self.0.fetch_xor(rhs, Ordering::Relaxed);
        self
    }

    /// Returns the bits in 'pattern' that are not active in 'self'
    #[inline]
    pub fn difference(&self, pattern: &Self) -> Self {
        let val = self.raw();
        let mask = pattern.raw();
        
        AtomicBitVec::init((val ^ mask) & mask)
    }

    /// Returns true if there is one or more bits setted
    #[inline]
    pub fn is_populated(&self) -> bool {
        !self.is_empty()
    }

    /// Returns true if vector is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw() == 0
    }

    /// Checks if vector cointais at least one of pattern's bits
    #[inline]
    pub fn intersects(&self, pattern: &Self) -> bool {
        let val = self.raw();
        let mask = pattern.raw();

        AtomicBitVec::init(val & mask).is_populated()
    }

    /// Checks if whole pattern is contained into self
    #[inline]
    pub fn superset_of(&self, pattern: &Self) -> bool {
        self.difference(pattern).is_empty()
    }

    #[inline]
    pub fn count_ones(&self) -> usize {
        self.raw().count_ones() as usize
    }

    #[inline]
    pub fn count_zeroes(&self) -> usize {
        self.raw().count_zeros() as usize
    }

    #[inline]
    pub fn find_highest_set(&self) -> Result<usize, ()> {
        if self.is_populated() == true {
            Ok(Self::HIGHEST_BIT - self.raw().leading_zeros() as usize)
        } else {
            Err(())
        }
    }

    #[inline]
    pub fn find_first_set(&self) -> Result<usize, ()> {
        if self.is_populated() {
            Ok(self.raw().trailing_zeros() as usize)
        } else {
            Err(())
        }
    }

    #[inline]
    pub fn find_first_zero(&self) -> Result<usize, ()> {
        let ones = self.raw().trailing_ones() as usize;
        if ones <= Self::HIGHEST_BIT {
            Ok(ones)
        } else {
            Err(())
        }
    }
}

impl IntoIterator for &AtomicBitVec {
    type Item = usize;

    type IntoIter = AtomicBitVecIter;

    fn into_iter(self) -> Self::IntoIter {
        AtomicBitVecIter { vec: AtomicBitVec::init(self.0.load(Ordering::Relaxed)) }
    }
}


pub struct AtomicBitVecIter {
    vec: AtomicBitVec,
}

impl Iterator for AtomicBitVecIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(bit) = self.vec.find_highest_set() {
            self.vec.clear(bit);
            Some(bit)
        } else {
            None
        }
    }
}