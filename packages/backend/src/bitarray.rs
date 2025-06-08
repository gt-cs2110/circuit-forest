#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BitState {
    Low = 0b00, High = 0b01, Imped = 0b10, Unk = 0b11
}
impl BitState {
    pub(crate) fn split(self) -> (bool /* data */, bool /* spec */) {
        ((self as u8) & 0b01 != 0, (self as u8) & 0b10 != 0)
    }
    pub(crate) fn join(data: bool, spec: bool) -> Self {
        Self::try_from((u8::from(spec) << 1) | u8::from(data)).unwrap()
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

#[derive(Debug)]
pub struct NotTwoValuedErr(BitState);
impl NotTwoValuedErr {
    pub fn is_imped(&self) -> bool { self.0 == BitState::Imped }
    pub fn is_unk(&self) -> bool { self.0 == BitState::Unk }
    pub fn bit_state(&self) -> BitState { self.0 }
}

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
impl TryFrom<u8> for BitState {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b00 => Ok(BitState::Low),
            0b01 => Ok(BitState::High),
            0b10 => Ok(BitState::Imped),
            0b11 => Ok(BitState::Unk),
            _ => Err(())
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

#[derive(Default, Clone, Copy)]
pub struct BitArray {
    data: u64,
    spec: u64,
    len: u8
}
impl BitArray {
    pub const MIN_BITSIZE: u8 = 1;
    pub const MAX_BITSIZE: u8 = u64::BITS as u8;

    pub fn new() -> Self {
        Default::default()
    }
    pub fn repeat(st: BitState, len: u8) -> Self {
        let (data, spec) = st.split();
        Self {
            data: if data { u64::MAX } else { 0 },
            spec: if spec { u64::MAX } else { 0 },
            len: len.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE - 1)
        }
    }
    pub fn floating(len: u8) -> Self {
        Self::repeat(BitState::Imped, len)
    }
    pub fn unknown(len: u8) -> Self {
        Self::repeat(BitState::Unk, len)
    }

    pub const fn len(self) -> u8 {
        match self.len {
            ..BitArray::MIN_BITSIZE => BitArray::MIN_BITSIZE,
            len @ BitArray::MIN_BITSIZE..BitArray::MAX_BITSIZE => len,
            _ => BitArray::MAX_BITSIZE,
        }
    }
    pub fn is_empty(self) -> bool {
        self.len == 0
    }

    const fn norm_mask(self) -> u64 {
        match self.len() {
            len @ 0..64 => (1 << len) - 1,
            _ => u64::MAX
        }
    }
    pub(crate) fn normalize(self) -> (u64, u64) {
        let mask = self.norm_mask();
        (self.data & mask, self.spec & mask)
    }

    const fn is_0(self) -> u64 {
        !self.data & !self.spec & self.norm_mask()
    }
    const fn is_1(self) -> u64 {
        self.data & !self.spec & self.norm_mask()
    }
    const fn is_z(self) -> u64 {
        !self.data & self.spec & self.norm_mask()
    }
    const fn is_x(self) -> u64 {
        self.data & self.spec & self.norm_mask()
    }
    pub(crate) fn all_low(self) -> bool {
        self.is_0() == self.norm_mask()
    }
    pub(crate) fn all_high(self) -> bool {
        self.is_1() == self.norm_mask()
    }

    fn get_raw(self, i: u8) -> BitState {
        let data = (self.data >> i) & 1 != 0;
        let spec = (self.spec >> i) & 1 != 0;
        BitState::join(data, spec)
    }
    pub fn get(self, i: u8) -> Option<BitState> {
        (i < self.len()).then(|| self.get_raw(i))
    }

    fn set_raw(&mut self, i: u8, st: BitState) {
        let (data, spec) = st.split();
        self.data &= !(1 << i);
        self.data |= u64::from(data) << i;
        self.spec &= !(1 << i);
        self.spec |= u64::from(spec) << i;
    }
    fn set(&mut self, i: u8, st: BitState) {
        if i < self.len() {
            self.set_raw(i, st);
        }
    }
    pub fn with(mut self, i: u8, st: BitState) -> Self {
        self.set(i, st);
        self
    }

    pub fn index(self, i: u8) -> BitState {
        self.get(i).expect("index to be in bounds")
    }
    pub fn join(self, rhs: BitArray) -> BitArray {
        // TODO: assert size
        // __ | 00 | 01 | 10 | 11
        // 00 | 11 | 11 | 00 | 11
        // 01 | 11 | 11 | 01 | 11
        // 10 | 00 | 01 | 10 | 11
        // 11 | 11 | 11 | 11 | 11
        let len = self.len();
        let lz = self.is_z();
        let rz = rhs.is_z();

        let data = (!lz & !rz) | (lz & !rz & rhs.data) | (rz & self.data);
        let spec = (!lz & !rz) | (lz & !rz & rhs.spec) | (rz & self.spec);
        
        Self { data, spec, len }
    }
    pub(crate) fn short_circuits(values: impl IntoIterator<Item=BitArray>) -> bool {
        let mut occupied = 0;
        for val in values {
            let not_z = val.is_0() | val.is_1() | val.is_x();

            // Short circuit if multiple bits have non-Z
            if occupied & not_z != 0 {
                return true;
            }
            occupied |= not_z;
        }
        false
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
        Self { data, spec: 0, len: 64 }
    }
}
impl TryFrom<BitArray> for u64 {
    type Error = NotTwoValuedErr;

    fn try_from(value: BitArray) -> Result<Self, Self::Error> {
        let (data, spec) = value.normalize();
        match spec == 0 {
            true => Ok(data),
            false => {
                let not_unk = value.is_x() == 0;
                let err_st = match not_unk {
                    true => BitState::Imped,
                    false => BitState::Unk,
                };

                Err(NotTwoValuedErr(err_st))
            }
        }
    }
}

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
impl IntoIterator for BitArray {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = BitArrayIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        BitArrayIntoIter(self)
    }
}
impl ExactSizeIterator for BitArrayIntoIter {
    fn len(&self) -> usize {
        usize::from(self.0.len())
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
        for bit in self.into_iter().rev() {
            write!(f, "{bit}")?;
        }
        Ok(())
    }
}

// assume same size
impl std::ops::BitAnd for BitArray {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        // __ | 00 | 01 | 10 | 11
        // 00 | 00 | 00 | 00 | 00
        // 01 | 00 | 01 | 11 | 11
        // 10 | 00 | 11 | 11 | 11
        // 11 | 00 | 11 | 11 | 11

        let any_false = self.is_0() | rhs.is_0();
        let all_true = self.is_1() & rhs.is_1();

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

        let any_true = self.is_1() | rhs.is_1();
        let all_false = self.is_0() & rhs.is_0();

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
mod test {
    use super::{BitArray, BitState};

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

        assert_eq!(format!("{ba}"), "ZX01X1Z0");
    }
}