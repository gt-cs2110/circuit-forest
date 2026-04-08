/// A u8 which is restricted to the `MIN..=MAX` range.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[cfg_attr(feature="serde", derive(serde::Serialize), serde(transparent))]
pub struct RangedByte<const MIN: u8, const MAX: u8>(u8);

impl<const MIN: u8, const MAX: u8> RangedByte<MIN, MAX> {
    /// Creates a new [`RangedByte`],
    /// or returns None if value is out of bounds.
    pub const fn new(n: u8) -> Option<Self> {
        const { assert!(MIN <= MAX); }
        if MIN <= n && n <= MAX {
            Some(Self(n))
        } else {
            None
        }
    }
    /// Creates a new [`RangedByte`],
    /// clamping to the bounds if out of bounds.
    pub fn new_clamped(n: u8) -> Self {
        const { assert!(MIN <= MAX); }
        Self(n.clamp(MIN, MAX))
    }
    /// Gets the value.
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Increments value if result would stay in bound.
    pub const fn incremented(self) -> Option<Self> {
        match self.0 < MAX {
            true => Some(Self(self.0 + 1)),
            false => None
        }
    }
    /// Decrements value if result would stay in bound.
    pub const fn decremented(self) -> Option<Self> {
        match self.0 > MIN {
            true => Some(Self(self.0 - 1)),
            false => None
        }
    }
}

impl<const MIN: u8, const MAX: u8> std::fmt::Debug for RangedByte<MIN, MAX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl<const MIN: u8, const MAX: u8> std::fmt::Display for RangedByte<MIN, MAX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<const MIN: u8, const MAX: u8> PartialEq<u8> for RangedByte<MIN, MAX> {
    fn eq(&self, other: &u8) -> bool {
        self.0 == *other
    }
}
impl<const MIN: u8, const MAX: u8> PartialOrd<u8> for RangedByte<MIN, MAX> {
    fn partial_cmp(&self, other: &u8) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(other))
    }
}
impl<const MIN: u8, const MAX: u8> PartialEq<RangedByte<MIN, MAX>> for u8 {
    fn eq(&self, other: &RangedByte<MIN, MAX>) -> bool {
        *self == other.0
    }
}
impl<const MIN: u8, const MAX: u8> PartialOrd<RangedByte<MIN, MAX>> for u8 {
    fn partial_cmp(&self, other: &RangedByte<MIN, MAX>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other.0))
    }
}
impl<const MAX: u8> Default for RangedByte<0, MAX> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<const MIN: u8, const MAX: u8> From<RangedByte<MIN, MAX>> for u8 {
    fn from(value: RangedByte<MIN, MAX>) -> Self {
        value.get()
    }
}
impl<const MIN: u8, const MAX: u8> From<RangedByte<MIN, MAX>> for usize {
    fn from(value: RangedByte<MIN, MAX>) -> Self {
        usize::from(value.get())
    }
}

#[cfg(feature="serde")]
impl<'de, const MIN: u8, const MAX: u8> serde::Deserialize<'de> for RangedByte<MIN, MAX> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        let n = u8::deserialize(deserializer)?;
        Self::new(n)
            .ok_or_else(|| serde::de::Error::invalid_value(
                serde::de::Unexpected::Unsigned(n.into()),
                &&*format!("integer in {MIN}..={MAX}")
            ))
    }
}