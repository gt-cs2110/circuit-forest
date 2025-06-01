#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BitState {
    Low = 0b00, High = 0b01, Imped = 0b10, Unk = 0b11
}
impl BitState {
    fn split(self) -> (bool /* data */, bool /* spec */) {
        ((self as u8) & 0b01 != 0, (self as u8) & 0b10 != 0)
    }
    fn join(data: bool, spec: bool) -> Self {
        match (data, spec) {
            (false, false) => BitState::Low,
            (true,  false) => BitState::High,
            (false, true)  => BitState::Imped,
            (true,  true)  => BitState::Unk,
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

pub struct NotTwoValuedErr(());
impl TryFrom<BitState> for bool {
    type Error = NotTwoValuedErr;

    fn try_from(value: BitState) -> Result<Self, Self::Error> {
        match value {
            BitState::Low   => Ok(false),
            BitState::High  => Ok(true),
            BitState::Imped => Err(NotTwoValuedErr(())),
            BitState::Unk   => Err(NotTwoValuedErr(())),
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

impl std::ops::BitAnd for BitState {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        // Identities:
        // F & a = F
        // T & a = a
        // X, o.w.
        match (self, rhs) {
            (BitState::Low, _) | (_, BitState::Low) => BitState::Low,
            (BitState::High, a) | (a, BitState::High) => a,
            _ => BitState::Unk
        }
    }
}
impl std::ops::BitOr for BitState {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        // Identities:
        // F & a = a
        // T & a = T
        // X, o.w.
        match (self, rhs) {
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

#[derive(Default, Clone)]
pub struct BitArray {
    data: u64,
    spec: u64,
    len: u8
}
impl BitArray {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn floating(len: u8) -> Self {
        Self {
            data: 0,
            spec: u64::MAX,
            len: len.min(64)
        }
    }
    pub fn len(&self) -> u8 {
        self.len.min(64)
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub(crate) fn from_u64(data: u64) -> Self {
        Self { data, spec: 0, len: 64 }
    }
    pub(crate) fn to_u64(&self) -> u64 {
        self.data
    }
    fn normalize(&self) -> (u64, u64) {
        let mask = 1u64.wrapping_shl(u32::from(self.len())).wrapping_sub(1);
        (self.data & mask, self.spec & mask)
    }

    pub fn get(&self, i: u8) -> Option<BitState> {
        (i < self.len()).then(|| {
            let data = (self.data >> i) & 1 != 0;
            let spec = (self.spec >> i) & 1 != 0;
            BitState::join(data, spec)
        })
    }
    pub fn pop(&mut self) -> Option<BitState> {
        if !self.is_empty() {
            self.len -= 1;
            let data = self.data & 1 != 0;
            let spec = self.spec & 1 != 0;
            self.data >>= 1;
            self.spec >>= 1;
            Some(BitState::join(data, spec))
        } else {
            None
        }
    }
}
impl FromIterator<BitState> for BitArray {
    fn from_iter<I: IntoIterator<Item = BitState>>(iter: I) -> Self {
        let (len, data, spec) = iter.into_iter()
            .map(BitState::split)
            .fold((0, 0, 0), |(l, data, spec), (d, s)| {
                (l + 1, data | (u64::from(d) << l), spec | (u64::from(s) << l))
            });
        
        Self { len, data, spec }
    }
}

pub struct BitArrayIntoIter(BitArray);
impl Iterator for BitArrayIntoIter {
    type Item = BitState;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}
impl IntoIterator for BitArray {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = BitArrayIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        BitArrayIntoIter(self)
    }
}

impl PartialEq for BitArray {
    fn eq(&self, other: &Self) -> bool {
        self.normalize() == other.normalize()
    }
}
impl Eq for BitArray {}
impl std::hash::Hash for BitArray {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let (data, spec) = self.normalize();
        data.hash(state);
        spec.hash(state);
        self.len.hash(state);
    }
}
impl std::fmt::Debug for BitArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.clone())
            .finish()
    }
}

// assume same size
impl std::ops::BitAnd for BitArray {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        let spec = self.spec | rhs.spec;
        let data = spec | (self.data & rhs.data);
        let len = self.len;
        Self { spec, data, len }
    }
}
impl std::ops::BitOr for BitArray {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        let spec = self.spec | rhs.spec;
        let data = spec | (self.data | rhs.data);
        let len = self.len;
        Self { spec, data, len }
    }
}
impl std::ops::BitXor for BitArray {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let spec = self.spec | rhs.spec;
        let data = spec | (self.data ^ rhs.data);
        let len = self.len;
        Self { spec, data, len }
    }
}
impl std::ops::Not for BitArray {
    type Output = Self;

    fn not(self) -> Self::Output {
        let spec = self.spec;
        let data = spec | !self.data;
        let len = self.len;
        Self { spec, data, len }
    }
}