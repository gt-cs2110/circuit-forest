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

/// An subtractor component.
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


/// An multiplier component.
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
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 5),
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

/// An Divider component.
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
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize }, 4),
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
                    PortUpdate { index: 3, value: error_bits },
                    PortUpdate { index: 4, value: error_bits }
                ]
            }
        };

        let b_val = match b {
            Ok(val) => {if val == 0 {
                    let error_bits = BitArray::repeat(BitState::Unk, self.bitsize);
                    return vec![
                        PortUpdate { index: 3, value: error_bits },
                        PortUpdate { index: 4, value: error_bits }
                    ]
                }
                val
            }
            Err(e) =>  {
                let error_bits = BitArray::repeat(e.bit_state(), self.bitsize);
                return vec![
                    PortUpdate { index: 3, value: error_bits },
                    PortUpdate { index: 4, value: error_bits }
                ]
            }
        };    

        let remainder = BitArray::from_bits(a_val % b_val, self.bitsize);
        let quotient = BitArray::from_bits(a_val / b_val, self.bitsize);


        vec![
            PortUpdate { index: 3, value: quotient },
            PortUpdate { index: 4, value: remainder }
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
                        PortUpdate { index: 3, value: quotient },
                        PortUpdate { index: 4, value: remainder }
                    ],
                    "Divider failed for A={a}, B={b}"
                );
            }
        }     
            
    }

}