//! Bit manipulation used for wires and bit values.

/// A `u64` bit mask of `len` 1s,
/// useful for masking against values which
/// should not affect [`BitArray`] comparisons.
const fn norm_mask(len: u8) -> u64 {
    match len {
        l @ 0..64 => (1 << l) - 1,
        _ => u64::MAX
    }
}

/// Creates a u64 which has this bit repeating the entire bitspace.
const fn stretch(v: bool) -> u64 {
    if v { u64::MAX } else { 0 }
}

/// Shift type for [`BitArray::shift`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShiftType {
    /// Shift all bits left
    LogicalLeft,
    /// Shift all bits right
    LogicalRight,
    /// Shift all bits right and sign extend
    ArithmeticRight,
    /// Shift all bits left, using wraparound logic
    RotateLeft,
    /// Shift all bits right, using wraparound logic
    RotateRight,
}
impl ShiftType {
    /// Applies this shift type on `data` (of bitsize `bitsize`), creating a shift by `shift`.
    /// 
    /// Any bits not within the `bitsize` should be ignored.
    pub fn apply(self, data: u64, bitsize: u8, shift: u32) -> u64 {
        let mask = norm_mask(bitsize);
        match self {
            ShiftType::LogicalLeft => data.wrapping_shl(shift),
            ShiftType::LogicalRight => (data & mask).wrapping_shr(shift),
            ShiftType::ArithmeticRight => {
                data.cast_signed()
                    // Get sign
                    .wrapping_shl(u64::BITS - u32::from(bitsize))
                    .wrapping_shr(u64::BITS - u32::from(bitsize))
                    // Perform actual shift
                    .wrapping_shr(shift)
                    .cast_unsigned()
            },
            // Minor optimization
            ShiftType::RotateLeft if bitsize >= 64 => data.rotate_left(shift),
            ShiftType::RotateLeft => {
                let shift = shift % u32::from(bitsize);
                let left = data.wrapping_shl(shift);
                let right = (data & mask).wrapping_shr(u32::from(bitsize) - shift);

                left | right
            },
            // Minor optimization
            ShiftType::RotateRight if bitsize >= 64 => data.rotate_right(shift),
            ShiftType::RotateRight => {
                let shift = shift % u32::from(bitsize);
                let right = (data & mask).wrapping_shr(shift);
                let left = data.wrapping_shl(u32::from(bitsize) - shift);
                left | right
            },
        }
    }
}
/// The state of a single bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BitState {
    /// 0
    Low = 0b00,
    /// 1
    High = 0b01,
    /// High impedance (Z, z)
    Imped = 0b10,
    /// Unknown (X, x)
    Unk = 0b11
}
impl BitState {
    /// Splits bit state into a data bit and a special bit.
    /// 
    /// State | Special | Data |
    /// ------|---------|------|
    ///   Low |       0 |    0 |
    ///  High |       0 |    1 |
    /// Imped |       1 |    0 |
    ///   Err |       1 |    1 |
    pub(crate) fn split(self) -> (bool /* data */, bool /* spec */) {
        ((self as u8) & 0b01 != 0, (self as u8) & 0b10 != 0)
    }

    /// Joins a data and special bit into a bitstate.
    pub(crate) fn join(data: bool, spec: bool) -> Self {
        Self::from_bits((u8::from(spec) << 1) | u8::from(data))
    }
    /// Converts bits into a bitstate.
    /// (the 0th bit is the data bit, the 1st is the special bit),
    pub(crate) fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0b00 => BitState::Low,
            0b01 => BitState::High,
            0b10 => BitState::Imped,
            0b11 => BitState::Unk,
            _ => unreachable!()
        }
    }
}
impl std::fmt::Display for BitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        match self {
            BitState::Low   => f.write_char('0'),
            BitState::High  => f.write_char('1'),
            BitState::Imped => f.write_char('Z'),
            BitState::Unk   => f.write_char('X'),
        }
    }
}

/// An error which occurs when converting a non-low/-high [`BitState`] to [`bool`].
#[derive(Debug)]
pub struct NotTwoValuedErr(BitState);
impl NotTwoValuedErr {
    /// If the value was high-impedance
    pub fn is_imped(&self) -> bool { self.0 == BitState::Imped }
    /// If the value was unknown
    pub fn is_unk(&self) -> bool { self.0 == BitState::Unk }
    /// The bit state (either [`BitState::Imped`] or [`BitState::Unk`])
    pub fn bit_state(&self) -> BitState { self.0 }
}

/// An error which occurs when trying to set to a bitarray using one of a different length.
#[derive(Debug)]
pub struct MismatchedBitsizes(());

/// This error can occur when converting a [`char`] to a [`BitState`]
/// if the [`char`] does not map to any [`BitState`] character.
#[derive(Debug)]
pub struct InvalidStateErr(());

impl TryFrom<BitState> for bool {
    type Error = NotTwoValuedErr;

    fn try_from(value: BitState) -> Result<Self, Self::Error> {
        match value {
            BitState::Low   => Ok(false),
            BitState::High  => Ok(true),
            st => Err(NotTwoValuedErr(st))
        }
    }
}
impl From<bool> for BitState {
    fn from(value: bool) -> Self {
        match value {
            true => Self::High,
            false => Self::Low,
        }
    }
}
impl TryFrom<char> for BitState {
    type Error = InvalidStateErr;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '0' => Ok(BitState::Low),
            '1' => Ok(BitState::High),
            'Z' | 'z' => Ok(BitState::Imped),
            'X' | 'x' => Ok(BitState::Unk),
            _ => Err(InvalidStateErr(()))
        }
    }
}
impl std::str::FromStr for BitState {
    type Err = InvalidStateErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.as_bytes() {
            &[n] => char::from(n).try_into(),
            _ => Err(InvalidStateErr(()))
        }
    }
}
impl std::ops::BitAnd for BitState {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        // Identities:
        // F & a = F
        // T & Z = X
        // T & a = a
        // X, o.w.
        match (self, rhs) {
            (BitState::Low, _) | (_, BitState::Low) => BitState::Low,
            (BitState::High, BitState::Imped) | (BitState::Imped, BitState::High) => BitState::Unk,
            (BitState::High, a) | (a, BitState::High) => a,
            _ => BitState::Unk
        }
    }
}
impl std::ops::BitOr for BitState {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        // Identities:
        // F & Z = X
        // F & a = a
        // T & a = T
        // X, o.w.
        match (self, rhs) {
            (BitState::Low, BitState::Imped) | (BitState::Imped, BitState::Low) => BitState::Unk,
            (BitState::Low, a) | (a, BitState::Low) => a,
            (BitState::High, _) | (_, BitState::High) => BitState::High,
            _ => BitState::Unk
        }
    }
}
impl std::ops::BitXor for BitState {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        match Option::zip(bool::try_from(self).ok(), bool::try_from(rhs).ok()) {
            Some((a, b)) => Self::from(a ^ b),
            None => Self::Unk,
        }
    }
}
impl std::ops::Not for BitState {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            BitState::High  => Self::Low,
            BitState::Low   => Self::High,
            _               => Self::Unk,
        }
    }
}

/// An array of bitstates.
#[derive(Default, Clone, Copy)]
pub struct BitArray {
    data: u64,
    spec: u64,
    len: u8
}

/// Creates a [`BitState`].
/// 
/// This accepts the inputs 0, 1, Z, X.
#[macro_export]
macro_rules! bitstate {
    (0) => { $crate::bitarray::BitState::Low };
    (1) => { $crate::bitarray::BitState::High };
    (Z) => { $crate::bitarray::BitState::Imped };
    (X) => { $crate::bitarray::BitState::Unk };
}
/// Creates a [`BitArray`] containing the literal arguments.
/// 
/// Each element can be one of 0, 1, Z, X.
#[macro_export]
macro_rules! bitarr {
    [$b:tt; $e:expr] => { $crate::bitarray::BitArray::repeat($crate::bitarray::bitstate!($b), $e) };
    [$($b:tt),*$(,)?] => { $crate::bitarray::BitArray::from_iter([$($crate::bitarray::bitstate!($b)),*]) };
}
pub use bitstate;
pub use bitarr;

impl BitArray {
    /// The minimum possible size for the array.
    pub const MIN_BITSIZE: u8 = 1;
    /// The maximum possible size for the array.
    pub const MAX_BITSIZE: u8 = u64::BITS as u8;

    /// Create a new zero-sized array.
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates a new bitarray from specified two-valued data and a length.
    /// 
    /// - `data` represents the data of the bitarray (lows and highs), between bit 0 and bit `(len - 1)`
    /// - `len` represents the length of the bitarray
    /// 
    /// Any bits after index `(len - 1)` in `data` are ignored.
    pub fn from_bits(data: u64, len: u8) -> Self {
        Self { data, spec: 0, len: len.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE) }
    }

    /// Creates a new array where the bit state is repeated `len` times.
    pub fn repeat(st: BitState, len: u8) -> Self {
        let (data, spec) = st.split();
        Self {
            data: stretch(data),
            spec: stretch(spec),
            len: len.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE)
        }
    }

    /// The length of the array.
    pub const fn len(self) -> u8 {
        match self.len {
            ..BitArray::MIN_BITSIZE => BitArray::MIN_BITSIZE,
            len @ BitArray::MIN_BITSIZE..BitArray::MAX_BITSIZE => len,
            _ => BitArray::MAX_BITSIZE,
        }
    }
    /// Whether the array has no elements.
    pub fn is_empty(self) -> bool {
        self.len == 0
    }

    // MASKING

    /// A `u64` bit mask of `len` 1s,
    /// useful for masking against values which
    /// should not affect [`BitArray`] comparisons.
    const fn norm_mask(self) -> u64 {
        norm_mask(self.len())
    }
    /// Applies the mask on the `BitArray`,
    /// returning the resulting data and spec bits.
    pub(crate) fn normalize(self) -> (u64, u64) {
        let mask = self.norm_mask();
        (self.data & mask, self.spec & mask)
    }

    /// Creates a `u64` bit set indicating where all values that match `st` are.
    /// If the bit is 1 at a given index, then that index matches `st`.
    pub(crate) const fn is(self, st: BitState) -> u64 {
        (match st {
            BitState::Low   => !self.data & !self.spec,
            BitState::High  =>  self.data & !self.spec,
            BitState::Imped => !self.data &  self.spec,
            BitState::Unk   =>  self.data &  self.spec,
        }) & self.norm_mask()
    }
    /// Whether all bits in the bit array match `st`.
    pub(crate) fn all(self, st: BitState) -> bool {
        self.is(st) == self.norm_mask()
    }
    /// Whether any bit in the bit array match `st`.
    pub(crate) fn any(self, st: BitState) -> bool {
        self.is(st) != 0
    }

    /// Gets the BitState at the given (unchecked) index.
    fn get_raw(self, i: u8) -> BitState {
        let data = (self.data >> i) & 1 != 0;
        let spec = (self.spec >> i) & 1 != 0;
        BitState::join(data, spec)
    }
    /// Gets the BitState at the given index (or None, if `i >= self.len()`).
    pub fn get(self, i: u8) -> Option<BitState> {
        (i < self.len()).then(|| self.get_raw(i))
    }

    /// Sets the BitState at the given (unchecked) index.
    fn set_raw(&mut self, i: u8, st: BitState) {
        let (data, spec) = st.split();
        self.data &= !(1 << i);
        self.data |= u64::from(data) << i;
        self.spec &= !(1 << i);
        self.spec |= u64::from(spec) << i;
    }
    /// Sets the BitState at the given index if `i < self.len()`.
    fn set(&mut self, i: u8, st: BitState) {
        if i < self.len() {
            self.set_raw(i, st);
        }
    }
    pub(crate) fn replace(&mut self, value: BitArray) -> Result<(), MismatchedBitsizes> {
        if self.len() == value.len() {
            *self = value;
            Ok(())
        } else {
            Err(MismatchedBitsizes(()))
        }
    }
    /// Creates a new BitState with the state at index `i` replaced with `st`.
    pub fn with(mut self, i: u8, st: BitState) -> Self {
        self.set(i, st);
        self
    }

    /// Gets the BitState at the given index (panicking if `i >= self.len()`).
    pub fn index(self, i: u8) -> BitState {
        self.get(i).expect("index to be in bounds")
    }

    /// Creates a new bit array with the same bits but new bitsize.
    /// 
    /// If the new bitsize is larger than the current bitsize,
    /// it is extended with the specified array.
    pub fn resize(self, new_len: u8, fill: BitState) -> Self {
        let new_len = new_len.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        if new_len <= self.len() {
            // Truncation
            Self { data: self.data, spec: self.spec, len: new_len }
        } else {
            // Extension
            let (data, spec) = fill.split();
            let data = stretch(data);
            let spec = stretch(spec);
            let mask = self.norm_mask();
            Self {
                data: (data & !mask) | (self.data & mask),
                spec: (spec & !mask) | (self.spec & mask),
                len: new_len
            }
        }
    }

    /// Shifts the bitarray by the specified `shift`, using the applied `shift_type`.
    pub fn shift(self, shift: u32, shift_type: ShiftType) -> Self {
        let data = shift_type.apply(self.data, self.len, shift);
        let spec = shift_type.apply(self.spec, self.len, shift);

        Self { data, spec, len: self.len }
    }
}
impl FromIterator<BitState> for BitArray {
    fn from_iter<I: IntoIterator<Item = BitState>>(iter: I) -> Self {
        iter.into_iter()
            .zip(0..64)
            .fold(BitArray::new(), |mut arr, (st, i)| {
                arr.set_raw(i, st);
                arr.len += 1;
                arr
            })
    }
}
impl From<u64> for BitArray {
    fn from(data: u64) -> Self {
        Self::from_bits(data, 64)
    }
}
impl From<bool> for BitArray {
    fn from(value: bool) -> Self {
        Self::from_bits(u64::from(value), 1)
    }
}
impl TryFrom<BitArray> for u64 {
    type Error = NotTwoValuedErr;

    fn try_from(value: BitArray) -> Result<Self, Self::Error> {
        let (data, spec) = value.normalize();
        match spec == 0 {
            true => Ok(data),
            false => {
                let err_st = match !value.any(BitState::Unk) {
                    true => BitState::Imped,
                    false => BitState::Unk,
                };

                Err(NotTwoValuedErr(err_st))
            }
        }
    }
}

/// [`BitArray::into_iter`].
pub struct BitArrayIntoIter(BitArray);
impl Iterator for BitArrayIntoIter {
    type Item = BitState;

    fn next(&mut self) -> Option<Self::Item> {
        (!self.0.is_empty()).then(|| {
            let raw = self.0.get_raw(0);
            self.0.data >>= 1;
            self.0.spec >>= 1;
            self.0.len -= 1;
            raw
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}
impl DoubleEndedIterator for BitArrayIntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        (!self.0.is_empty()).then(|| {
            let raw = self.0.get_raw(self.0.len() - 1);
            self.0.len -= 1;
            raw
        })
    }
}
impl ExactSizeIterator for BitArrayIntoIter {
    fn len(&self) -> usize {
        usize::from(self.0.len())
    }
}
impl IntoIterator for BitArray {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = BitArrayIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        BitArrayIntoIter(self)
    }
}
impl IntoIterator for &BitArray {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = BitArrayIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (*self).into_iter()
    }
}

impl PartialEq for BitArray {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.normalize() == other.normalize()
    }
}
impl Eq for BitArray {}
impl std::hash::Hash for BitArray {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        self.normalize().hash(state);
    }
}
impl std::fmt::Debug for BitArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(*self)
            .finish()
    }
}
impl std::fmt::Display for BitArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bit in *self {
            write!(f, "{bit}")?;
        }
        Ok(())
    }
}

// assume same size
impl BitArray {
    /// Combines two bit arrays (as if two wires are connected).
    /// 
    /// A join operation is as follows:
    /// - Z join a = a
    /// - a join Z = a
    /// - anything else = X
    pub fn join(self, rhs: BitArray) -> BitArray {
        // TODO: assert size
        // __ | 00 | 01 | 10 | 11
        // 00 | 11 | 11 | 00 | 11
        // 01 | 11 | 11 | 01 | 11
        // 10 | 00 | 01 | 10 | 11
        // 11 | 11 | 11 | 11 | 11
        let len = self.len();
        let lz = self.is(BitState::Imped);
        let rz = rhs.is(BitState::Imped);

        let data = (!lz & !rz) | (lz & !rz & rhs.data) | (rz & self.data);
        let spec = (!lz & !rz) | (lz & !rz & rhs.spec) | (rz & self.spec);
        
        Self { data, spec, len }
    }

    pub(crate) fn short_circuits(self, occupied: u64) -> Option<u64> {
        let not_z = !self.is(BitState::Imped) & self.norm_mask();

        // Short circuit if multiple bits have non-Z
        (occupied & not_z == 0).then_some(occupied | not_z)
    }
}
impl From<BitState> for BitArray {
    fn from(value: BitState) -> Self {
        BitArray::repeat(value, 1)
    }
}
impl std::str::FromStr for BitArray {
    type Err = InvalidStateErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.bytes()
            .map(char::from)
            .map(BitState::try_from)
            .collect()
    }
}

impl std::ops::BitAnd for BitArray {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        // __ | 00 | 01 | 10 | 11
        // 00 | 00 | 00 | 00 | 00
        // 01 | 00 | 01 | 11 | 11
        // 10 | 00 | 11 | 11 | 11
        // 11 | 00 | 11 | 11 | 11

        let any_false = self.is(BitState::Low) | rhs.is(BitState::Low);
        let all_true = self.is(BitState::High) & rhs.is(BitState::High);

        let data = !any_false;
        let spec = !any_false & !all_true;
        let len = self.len();
        Self { spec, data, len }
    }
}
impl std::ops::BitOr for BitArray {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        // __ | 00 | 01 | 10 | 11
        // 00 | 00 | 01 | 11 | 11
        // 01 | 01 | 01 | 01 | 01
        // 10 | 11 | 01 | 11 | 11
        // 11 | 11 | 01 | 11 | 11

        let any_true = self.is(BitState::High) | rhs.is(BitState::High);
        let all_false = self.is(BitState::Low) & rhs.is(BitState::Low);

        let data = !all_false;
        let spec = !all_false & !any_true;
        let len = self.len;
        Self { spec, data, len }
    }
}
impl std::ops::BitXor for BitArray {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        // __ | 00 | 01 | 10 | 11
        // 00 | 00 | 01 | 11 | 11
        // 01 | 01 | 00 | 11 | 11
        // 10 | 11 | 11 | 11 | 11
        // 11 | 11 | 11 | 11 | 11
        let any_ntv = self.spec | rhs.spec;

        let data = any_ntv | (self.data ^ rhs.data);
        let spec = any_ntv | (self.spec ^ rhs.spec);
        let len = self.len;
        Self { spec, data, len }
    }
}
impl std::ops::Not for BitArray {
    type Output = Self;

    fn not(self) -> Self::Output {
        // 00 | 01
        // 01 | 00
        // 10 | 11
        // 11 | 11
        let spec = self.spec;
        let data = self.spec | !self.data;
        let len = self.len;
        Self { spec, data, len }
    }
}

#[cfg(test)]
mod tests {
    use crate::bitarray::{BitState, ShiftType, norm_mask};

    use super::BitArray;

    #[test]
    fn and_two_valued() {
        let a: u64 = 0x4cb3052010db508b;
        let b: u64 = 0x1a5b254efea6bcab;
        let result = a & b;

        let expected = BitArray::from(result);
        let actual = BitArray::from(a) & BitArray::from(b);
        assert_eq!(actual, expected);
    }
    #[test]
    fn or_two_valued() {
        let a: u64 = 0x6fc5f87e5cc83658;
        let b: u64 = 0x0c02737be3c85b62;
        let result = a | b;

        let expected = BitArray::from(result);
        let actual = BitArray::from(a) | BitArray::from(b);
        assert_eq!(actual, expected);
    }
    #[test]
    fn xor_two_valued() {
        let a: u64 = 0x97f7bd2f3d4a2aad;
        let b: u64 = 0x8769806a06948e4a;
        let result = a ^ b;

        let expected = BitArray::from(result);
        let actual = BitArray::from(a) ^ BitArray::from(b);
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse() {
        let str_ba = "0Z1X10XZ".parse::<BitArray>().unwrap();
        let iter_ba = BitArray::from_iter([
            BitState::Low,
            BitState::Imped,
            BitState::High,
            BitState::Unk,
            BitState::High,
            BitState::Low,
            BitState::Unk,
            BitState::Imped,
        ]);

        assert_eq!(str_ba, iter_ba);
    }
    #[test]
    fn collect_roundtrip() {
        let bits = [
            BitState::Low,
            BitState::Imped,
            BitState::High,
            BitState::Unk,
            BitState::High,
            BitState::Low,
            BitState::Unk,
            BitState::Imped,
        ];

        let bitarray = BitArray::from_iter(bits);
        assert_eq!(bitarray.into_iter().collect::<Vec<_>>(), bits);
    }

    #[test]
    fn display() {
        let ba = BitArray::from_iter([
            BitState::Low,
            BitState::Imped,
            BitState::High,
            BitState::Unk,
            BitState::High,
            BitState::Low,
            BitState::Unk,
            BitState::Imped,
        ]);

        assert_eq!(format!("{ba}"), "0Z1X10XZ");
    }

    #[test]
    fn resized() {
        let ba = bitarr![Z, 0, 1];
        assert_eq!(ba.resize(4, BitState::Imped), bitarr![Z, 0, 1, Z]);
        assert_eq!(ba.resize(4, BitState::Unk),   bitarr![Z, 0, 1, X]);
        assert_eq!(ba.resize(4, BitState::High),  bitarr![Z, 0, 1, 1]);
        assert_eq!(ba.resize(4, BitState::Low),   bitarr![Z, 0, 1, 0]);

        assert_eq!(ba.resize(1, BitState::High), bitarr![Z]);
    }

    #[test]
    fn shift_log_left() {
        const SHIFT: ShiftType = ShiftType::LogicalLeft;
        let data = 0b101101;
        let size = 6;
        let mask = norm_mask(size);
        assert_eq!(SHIFT.apply(data, size, 0) & mask, 0b101101);
        assert_eq!(SHIFT.apply(data, size, 1) & mask, 0b011010);
        assert_eq!(SHIFT.apply(data, size, 2) & mask, 0b110100);
        assert_eq!(SHIFT.apply(data, size, 3) & mask, 0b101000);
        assert_eq!(SHIFT.apply(data, size, 4) & mask, 0b010000);
        assert_eq!(SHIFT.apply(data, size, 5) & mask, 0b100000);
        assert_eq!(SHIFT.apply(data, size, 6) & mask, 0b000000);
        assert_eq!(SHIFT.apply(data, size, 7) & mask, 0b000000);

        let data = 0b01011;
        let size = 5;
        let mask = norm_mask(size);
        assert_eq!(SHIFT.apply(data, size, 0) & mask, 0b01011);
        assert_eq!(SHIFT.apply(data, size, 1) & mask, 0b10110);
        assert_eq!(SHIFT.apply(data, size, 2) & mask, 0b01100);
        assert_eq!(SHIFT.apply(data, size, 3) & mask, 0b11000);
        assert_eq!(SHIFT.apply(data, size, 4) & mask, 0b10000);
        assert_eq!(SHIFT.apply(data, size, 5) & mask, 0b00000);
        assert_eq!(SHIFT.apply(data, size, 6) & mask, 0b00000);
        assert_eq!(SHIFT.apply(data, size, 7) & mask, 0b00000);

        let data = 0b10110;
        assert_eq!(SHIFT.apply(data, 64, 0), 0b10110);
        assert_eq!(SHIFT.apply(data, 64, 1), 0b101100);
        assert_eq!(SHIFT.apply(data, 64, 2), 0b1011000);
        assert_eq!(SHIFT.apply(data, 64, 3), 0b10110000);
        assert_eq!(SHIFT.apply(data, 64, 4), 0b101100000);
        assert_eq!(SHIFT.apply(data, 64, 5), 0b1011000000);
        assert_eq!(SHIFT.apply(data, 64, 6), 0b10110000000);
        assert_eq!(SHIFT.apply(data, 64, 7), 0b101100000000);
    }

    #[test]
    fn shift_log_right() {
        const SHIFT: ShiftType = ShiftType::LogicalRight;
        let data = 0b101101;
        let size = 6;
        let mask = norm_mask(size);
        assert_eq!(SHIFT.apply(data, size, 0) & mask, 0b101101);
        assert_eq!(SHIFT.apply(data, size, 1) & mask, 0b010110);
        assert_eq!(SHIFT.apply(data, size, 2) & mask, 0b001011);
        assert_eq!(SHIFT.apply(data, size, 3) & mask, 0b000101);
        assert_eq!(SHIFT.apply(data, size, 4) & mask, 0b000010);
        assert_eq!(SHIFT.apply(data, size, 5) & mask, 0b000001);
        assert_eq!(SHIFT.apply(data, size, 6) & mask, 0b000000);
        assert_eq!(SHIFT.apply(data, size, 7) & mask, 0b000000);

        let data = 0b101011;
        let size = 5;
        let mask = norm_mask(size);
        assert_eq!(SHIFT.apply(data, size, 0) & mask, 0b01011);
        assert_eq!(SHIFT.apply(data, size, 1) & mask, 0b00101);
        assert_eq!(SHIFT.apply(data, size, 2) & mask, 0b00010);
        assert_eq!(SHIFT.apply(data, size, 3) & mask, 0b00001);
        assert_eq!(SHIFT.apply(data, size, 4) & mask, 0b00000);
        assert_eq!(SHIFT.apply(data, size, 5) & mask, 0b00000);
        assert_eq!(SHIFT.apply(data, size, 6) & mask, 0b00000);
        assert_eq!(SHIFT.apply(data, size, 7) & mask, 0b00000);

        let data = 0b10110;
        assert_eq!(SHIFT.apply(data, 64, 0), 0b10110);
        assert_eq!(SHIFT.apply(data, 64, 1), 0b1011);
        assert_eq!(SHIFT.apply(data, 64, 2), 0b101);
        assert_eq!(SHIFT.apply(data, 64, 3), 0b10);
        assert_eq!(SHIFT.apply(data, 64, 4), 0b1);
        assert_eq!(SHIFT.apply(data, 64, 5), 0b0);
        assert_eq!(SHIFT.apply(data, 64, 6), 0b0);
        assert_eq!(SHIFT.apply(data, 64, 7), 0b0);
    }

    #[test]
    fn shift_art_right() {
        const SHIFT: ShiftType = ShiftType::ArithmeticRight;
        
        let data = 0b001101;
        let size = 6;
        let mask = norm_mask(size);
        assert_eq!(SHIFT.apply(data, size, 0) & mask, 0b001101);
        assert_eq!(SHIFT.apply(data, size, 1) & mask, 0b000110);
        assert_eq!(SHIFT.apply(data, size, 2) & mask, 0b000011);
        assert_eq!(SHIFT.apply(data, size, 3) & mask, 0b000001);
        assert_eq!(SHIFT.apply(data, size, 4) & mask, 0b000000);
        assert_eq!(SHIFT.apply(data, size, 5) & mask, 0b000000);
        assert_eq!(SHIFT.apply(data, size, 6) & mask, 0b000000);
        assert_eq!(SHIFT.apply(data, size, 7) & mask, 0b000000);
        
        let data = 0b101101;
        let size = 6;
        let mask = norm_mask(size);
        assert_eq!(SHIFT.apply(data, size, 0) & mask, 0b101101);
        assert_eq!(SHIFT.apply(data, size, 1) & mask, 0b110110);
        assert_eq!(SHIFT.apply(data, size, 2) & mask, 0b111011);
        assert_eq!(SHIFT.apply(data, size, 3) & mask, 0b111101);
        assert_eq!(SHIFT.apply(data, size, 4) & mask, 0b111110);
        assert_eq!(SHIFT.apply(data, size, 5) & mask, 0b111111);
        assert_eq!(SHIFT.apply(data, size, 6) & mask, 0b111111);
        assert_eq!(SHIFT.apply(data, size, 7) & mask, 0b111111);

        let data = 0b11101011;
        let size = 5;
        let mask = norm_mask(size);
        assert_eq!(SHIFT.apply(data, size, 0) & mask, 0b01011);
        assert_eq!(SHIFT.apply(data, size, 1) & mask, 0b00101);
        assert_eq!(SHIFT.apply(data, size, 2) & mask, 0b00010);
        assert_eq!(SHIFT.apply(data, size, 3) & mask, 0b00001);
        assert_eq!(SHIFT.apply(data, size, 4) & mask, 0b00000);
        assert_eq!(SHIFT.apply(data, size, 5) & mask, 0b00000);
        assert_eq!(SHIFT.apply(data, size, 6) & mask, 0b00000);
        assert_eq!(SHIFT.apply(data, size, 7) & mask, 0b00000);

        let data = 0b1111011;
        let size = 5;
        let mask = norm_mask(size);
        assert_eq!(SHIFT.apply(data, size, 0) & mask, 0b11011);
        assert_eq!(SHIFT.apply(data, size, 1) & mask, 0b11101);
        assert_eq!(SHIFT.apply(data, size, 2) & mask, 0b11110);
        assert_eq!(SHIFT.apply(data, size, 3) & mask, 0b11111);
        assert_eq!(SHIFT.apply(data, size, 4) & mask, 0b11111);
        assert_eq!(SHIFT.apply(data, size, 5) & mask, 0b11111);
        assert_eq!(SHIFT.apply(data, size, 6) & mask, 0b11111);
        assert_eq!(SHIFT.apply(data, size, 7) & mask, 0b11111);

        let data = 0xF000_0000_0000_000A; // 1111 ... 1010
        assert_eq!(SHIFT.apply(data, 64, 0), 0xF000_0000_0000_000A);
        assert_eq!(SHIFT.apply(data, 64, 1), 0xF800_0000_0000_0005);
        assert_eq!(SHIFT.apply(data, 64, 2), 0xFC00_0000_0000_0002);
        assert_eq!(SHIFT.apply(data, 64, 3), 0xFE00_0000_0000_0001);
        assert_eq!(SHIFT.apply(data, 64, 4), 0xFF00_0000_0000_0000);
        assert_eq!(SHIFT.apply(data, 64, 5), 0xFF80_0000_0000_0000);
        assert_eq!(SHIFT.apply(data, 64, 6), 0xFFC0_0000_0000_0000);
        assert_eq!(SHIFT.apply(data, 64, 7), 0xFFE0_0000_0000_0000);
        assert_eq!(SHIFT.apply(data, 64, 8), 0xFFF0_0000_0000_0000);
    }

    #[test]
    fn shift_rot_left() {
        const SHIFT: ShiftType = ShiftType::RotateLeft;
        
        let data = 0b101100;
        let size = 6;
        let mask = norm_mask(size);
        assert_eq!(SHIFT.apply(data, size, 0) & mask, 0b101100);
        assert_eq!(SHIFT.apply(data, size, 1) & mask, 0b011001);
        assert_eq!(SHIFT.apply(data, size, 2) & mask, 0b110010);
        assert_eq!(SHIFT.apply(data, size, 3) & mask, 0b100101);
        assert_eq!(SHIFT.apply(data, size, 4) & mask, 0b001011);
        assert_eq!(SHIFT.apply(data, size, 5) & mask, 0b010110);
        assert_eq!(SHIFT.apply(data, size, 6) & mask, 0b101100);
        assert_eq!(SHIFT.apply(data, size, 7) & mask, 0b011001);

        let data = 0b11101011;
        let size = 5;
        let mask = norm_mask(size);
        assert_eq!(SHIFT.apply(data, size, 0) & mask, 0b01011);
        assert_eq!(SHIFT.apply(data, size, 1) & mask, 0b10110);
        assert_eq!(SHIFT.apply(data, size, 2) & mask, 0b01101);
        assert_eq!(SHIFT.apply(data, size, 3) & mask, 0b11010);
        assert_eq!(SHIFT.apply(data, size, 4) & mask, 0b10101);
        assert_eq!(SHIFT.apply(data, size, 5) & mask, 0b01011);
        assert_eq!(SHIFT.apply(data, size, 6) & mask, 0b10110);
        assert_eq!(SHIFT.apply(data, size, 7) & mask, 0b01101);

        let data = 0xF000_0000_0000_000A; // 1111 ... 1010
        assert_eq!(SHIFT.apply(data, 64, 0), 0xF000_0000_0000_000A);
        assert_eq!(SHIFT.apply(data, 64, 1), 0xE000_0000_0000_0015);
        assert_eq!(SHIFT.apply(data, 64, 2), 0xC000_0000_0000_002B);
        assert_eq!(SHIFT.apply(data, 64, 3), 0x8000_0000_0000_0057);
        assert_eq!(SHIFT.apply(data, 64, 4), 0x0000_0000_0000_00AF);
        assert_eq!(SHIFT.apply(data, 64, 5), 0x0000_0000_0000_015E);
        assert_eq!(SHIFT.apply(data, 64, 6), 0x0000_0000_0000_02BC);
        assert_eq!(SHIFT.apply(data, 64, 7), 0x0000_0000_0000_0578);
        assert_eq!(SHIFT.apply(data, 64, 8), 0x0000_0000_0000_0AF0);
    }

    #[test]
    fn shift_rot_right() {
        const SHIFT: ShiftType = ShiftType::RotateRight;
        
        let data = 0b101100;
        let size = 6;
        let mask = norm_mask(size);
        assert_eq!(SHIFT.apply(data, size, 0) & mask, 0b101100);
        assert_eq!(SHIFT.apply(data, size, 1) & mask, 0b010110);
        assert_eq!(SHIFT.apply(data, size, 2) & mask, 0b001011);
        assert_eq!(SHIFT.apply(data, size, 3) & mask, 0b100101);
        assert_eq!(SHIFT.apply(data, size, 4) & mask, 0b110010);
        assert_eq!(SHIFT.apply(data, size, 5) & mask, 0b011001);
        assert_eq!(SHIFT.apply(data, size, 6) & mask, 0b101100);
        assert_eq!(SHIFT.apply(data, size, 7) & mask, 0b010110);

        let data = 0b11101011;
        let size = 5;
        let mask = norm_mask(size);
        assert_eq!(SHIFT.apply(data, size, 0) & mask, 0b01011);
        assert_eq!(SHIFT.apply(data, size, 1) & mask, 0b10101);
        assert_eq!(SHIFT.apply(data, size, 2) & mask, 0b11010);
        assert_eq!(SHIFT.apply(data, size, 3) & mask, 0b01101);
        assert_eq!(SHIFT.apply(data, size, 4) & mask, 0b10110);
        assert_eq!(SHIFT.apply(data, size, 5) & mask, 0b01011);
        assert_eq!(SHIFT.apply(data, size, 6) & mask, 0b10101);
        assert_eq!(SHIFT.apply(data, size, 7) & mask, 0b11010);

        let data = 0xF000_0000_0000_000A; // 1111 ... 1010
        assert_eq!(SHIFT.apply(data, 64, 0), 0xF000_0000_0000_000A);
        assert_eq!(SHIFT.apply(data, 64, 1), 0x7800_0000_0000_0005);
        assert_eq!(SHIFT.apply(data, 64, 2), 0xBC00_0000_0000_0002);
        assert_eq!(SHIFT.apply(data, 64, 3), 0x5E00_0000_0000_0001);
        assert_eq!(SHIFT.apply(data, 64, 4), 0xAF00_0000_0000_0000);
        assert_eq!(SHIFT.apply(data, 64, 5), 0x5780_0000_0000_0000);
        assert_eq!(SHIFT.apply(data, 64, 6), 0x2BC0_0000_0000_0000);
        assert_eq!(SHIFT.apply(data, 64, 7), 0x15E0_0000_0000_0000);
        assert_eq!(SHIFT.apply(data, 64, 8), 0x0AF0_0000_0000_0000);
    }
}