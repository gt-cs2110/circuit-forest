use crate::circuit::CircuitGraphMap;
use crate::func::{Component, PortProperties, PortType, PortUpdate, RunContext, port_list};
use crate::{bitarray::BitArray, bitarray::BitState};

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
        let a = u64::try_from(ctx.new_ports[0]);
        let b = u64::try_from(ctx.new_ports[1]);
        let cin = u64::try_from(ctx.new_ports[2]);

        let a_val = match a {
            Ok(val) => val,
            Err(e) =>  {
                let carry_bit = BitArray::repeat(e.bit_state(), 1);
                let out_bits = BitArray::repeat(e.bit_state(), self.bitsize);
                return vec![
                    PortUpdate { index: 3, value: carry_bit },
                    PortUpdate { index: 4, value: out_bits }
                ]
            }
        };

        let b_val = match b {
            Ok(val) => val,
            Err(e) =>  {
                let carry_bit = BitArray::repeat(e.bit_state(), 1);
                let out_bits = BitArray::repeat(e.bit_state(), self.bitsize);
                return vec![
                    PortUpdate { index: 3, value: carry_bit },
                    PortUpdate { index: 4, value: out_bits }
                ]
            }
        };    

        let cin_val = match cin {
            Ok(val) => val,
            Err(e) =>  {
                let carry_bit = BitArray::repeat(e.bit_state(), 1);
                let out_bits = BitArray::repeat(e.bit_state(), self.bitsize);
                return vec![
                    PortUpdate { index: 3, value: carry_bit },
                    PortUpdate { index: 4, value: out_bits }
                ]
            }
        };  


        let bit_mask = (1u128 << self.bitsize) - 1;
        let sum_val = (a_val as u128) + (b_val as u128) + (cin_val as u128);
        let sum_bits = BitArray::from_bits((sum_val & bit_mask) as u64, self.bitsize);
        let cout_bits = BitArray::from_bits(((sum_val >> self.bitsize) & 1) as u64, 1);


        vec![
            PortUpdate { index: 3, value: cout_bits },
            PortUpdate { index: 4, value: sum_bits }
        ]
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
            // Carry In Bit
            (PortProperties { ty: PortType::Input, bitsize: 1}, 1),
            // Carry Out Bit
            (PortProperties { ty: PortType::Output, bitsize: 1}, 1),
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize }, 1),
        ])
    }

    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        let a = u64::try_from(ctx.new_ports[0]);
        let b = u64::try_from(ctx.new_ports[1]);
        let cin = u64::try_from(ctx.new_ports[2]);

        let a_val = match a {
            Ok(val) => val,
            Err(e) =>  {
                let carry_bit = BitArray::repeat(e.bit_state(), 1);
                let out_bits = BitArray::repeat(e.bit_state(), self.bitsize);
                return vec![
                    PortUpdate { index: 3, value: carry_bit },
                    PortUpdate { index: 4, value: out_bits }
                ]
            }
        };

        let b_val = match b {
            Ok(val) => val,
            Err(e) =>  {
                let carry_bit = BitArray::repeat(e.bit_state(), 1);
                let out_bits = BitArray::repeat(e.bit_state(), self.bitsize);
                return vec![
                    PortUpdate { index: 3, value: carry_bit },
                    PortUpdate { index: 4, value: out_bits }
                ]
            }
        };    

        let cin_val = match cin {
            Ok(val) => val,
            Err(e) =>  {
                let carry_bit = BitArray::repeat(e.bit_state(), 1);
                let out_bits = BitArray::repeat(e.bit_state(), self.bitsize);
                return vec![
                    PortUpdate { index: 3, value: carry_bit },
                    PortUpdate { index: 4, value: out_bits }
                ]
            }
        };  

        let bit_mask = (1i128 << self.bitsize) - 1;
        let sum_val = (a_val as i128) - (b_val as i128) - (cin_val as i128);
        let sum_bits = BitArray::from_bits((sum_val & bit_mask) as u64, self.bitsize);

        let cout_bits = if sum_val < 0 {
            BitArray::from_bits(1, 1)
        } else {
            BitArray::from_bits(0, 1)
        };


        vec![
            PortUpdate { index: 3, value: cout_bits },
            PortUpdate { index: 4, value: sum_bits }
        ]
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
        let a = u64::try_from(ctx.new_ports[0]);

        let a_val = match a {
            Ok(val) => val,
            Err(e) =>  {
                let error_bits = BitArray::repeat(e.bit_state(), self.bitsize);
                return vec![
                    PortUpdate { index: 1, value: error_bits },
                ]
            }
        }; 

        let mask = (1u128 << self.bitsize) - 1;
        let val = (!(a_val as u128)) & mask;
        let neg_val = (val + 1) & mask;
        let out = BitArray::from_bits(neg_val as u64, self.bitsize);

        vec![
            PortUpdate { index: 1, value: out },
        ]
    }
}

/// A Comparator component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Comparator {
    bitsize: u8
}
impl Comparator {
    /// Creates a new instance of the comparator with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self {
            bitsize: bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE),
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
        let a = u64::try_from(ctx.new_ports[0]);
        let b = u64::try_from(ctx.new_ports[1]);

        let a_val = match a {
            Ok(val) => val,
            Err(e) =>  {
                let error_bits = BitArray::repeat(e.bit_state(), 1);
                return vec![
                    PortUpdate { index: 2, value: error_bits },
                    PortUpdate { index: 3, value: error_bits },
                    PortUpdate { index: 4, value: error_bits },
                ]
            }
        }; 

        let b_val = match b {
            Ok(val) => val,
            Err(e) =>  {
                let error_bits = BitArray::repeat(e.bit_state(), 1);
                return vec![
                    PortUpdate { index: 2, value: error_bits },
                    PortUpdate { index: 3, value: error_bits },
                    PortUpdate { index: 4, value: error_bits },
                ]
            }
        }; 

        let lt = BitArray::from_bits((a_val < b_val) as u64, 1);
        let eq = BitArray::from_bits((a_val == b_val) as u64, 1);
        let gt = BitArray::from_bits((a_val > b_val) as u64, 1);

        vec![
            PortUpdate { index: 2, value: lt },
            PortUpdate { index: 3, value: eq },
            PortUpdate { index: 4, value: gt },
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
        let a = u64::try_from(ctx.new_ports[0]);

        let a_val = match a {
            Ok(val) => val,
            Err(e) =>  {
                let error_bits = BitArray::repeat(e.bit_state(), self.out_bitsize);
                return vec![
                    PortUpdate { index: 1, value: error_bits },
                ]
            }
        }; 
        
        if self.in_bitsize < self.out_bitsize {
            let ext_bits = match self.ext_type {
                ExtensionType::Zero => 0u128,
                ExtensionType::One => u128::MAX,
                ExtensionType::Sign => {
                    let msb = (a_val >> (self.in_bitsize - 1)) & 1;
                    if msb == 1 {
                        u128::MAX
                    } else {
                        0u128
                    }
                }
            };
            let clear_mask = (!0u128) << (self.in_bitsize);
            let bit_ext = (ext_bits & clear_mask) | (a_val as u128);
            vec![
                PortUpdate { index: 1, value: BitArray::from_bits(bit_ext as u64, self.out_bitsize) }
            ]
        } else {
            vec![
                PortUpdate { index: 1, value: BitArray::from_bits(a_val, self.out_bitsize) }
            ]
        }
    }
}

/// Shift type for bit shifter
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

/// A Shifter component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Shifter {
    bitsize: u8,
    shift: u8,
    shift_type: ShiftType
}
impl Shifter {
    /// Creates a new instance of the Shifter with specified bitsize.
    pub fn new(bitsize: u8, shift_type: ShiftType) -> Self {
        let bit = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        Self {
            bitsize: bit,
            shift: (bit as f64).log2().ceil() as u8,
            shift_type
        }
    }
}
impl Component for Shifter {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // Input
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 1),
            (PortProperties { ty: PortType::Input, bitsize: self.shift }, 1),
            // Output
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize }, 1),
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
                ]
            }
        }; 

        let b_val = match b {
            Ok(val) => val,
            Err(e) =>  {
                let error_bits = BitArray::repeat(e.bit_state(), self.bitsize);
                return vec![
                    PortUpdate { index: 2, value: error_bits },
                ]
            }
        }; 
        let mask = if self.bitsize == 64 {
            u64::MAX
        } else {
            (1u64 << self.bitsize) - 1
        };
        let out_val = match self.shift_type {
            ShiftType::LogicalLeft => (a_val << b_val) & mask,
            ShiftType::LogicalRight => a_val >> b_val,
            ShiftType::ArithmeticRight => {
                let signed_val = if (a_val >> (self.bitsize - 1)) != 0 {
                    (a_val | (!0u64 << self.bitsize)) as i64
                } else {
                    a_val as i64
                };

                (signed_val >> b_val) as u64 & mask
            },
            ShiftType::RotateLeft => {
                ((a_val << b_val) | (a_val >> (self.bitsize - (b_val as u8)))) & mask
            },
            ShiftType::RotateRight => {
                ((a_val >> b_val) | (a_val << (self.bitsize - (b_val as u8)))) & mask
            },
        };

        vec![
            PortUpdate { index: 2, value: BitArray::from_bits(out_val, self.bitsize) }
        ]

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
    fn test_comparator_exhaustive() {
        let cmp = Comparator::new(4); // 4-bit comparator

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

        // Expected output: 6-bit zero-extended
        let expected_bits = bitarr![0,1,0,1,1,1];

        assert_eq!(
            updates,
            vec![PortUpdate { index: 1, value: expected_bits }],
            "BitExtender failed for input 0b1010 with Zero extension"
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