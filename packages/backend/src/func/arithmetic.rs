use std::cmp::Ordering;

use crate::bitarr;
use crate::bitarray::{NotTwoValuedErr, ShiftType};
use crate::circuit::CircuitGraphMap;
use crate::func::{Component, PortProperties, PortType, PortUpdate, RunContext, port_list};
use crate::{bitarray::BitArray, bitarray::BitState};

/// Parses a set of inputs, returning the results of each port.
/// 
/// This returns an error if any inputs are unknown.
/// If any inputs are tristate, the corresponding input is None.
/// Otherwise, this returns the bitvalue for each input.
fn parse_args<const N: usize>(ports: &[BitArray]) -> Result<[Option<u64>; N], NotTwoValuedErr> {
    // heh
    fn trystate(a: BitArray) -> Result<Option<u64>, NotTwoValuedErr> {
        match u64::try_from(a) {
            Ok(v) => Ok(Some(v)),
            Err(e) if e.is_imped() => Ok(None),
            Err(e) => Err(e)
        }
    }

    let mut out = [None; N];
    for (out, &inp) in std::iter::zip(&mut out, ports) {
        *out = trystate(inp)?;
    }
    Ok(out)
}

/// An adder component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Adder {
    bitsize: u8
}
impl Adder {
    /// Creates a new instance of the Adder with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self {
            bitsize: bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE),
        }
    }
}
impl Component for Adder {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // inputs A and B for Adder
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 2),
            // Carry In Bit
            (PortProperties { ty: PortType::Input, bitsize: 1}, 1),
            // Carry Out Bit
            (PortProperties { ty: PortType::Output, bitsize: 1}, 1),
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize }, 1),
        ])
    }

    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        // inputs: A[n], B[n], Cin[1]
        // - Cin represents the carry in from the right
        // outputs: Cout[1], S[n]
        // - Cout represents the carry out to the left
        //
        // If any of A, B, cin are X, then all outputs are X
        // If any of A, B are Z, then it is all Z
        // If cin is Z, treat as 0
        let (a, b, cin) = match parse_args(ctx.new_ports) {
            Ok([Some(a), Some(b), cin]) => (a, b, cin),
            Ok(_) => return vec![
                PortUpdate { index: 3, value: bitarr![Z] },
                PortUpdate { index: 4, value: bitarr![Z; self.bitsize] },
            ],
            Err(_) => return vec![
                PortUpdate { index: 3, value: bitarr![X] },
                PortUpdate { index: 4, value: bitarr![X; self.bitsize] },
            ]
        };
        let cin = cin.unwrap_or(0);

        let (sum, cout) = a.carrying_add(b, cin & 1 != 0);
        match self.bitsize {
            64.. => vec![
                PortUpdate { index: 3, value: BitArray::from(cout) },
                PortUpdate { index: 4, value: BitArray::from(sum) }
            ],
            _ => {
                let cout = sum & (1 << self.bitsize) != 0;
                vec![
                    PortUpdate { index: 3, value: BitArray::from(cout) },
                    PortUpdate { index: 4, value: BitArray::from_bits(sum, self.bitsize) }
                ]
            }
        }
    }
}

/// A subtractor component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Subtractor {
    bitsize: u8
}
impl Subtractor {
    /// Creates a new instance of the Subtractor with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self {
            bitsize: bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE),
        }
    }
}
impl Component for Subtractor {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // inputs A and B for Subtractor
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 2),
            // Borrow In Bit
            (PortProperties { ty: PortType::Input, bitsize: 1}, 1),
            // Borrow Out Bit
            (PortProperties { ty: PortType::Output, bitsize: 1}, 1),
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize }, 1),
        ])
    }

    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        // inputs: A[n], B[n], Bin[1]
        // - Bin represents the borrow out from the right
        // outputs: Bout[1], S[n]
        // - Bout represents the borrow out to the left
        //
        // If any of A, B, cin are X, then all outputs are X
        // If any of A, B are Z, then it is all Z
        // If cin is Z, treat as 0
        let (a, b, bin) = match parse_args(ctx.new_ports) {
            Ok([Some(a), Some(b), bin]) => (a, b, bin),
            Ok(_) => return vec![
                PortUpdate { index: 3, value: bitarr![Z] },
                PortUpdate { index: 4, value: bitarr![Z; self.bitsize] },
            ],
            Err(_) => return vec![
                PortUpdate { index: 3, value: bitarr![X] },
                PortUpdate { index: 4, value: bitarr![X; self.bitsize] },
            ]
        };
        let bin = bin.unwrap_or(0);

        let (diff, bout) = a.borrowing_sub(b, bin & 1 != 0);
        match self.bitsize {
            64.. => vec![
                PortUpdate { index: 3, value: BitArray::from(bout) },
                PortUpdate { index: 4, value: BitArray::from(diff) }
            ],
            _ => {
                let bout = diff & (1 << self.bitsize) != 0;
                vec![
                    PortUpdate { index: 3, value: BitArray::from(bout) },
                    PortUpdate { index: 4, value: BitArray::from_bits(diff, self.bitsize) }
                ]
            }
        }
    }
}


/// A multiplier component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Multiplier {
    bitsize: u8
}
impl Multiplier {
    /// Creates a new instance of the Multiplier with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self {
            bitsize: bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE),
        }
    }
}
impl Component for Multiplier {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            /*
                [0] = A,
                [1] = B,
                [2] = Carry In
                [3] = Out
                [4] = Upper Bits
             */
            // Inputs
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 3),
            // Outputs
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize }, 2),
        ])
    }

    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        let a = u64::try_from(ctx.new_ports[0]);
        let b = u64::try_from(ctx.new_ports[1]);
        let cin = u64::try_from(ctx.new_ports[2]);

        let a_val = match a {
            Ok(val) => val,
            Err(e) =>  {
                let error_bits = BitArray::repeat(e.bit_state(), self.bitsize);
                return vec![
                    PortUpdate { index: 3, value: error_bits },
                    PortUpdate { index: 4, value: error_bits }
                ]
            }
        };

        let b_val = match b {
            Ok(val) => val,
            Err(e) =>  {
                let error_bits = BitArray::repeat(e.bit_state(), self.bitsize);
                return vec![
                    PortUpdate { index: 3, value: error_bits },
                    PortUpdate { index: 4, value: error_bits }
                ]
            }
        };    

        let cin_val = match cin {
            Ok(val) => val,
            Err(e) =>  {
                let error_bits = BitArray::repeat(e.bit_state(), self.bitsize);
                return vec![
                    PortUpdate { index: 3, value: error_bits },
                    PortUpdate { index: 4, value: error_bits }
                ]
            }
        };  

        let bit_mask = (1u128 << self.bitsize) - 1;
        let mult_val = (a_val as u128) * (b_val as u128) + (cin_val as u128);
        let lower_bits = BitArray::from_bits((mult_val & bit_mask) as u64, self.bitsize);
        let upper_bits = BitArray::from_bits(((mult_val >> self.bitsize) & bit_mask) as u64, self.bitsize);


        vec![
            PortUpdate { index: 3, value: lower_bits },
            PortUpdate { index: 4, value: upper_bits }
        ]
    }
}

/// A Divider component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Divider {
    bitsize: u8
}
impl Divider {
    /// Creates a new instance of the Divider with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self {
            bitsize: bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE),
        }
    }
}
impl Component for Divider {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            /*
                [0] = A,
                [1] = B,
                [2] = Quotient
                [3] = Remainder
             */
            // Inputs
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 2),
            // Outputs
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 2),
        ])
    }

    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        let a = u64::try_from(ctx.new_ports[0]);
        let b = u64::try_from(ctx.new_ports[1]);

        let a_val = match a {
            Ok(val) => val,
            Err(e) =>  {
                let error_bits = BitArray::repeat(e.bit_state(), self.bitsize);
                return vec![
                    PortUpdate { index: 2, value: error_bits },
                    PortUpdate { index: 3, value: error_bits }
                ]
            }
        };

        let b_val = match b {
            Ok(val) => {if val == 0 {
                    let error_bits = BitArray::repeat(BitState::Unk, self.bitsize);
                    return vec![
                        PortUpdate { index: 2, value: error_bits },
                        PortUpdate { index: 3, value: error_bits }
                    ]
                }
                val
            }
            Err(e) =>  {
                let error_bits = BitArray::repeat(e.bit_state(), self.bitsize);
                return vec![
                    PortUpdate { index: 2, value: error_bits },
                    PortUpdate { index: 3, value: error_bits }
                ]
            }
        };    

        let remainder = BitArray::from_bits(a_val % b_val, self.bitsize);
        let quotient = BitArray::from_bits(a_val / b_val, self.bitsize);


        vec![
            PortUpdate { index: 2, value: quotient },
            PortUpdate { index: 3, value: remainder }
        ]
    }
}

/// A negator component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Negator {
    bitsize: u8
}
impl Negator {
    /// Creates a new instance of the negator with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self {
            bitsize: bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE),
        }
    }
}
impl Component for Negator {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // Input
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 1),
            // Output
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize }, 1),
        ])
    }

    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        let inp = match u64::try_from(ctx.new_ports[0]) {
            Ok(val) => val,
            Err(e) => return vec![PortUpdate {
                index: 1,
                value: BitArray::repeat(e.bit_state(), self.bitsize)
            }]
        };

        let out = BitArray::from_bits(inp.wrapping_neg(), self.bitsize);
        vec![PortUpdate { index: 1, value: out }]
    }
}

/// Signedness for integers, used for certain operations that differ between signedness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignType {
    /// Two's complement.
    TwosComplement,
    /// Unsigned.
    Unsigned
}

/// A Comparator component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Comparator {
    bitsize: u8,
    signedness: SignType

}
impl Comparator {
    /// Creates a new instance of the comparator with specified bitsize.
    pub fn new(bitsize: u8, signedness: SignType) -> Self {
        Self {
            bitsize: bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE),
            signedness
        }
    }
}
impl Component for Comparator {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // Input
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 2),
            /*
                [2] = <
                [3] = =
                [4] = >
             */ 
            (PortProperties { ty: PortType::Output, bitsize: 1 }, 3),
        ])
    }

    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        let [a, b] = match parse_args(ctx.new_ports) {
            Ok([Some(a), Some(b)]) => [a, b],
            Ok(_) => return vec![
                PortUpdate { index: 2, value: bitarr![Z] },
                PortUpdate { index: 3, value: bitarr![Z] },
                PortUpdate { index: 4, value: bitarr![Z] },
            ],
            Err(_) => return vec![
                PortUpdate { index: 2, value: bitarr![X] },
                PortUpdate { index: 3, value: bitarr![X] },
                PortUpdate { index: 4, value: bitarr![X] },
            ]
        };

        let cmp = match self.signedness {
            SignType::TwosComplement => (a as i64).cmp(&(b as i64)),
            SignType::Unsigned => a.cmp(&b),
        };
        vec![
            PortUpdate { index: 2, value: BitArray::from(cmp == Ordering::Less) },
            PortUpdate { index: 3, value: BitArray::from(cmp == Ordering::Equal) },
            PortUpdate { index: 4, value: BitArray::from(cmp == Ordering::Greater) },
        ]
    }
}

/// Extension type for bit extender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtensionType {
    /// pad with 0
    Zero,
    /// pad with 1
    One,
    /// pad with MSB of input
    Sign,  
}

/// A Bit-Extender component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct BitExtender {
    in_bitsize: u8,
    out_bitsize: u8,
    ext_type: ExtensionType
}
impl BitExtender {
    /// Creates a new instance of the bitextender with specified bitsize.
    pub fn new(in_bitsize: u8, out_bitsize:u8, ext_type: ExtensionType) -> Self {
        Self {
            in_bitsize: in_bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE),
            out_bitsize: out_bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE),
            ext_type
        }
    }
}
impl Component for BitExtender {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // Input
            (PortProperties { ty: PortType::Input, bitsize: self.in_bitsize}, 1),
            // Output
            (PortProperties { ty: PortType::Output, bitsize: self.out_bitsize }, 1),
        ])
    }

    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        let fill = match self.ext_type {
            ExtensionType::Zero => BitState::Low,
            ExtensionType::One => BitState::High,
            ExtensionType::Sign => ctx.new_ports[0].get(ctx.new_ports[0].len() - 1).unwrap(),
        };
        let value = ctx.new_ports[0].resize(self.out_bitsize, fill);
        dbg!(fill, value);
        vec![PortUpdate { index: 1, value }]
    }
}

/// A Shifter component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Shifter {
    bitsize: u8,
    shift_type: ShiftType
}
impl Shifter {
    /// Creates a new instance of the Shifter with specified bitsize.
    pub fn new(bitsize: u8, shift_type: ShiftType) -> Self {
        let bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        Self {
            bitsize,
            shift_type
        }
    }

    /// Gets the bitsize of the shift parameter.
    pub fn shift_bitsize(self) -> u8 {
        // ilog2 ceil
        // doesn't exist in stdlib so we're just gonna hardcode it lol
        match self.bitsize {
            0..=2   => 1,
            3..=4   => 2,
            5..=8   => 3,
            9..=16  => 4,
            17..=32 => 5,
            _ => 6,
        }
    }
}
impl Component for Shifter {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // Input
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 1),
            // Shift
            (PortProperties { ty: PortType::Input, bitsize: self.shift_bitsize() }, 1),
            // Output
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize }, 1),
        ])
    }

    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        let a = ctx.new_ports[0];
        let Ok(b) = u64::try_from(ctx.new_ports[1]) else {
            return vec![PortUpdate { index: 2, value: bitarr![X; self.bitsize] }]
        };
        
        vec![PortUpdate { index: 2, value: a.shift(b as u32, self.shift_type) }]
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bitarr};
    #[test]
    fn test_adder_exhaustive() {
        let adder = Adder::new(4); // 4-bit adder

        // Iterate over all 4-bit values for A and B
        for a in 0..16 {
            for b in 0..16 {
                for cin in 0..2 {
                    // Convert to little-endian
                    let in_a = BitArray::from_bits(a, 4);
                    let in_b = BitArray::from_bits(b, 4);
                    let cin_ba = BitArray::from_bits(cin, 1);

                    let old_ports = &[
                        bitarr![Z; 4], // A
                        bitarr![Z; 4], // B
                        bitarr![Z],    // Cin
                        bitarr![Z],    // Cout
                        bitarr![Z; 4], // Sum
                    ];

                    let updates = adder.run(RunContext {
                        graphs: &Default::default(),
                        old_ports,
                        new_ports: &[
                            in_a,
                            in_b,
                            cin_ba,
                            bitarr![Z],    // Cout placeholder
                            bitarr![Z; 4], // Sum placeholder
                        ],
                        inner_state: None,
                    });

                    // Compute expected sum and carry-out
                    let total = a + b + cin;
                    let expected_sum = BitArray::from_bits(total & 0b1111, 4);
                    let expected_cout = BitArray::from_bits((total >> 4) & 0b1, 1);

                    assert_eq!(
                        updates,
                        vec![
                            PortUpdate { index: 3, value: expected_cout },
                            PortUpdate { index: 4, value: expected_sum }
                        ],
                        "Adder failed for A={a}, B={b}, Cin={cin}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_subtractor_exhaustive() {
        let subtractor = Subtractor::new(4); // 4-bit subtractor

        // Iterate over all 4-bit values for A and B
        for a in 0..16 {
            for b in 0..16 {
                for cin in 0..2 {
                    // Convert to little-endian
                    let in_a = BitArray::from_bits(a, 4);
                    let in_b = BitArray::from_bits(b, 4);
                    let cin_ba = BitArray::from_bits(cin, 1);

                    let old_ports = &[
                        bitarr![Z; 4], // A
                        bitarr![Z; 4], // B
                        bitarr![Z],    // Cin
                        bitarr![Z],    // Cout
                        bitarr![Z; 4], // Sum
                    ];

                    let updates = subtractor.run(RunContext {
                        graphs: &Default::default(),
                        old_ports,
                        new_ports: &[
                            in_a,
                            in_b,
                            cin_ba,
                            bitarr![Z],    // Cout placeholder
                            bitarr![Z; 4], // Sum placeholder
                        ],
                        inner_state: None,
                    });

                    // Compute expected sum and carry-out
                    let total = (a as i64) - (b as i64) - (cin as i64);
                    let expected_sum = BitArray::from_bits((total as u64) & 0b1111, 4);
                    let expected_cout = if total < 0 {
                        BitArray::from_bits(1, 1)
                    } else {
                        BitArray::from_bits(0, 1)
                    };

                    assert_eq!(
                        updates,
                        vec![
                            PortUpdate { index: 3, value: expected_cout },
                            PortUpdate { index: 4, value: expected_sum }
                        ],
                        "Subtractor failed for A={a}, B={b}, Cin={cin}"
                    );
                }
            }
        }     
            
    }

    #[test]
    fn test_multiplier_exhaustive() {
        let multiplier = Multiplier::new(4); // 4-bit multiplier

        // Iterate over all 4-bit values for A and B
        for a in 0..16 {
            for b in 0..16 {
                for cin in 0..16 {
                    // Convert to little-endian
                    let in_a = BitArray::from_bits(a, 4);
                    let in_b = BitArray::from_bits(b, 4);
                    let cin_ba = BitArray::from_bits(cin, 4);

                    let old_ports = &[bitarr![Z; 4]; 5];

                    let updates = multiplier.run(RunContext {
                        graphs: &Default::default(),
                        old_ports,
                        new_ports: &[
                            in_a,
                            in_b,
                            cin_ba,
                            bitarr![Z; 4], // Out placeholder
                            bitarr![Z; 4], // Upper Bits placeholder
                        ],
                        inner_state: None,
                    });

                    // Compute expected upper and lower bits
                    let total = a * b + cin;
                    let lower_bits = BitArray::from_bits(total & 0b1111, 4);
                    let upper_bits = BitArray::from_bits((total >> 4) & 0b1111, 4);

                    assert_eq!(
                        updates,
                        vec![
                            PortUpdate { index: 3, value: lower_bits },
                            PortUpdate { index: 4, value: upper_bits }
                        ],
                        "Multiplier failed for A={a}, B={b}, Cin={cin}"
                    );
                }
            }
        }     
            
    }

    #[test]
    fn test_divider_exhaustive() {
        let divider = Divider::new(4); // 4-bit Divider

        // Iterate over all 4-bit values for A and B
        for a in 0..16 {
            for b in 0..16 {
                // Convert to little-endian
                let in_a = BitArray::from_bits(a, 4);
                let in_b = BitArray::from_bits(b, 4);

                let old_ports = &[bitarr![Z; 4]; 4];

                let updates = divider.run(RunContext {
                    graphs: &Default::default(),
                    old_ports,
                    new_ports: &[
                        in_a,
                        in_b,
                        bitarr![Z; 4], // Quotient placeholder
                        bitarr![Z; 4], // Remainder placeholder
                    ],
                    inner_state: None,
                });

                // Compute expected quotient and remainder
                let quotient = if b == 0 {
                    BitArray::repeat(BitState::Unk, 4)
                } else {
                    BitArray::from_bits((a / b) & 0b1111, 4)
                };


                let remainder = if b == 0 {
                    BitArray::repeat(BitState::Unk, 4)
                } else {
                    BitArray::from_bits( (a % b) & 0b1111, 4)
                };

                assert_eq!(
                    updates,
                    vec![
                        PortUpdate { index: 2, value: quotient },
                        PortUpdate { index: 3, value: remainder }
                    ],
                    "Divider failed for A={a}, B={b}"
                );
            }
        }     
            
    }

    #[test]
    fn test_negator_exhaustive() {
        let negator = Negator::new(4); // 4-bit Negator

        // Iterate over all 4-bit values for input
        for a in 0..16 {
            // Convert to little-endian
            let in_a = BitArray::from_bits(a, 4);

            let old_ports = &[bitarr![Z; 4]; 2];

            let updates = negator.run(RunContext {
                graphs: &Default::default(),
                old_ports,
                new_ports: &[
                    in_a,
                    bitarr![Z; 4], // Output placeholder
                ],
                inner_state: None,
            });

            // Compute expected output



            let out = BitArray::from_bits((-(a as i64) & 0b1111) as u64, 4);

            assert_eq!(
                updates,
                vec![
                    PortUpdate { index: 1, value: out },
                ],
                "Negator failed for A={a}"
            );
        }     
            
    }

    #[test]
    fn test_comparator_unsigned_exhaustive() {
        let cmp = Comparator::new(4, SignType::Unsigned); // 4-bit comparator

        // Iterate over all 4-bit pairs A and B
        for a in 0..16 {
            for b in 0..16 {
                let in_a = BitArray::from_bits(a, 4);
                let in_b = BitArray::from_bits(b, 4);

                // Dummy old_ports
                let old_ports = &[
                    bitarr![Z; 4], // A
                    bitarr![Z; 4], // B
                    bitarr![Z],    // LT
                    bitarr![Z],    // EQ
                    bitarr![Z],    // GT
                ];

                // Run the comparator
                let updates = cmp.run(RunContext {
                    graphs: &Default::default(),
                    old_ports,
                    new_ports: &[
                        in_a,        // A
                        in_b,        // B
                        bitarr![Z],  // LT placeholder
                        bitarr![Z],  // EQ placeholder
                        bitarr![Z],  // GT placeholder
                    ],
                    inner_state: None,
                });

                // Expected results
                let lt = BitArray::from_bits((a < b) as u64, 1);
                let eq = BitArray::from_bits((a == b) as u64, 1);
                let gt = BitArray::from_bits((a > b) as u64, 1);

                assert_eq!(
                    updates,
                    vec![
                        PortUpdate { index: 2, value: lt },
                        PortUpdate { index: 3, value: eq },
                        PortUpdate { index: 4, value: gt },
                    ],
                    "Comparator failed for A={a}, B={b}"
                );
            }
        }
    }

    #[test]
    fn test_bit_extender_simple() {
        // 4-bit input, 6-bit output, zero extension
        let extender = BitExtender::new(4, 6, ExtensionType::Sign);

        let input_val = 0b1010; // A = 10
        let in_a = BitArray::from_bits(input_val, 4);

        let old_ports = &[
            bitarr![Z; 4], // A
            bitarr![Z; 6], // B
        ];

        let updates = extender.run(RunContext {
            graphs: &Default::default(),
            old_ports,
            new_ports: &[in_a, bitarr![Z; 6]],
            inner_state: None,
        });

        // Expected output: 6-bit sign-extended
        let expected_bits = bitarr![0,1,0,1,1,1];

        assert_eq!(
            updates,
            vec![PortUpdate { index: 1, value: expected_bits }],
            "BitExtender failed for input 0b1010 with Sign extension"
        );
    }

    #[test]
    fn test_shifter_simple() {
        let a_val = 0b1100u64; // input value (4 bits)
        let shift_val = 1u64;  // shift amount

        let in_a = BitArray::from_bits(a_val, 4);
        let in_b = BitArray::from_bits(shift_val, 2);

        let old_ports = &[
            bitarr![Z; 4], // A
            bitarr![Z; 2], // B
            bitarr![Z; 4], // Output 
        ];

        // Expected outputs for each shift type
        let expected_outputs = [
            (ShiftType::LogicalLeft, 0b1000u64),     // 1100 << 1 = 1000 (4-bit mask)
            (ShiftType::LogicalRight, 0b0110u64),    // 1100 >> 1 = 0110
            (ShiftType::ArithmeticRight, 0b1110u64), // 1100 as signed i64 >> 1 = 1110
            (ShiftType::RotateLeft, 0b1001u64),      // rotate left 1: 1100 -> 1001
            (ShiftType::RotateRight, 0b0110u64),     // rotate right 1: 1100 -> 0110
        ];

        for &(stype, expected_val) in &expected_outputs {
            let shifter = Shifter::new(4, stype);

            let updates = shifter.run(RunContext {
                graphs: &Default::default(),
                old_ports,
                new_ports: &[in_a, in_b, bitarr![Z; 4]],
                inner_state: None,
            });

            let expected_bits = BitArray::from_bits(expected_val, 4);

            assert_eq!(
                updates,
                vec![PortUpdate { index: 2, value: expected_bits }],
                "Shifter failed for input 0b{:04b}, shift {} with {:?}",
                a_val,
                shift_val,
                stype
            );
        }
    }

}