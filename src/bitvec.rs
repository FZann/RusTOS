use core::fmt::{Binary, Display, Formatter};

/// BitVector, alias an unsigned number that is used to store an array of booleans
/// This simple implementation focuses on bit operations and manipulation
#[derive(Debug, Clone, Copy)]
pub struct BitVec(VecType);
type VecType = usize;

impl BitVec {
    /// Get number of bits, knowing byte-size and multiply by 8
    pub const BITS: usize = core::mem::size_of::<VecType>() << 3;

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
        if let Ok(bit) = self.find_first_zero() {
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
    pub const fn intersects(&self, pattern: &BitVec) -> bool {
        BitVec(self.0 & pattern.0).is_populated()
    }

    /// Checks if whole pattern is contained into self
    #[inline]
    pub const fn superset_of(&self, pattern: &BitVec) -> bool {
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
    
    pub const fn find_higher_set(&self) -> Result<usize, ()> {
        if self.is_populated() {
            Ok(Self::BITS - 1 - self.0.leading_zeros() as usize)
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
        if ones <= 31 {
            Ok(ones)
        } else {
            Err(())
        }
    }

}


impl core::ops::BitAnd for BitVec {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        BitVec(self.0 & rhs.0)
    }
}

impl core::ops::BitAndAssign for BitVec {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 = self.0 & rhs.0;
    }
}

impl core::ops::BitOr for BitVec {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        BitVec(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for BitVec {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 = self.0 | rhs.0;
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
        BitVecIter {
            vec: self.clone()
        }
    }
}

pub struct BitVecIter {
    vec: BitVec
}

impl Iterator for BitVecIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(bit) = self.vec.find_higher_set() {
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
pub struct BitList<U: Sized + Default>
{
    bv: BitVec,
    list: [U; BitVec::BITS],
}

impl<U: Sized + Default + Copy> BitList<U> {
    pub fn new() -> Self {
        Self {
            bv: BitVec::new(),
            list: [U::default(); BitVec::BITS],
        }
    }

    pub fn insert(&mut self, element: U) -> Result<usize, ()> {
        let idx = self.bv.find_first_zero()?;
        if idx < BitVec::BITS {
            self.bv.set(idx);
            self.list[idx] = element;
            Ok(idx)
        } else {
            Err(())
        }
    }

    pub fn insert_at(&mut self, idx: usize, element: U) -> Result<usize, ()> {
        let used = self.bv.check(idx);
        if idx < BitVec::BITS && !used {
            self.bv.set(idx);
            self.list[idx] = element;
            Ok(idx)
        } else {
            Err(())
        }
    }


    pub fn peek(&self, idx: usize) -> Result<&U, ()> {
        if self.bv.check(idx) == false {
            Err(())
        } else {
            let res = &self.list[idx];
            Ok(res)
        }
    }

    pub fn remove(&mut self, idx: usize) -> Result<&U, ()> {
        if self.bv.check(idx) == false {
            Err(())
        } else {
            self.bv.clear(idx);
            let res = &self.list[idx];
            Ok(res)
        }
    }

    /// Used prior to fuse two lists together to check if some element will be lost
    pub fn intersects(&self, rhs: Self) -> bool {
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
            self.list[id] = rhs.list[id];
        }

        wasted
    }

    pub fn space_left(&self) -> usize {
        self.bv.count_zeroes()
    }

    pub fn space_used(&self) -> usize {
        self.bv.count_ones()
    }

}


pub struct BitListIter<'a, U: Sized> {
    vec: BitVec,
    items: &'a [U],
}

impl<'a, U: Sized> Iterator for BitListIter<'a, U> {
    type Item = (usize, &'a U);

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(bit) = self.vec.find_higher_set() {
            self.vec.clear(bit);
            Some((bit, &self.items[bit]))
        } else {
            None
        }
    }
}


impl<'a, U: Sized + Default> IntoIterator for &'a BitList<U> {
    type Item = (usize, &'a U);

    type IntoIter = BitListIter<'a, U>;

    fn into_iter(self) -> Self::IntoIter {
        BitListIter {
            vec: self.bv,
            items: &self.list.as_slice(),
        }
    }
}

