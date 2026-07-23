//! The splitter component.

use thiserror::Error;

use crate::bitarr;
use crate::engine::{CircuitGraphMap, Component, PortProperties};
use crate::engine::func::{BitSize, PortType, PortUpdate, RunContext, Sensitivity};

/// Copies a slice into a 64-bit array, allowing for it to be used in port assignments.
pub fn splitter_ports_slice<T: Copy>(a: &[Option<T>]) -> [Option<T>; 64] {
    let mut result = [None; 64];
    let len = result.len().min(a.len());
    result[..len].copy_from_slice(&a[..len]);
    result
}
/// Creates a port assignment array from 0..`up_to`.
/// 
/// If `up_to` >= 64, this is equivalent to a port assignment array for `0..64`.
pub fn splitter_ports_range(up_to: u8) -> [Option<u8>; 64] {
    std::array::from_fn(|i| (i < up_to.into()).then_some(i as u8))
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SplitterConfig {
    //mapping of bits to legs
    #[serde(with = "serde_arrays")]
    port_assignments: [Option<u8>; 64],
    num_legs: u8,
    bitsize: BitSize,
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SplitterConfigError {
    #[error("bit {0} is assigned but out of range (bitsize is {1})")]
    AssignmentOutOfBitsize(usize, u8),
    #[error("leg index {0} is out of range (only {1} legs configured)")]
    LegOutOfRange(u8, u8),
}

impl SplitterConfig {
    pub fn new(
        port_assignments: [Option<u8>; 64],
        num_legs: u8,
        bitsize: u8,
    ) -> Result<Self, SplitterConfigError> {
        for (bit, &leg) in port_assignments.iter().enumerate() {
            let Some(leg) = leg else { continue };
            if bit >= bitsize as usize {
                return Err(SplitterConfigError::AssignmentOutOfBitsize(bit, bitsize));
            }

            if leg >= num_legs {
                return Err(SplitterConfigError::LegOutOfRange(leg, num_legs));
            }
        }
        // catch legs with zero width
        for leg in 0..num_legs {
            if !port_assignments.contains(&Some(leg)) {
                //TODO Somehow pass up if a leg isnt used so it can auto condense; maybe let frontend do this and only auto condense visually
            }
        }
        Ok(Self {
            port_assignments,
            num_legs,
            bitsize: BitSize::new_clamped(bitsize),
        })
    }

    fn bits_for_leg(&self, leg: u8) -> impl Iterator<Item = usize> {
        self.port_assignments
            .iter()
            .enumerate()
            .filter_map(move |(bit, &f)| (f == Some(leg)).then_some(bit))
    }
    fn leg_width(&self, leg: u8) -> usize {
        self.bits_for_leg(leg).count()
    }
    pub fn get_bitsize(&self) -> BitSize {
        self.bitsize
    }
    pub fn get_num_legs(&self) -> u8 {
        self.num_legs
    }
    pub fn get_num_active_legs(&self) -> usize {
        (0..self.get_num_legs())
            .filter(|&l| self.leg_width(l) > 0)
            .count()
    }
}

/// A splitter component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Splitter {
    config: SplitterConfig,
}
impl Splitter {
    /// Creates a new instance of the Splitter with specified bitsize.
    pub fn new(config: SplitterConfig) -> Self {
        Self { config }
    }
}

impl Component for Splitter {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        let mut ports = vec![PortProperties {
            ty: PortType::Inout,
            bitsize: self.config.get_bitsize().get(),
        }];
        ports.extend(
            (0..self.config.get_num_legs())
                .into_iter()
                .filter_map(|leg| {
                    (self.config.leg_width(leg) > 0).then_some(PortProperties {
                        ty: PortType::Inout,
                        bitsize: self.config.leg_width(leg) as u8,
                    })
                }),
        );
        ports
    }

    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        //removes inactive legs and shifts from leg index to corresponding port index

        let active_legs: Vec<(u8, usize)> = (0..self.config.get_num_legs())
            .filter(|&l| self.config.leg_width(l) > 0)
            .enumerate()
            .map(|(index, leg)| (leg, index + 1))
            .collect();

        if Sensitivity::Anyedge.activated(ctx.old_ports[0], ctx.new_ports[0]) {
            //set the leg values
            let mut updates: Vec<PortUpdate> = active_legs
                .iter()
                .map(|&(leg, index)| PortUpdate {
                    index,
                    value: self
                        .config
                        .bits_for_leg(leg)
                        .map(|bit| ctx.new_ports[0].index(bit as u8))
                        .collect(),
                })
                .collect();
            //set the joined value to unknow
            updates.push(PortUpdate {
                index: 0,
                value: bitarr![Z;self.config.get_bitsize().get()],
            });
            updates
        } else if Sensitivity::Anyedge.any_activated(&ctx.old_ports[1..], &ctx.new_ports[1..]) {
            // combine each legs bits back into position
            let mut value = ctx.new_ports[0];
            for leg in 0..self.config.get_num_legs() {
                for (i, bit) in self.config.bits_for_leg(leg).enumerate() {
                    value = value.with(bit as u8, ctx.new_ports[(leg as usize) + 1].index(i as u8));
                }
            }
            //Drive the joined value
            let mut updates = vec![PortUpdate { index: 0, value }];
            //set the legs to unknow so that they dont drive backwards and cause a short circuit

            updates.extend(active_legs.iter().map(|&(leg, index)| PortUpdate {
                index,
                value: bitarr![Z; self.config.leg_width(leg) as u8],
            }));

            updates
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bitarray::{BitArray, BitState};
    use crate::engine::func::floating_ports;

    use super::*;

    fn alternating(bitsize: u8) -> BitArray {
        (0..bitsize)
            .map(|i| {
                if i % 2 == 0 {
                    BitState::High
                } else {
                    BitState::Low
                }
            })
            .collect()
    }

    fn assert_port_updates_eq(mut actual: Vec<PortUpdate>, mut expected: Vec<PortUpdate>, msg: &str) {
        actual.sort_by_key(|u| u.index);
        expected.sort_by_key(|u| u.index);

        assert_eq!(actual, expected, "{msg}");
    }

    // ---------- SplitterConfig::new validation ----------

    #[test]
    fn config_rejects_leg_out_of_range() {
        let a = splitter_ports_slice(&[Some(5)]);
        let err = SplitterConfig::new(a, 2, 1).unwrap_err();
        assert_eq!(err, SplitterConfigError::LegOutOfRange(5, 2));
    }

    #[test]
    fn config_rejects_assignment_beyond_bitsize() {
        let a = splitter_ports_slice(&[Some(0), Some(0), None, Some(1)]);
        let err = SplitterConfig::new(a, 2, 2).unwrap_err();
        assert_eq!(err, SplitterConfigError::AssignmentOutOfBitsize(3, 2));
    }

    // ---------- identity split (mirrors old Splitter::new(bitsize) behavior) ----------

    fn identity_config(bitsize: u8) -> SplitterConfig {
        SplitterConfig::new(splitter_ports_range(bitsize), bitsize, bitsize)
            .expect("identity split should always be valid")
    }

    #[test]
    fn test_splitter_split_identity() {
        for bitsize in BitArray::MIN_BITSIZE..=BitArray::MAX_BITSIZE {
            let config = identity_config(bitsize);
            let splitter = Splitter::new(config);
            let props = splitter.ports(&Default::default());

            assert_eq!(props.len(), 1 + bitsize as usize);
            assert_eq!(
                props[0],
                PortProperties {
                    ty: PortType::Inout,
                    bitsize
                }
            );
            assert_eq!(
                props[1..],
                vec![
                    PortProperties {
                        ty: PortType::Inout,
                        bitsize: 1
                    };
                    bitsize as usize
                ]
            );

            let old_ports = floating_ports(&props);
            let mut new_ports = floating_ports(&props);

            let joined = alternating(bitsize);
            assert!(new_ports[0].replace(joined).is_ok());

            let actual = splitter.run(RunContext {
                graphs: &Default::default(),
                old_ports: &old_ports,
                new_ports: &new_ports,
                inner_state: None,
            });
            let mut expected: Vec<_> = joined
                .into_iter()
                .enumerate()
                .map(|(i, st)| PortUpdate {
                    index: 1 + i,
                    value: BitArray::from(st),
                })
                .collect();
            expected.push(PortUpdate {
                index: 0,
                value: bitarr![Z; bitsize],
            });

            assert_port_updates_eq(
                actual,
                expected,
                "identity splitter should split each bit to its own leg"
            );
        }
    }

    #[test]
    fn test_splitter_join_identity() {
        for bitsize in BitArray::MIN_BITSIZE..=BitArray::MAX_BITSIZE {
            let config = identity_config(bitsize);
            let splitter = Splitter::new(config);
            let props = splitter.ports(&Default::default());

            let old_ports = floating_ports(&props);
            let mut new_ports = floating_ports(&props);

            let joined = alternating(bitsize);
            let split: Vec<_> = joined.into_iter().map(BitArray::from).collect();

            for (p, arr) in std::iter::zip(new_ports[1..].iter_mut(), split) {
                assert!(p.replace(arr).is_ok());
            }

            let actual = splitter.run(RunContext {
                graphs: &Default::default(),
                old_ports: &old_ports,
                new_ports: &new_ports,
                inner_state: None,
            });
            let mut expected = vec![PortUpdate {
                index: 0,
                value: joined,
            }];
            expected.extend((0..bitsize).map(|i| PortUpdate {
                index: 1 + i as usize,
                value: bitarr![Z; 1],
            }));
            assert_port_updates_eq(
                actual,
                expected,
                "identity splitter should rejoin legs into the bus"
            );
        }
    }

    // ---------- uneven split: new behavior the old splitter couldn't express ----------

    #[test]
    fn test_splitter_split_uneven_legs() {
        // 5-bit bus: leg 0 = bits [0,1,2], leg 1 = bits [3,4]
        let a = splitter_ports_slice(&[Some(0), Some(0), Some(0), Some(1), Some(1)]);
        let config = SplitterConfig::new(a, 2, 5).unwrap();
        let splitter = Splitter::new(config);
        let props = splitter.ports(&Default::default());

        assert_eq!(props.len(), 3);
        assert_eq!(
            props[0],
            PortProperties {
                ty: PortType::Inout,
                bitsize: 5
            }
        );
        assert_eq!(
            props[1],
            PortProperties {
                ty: PortType::Inout,
                bitsize: 3
            }
        );
        assert_eq!(
            props[2],
            PortProperties {
                ty: PortType::Inout,
                bitsize: 2
            }
        );

        let old_ports = floating_ports(&props);
        let mut new_ports = floating_ports(&props);

        // bits: [High, Low, High, Low, High] (0=H,1=L,2=H,3=L,4=H)
        let joined = alternating(5);
        assert!(new_ports[0].replace(joined).is_ok());

        let actual = splitter.run(RunContext {
            graphs: &Default::default(),
            old_ports: &old_ports,
            new_ports: &new_ports,
            inner_state: None,
        });

        let expected = vec![
            PortUpdate {
                index: 1,
                value: joined.subslice(0..3),
            },
            PortUpdate {
                index: 2,
                value: joined.subslice(3..5),
            },
            PortUpdate {
                index: 0,
                value: bitarr![Z; 5],
            },
        ];

        assert_port_updates_eq(
            actual,
            expected,
            "uneven-leg join should reassemble the bus correctly"
        );
    }

    #[test]
    fn test_splitter_join_uneven_legs() {
        let a = splitter_ports_slice(&[Some(0), Some(0), Some(0), Some(1), Some(1)]);
        let config = SplitterConfig::new(a, 2, 5).unwrap();
        let splitter = Splitter::new(config);
        let props = splitter.ports(&Default::default());

        let old_ports = floating_ports(&props);
        let mut new_ports = floating_ports(&props);

        let joined = alternating(5);
        let leg0 = joined.subslice(0..3);
        let leg1 = joined.subslice(3..5);

        assert!(new_ports[1].replace(leg0).is_ok());
        assert!(new_ports[2].replace(leg1).is_ok());

        let actual = splitter.run(RunContext {
            graphs: &Default::default(),
            old_ports: &old_ports,
            new_ports: &new_ports,
            inner_state: None,
        });
        let expected = vec![
            PortUpdate {
                index: 0,
                value: joined,
            },
            PortUpdate {
                index: 1,
                value: bitarr![Z; 3],
            },
            PortUpdate {
                index: 2,
                value: bitarr![Z; 2],
            },
        ];
        assert_port_updates_eq(
            actual,
            expected,
            "uneven-leg join should reassemble the bus correctly"
        );
    }

    // ---------- floating bits ----------

    #[test]
    fn test_splitter_join_preserves_floating_bit() {
        // 3-bit bus, bit 1 unassigned (floating). leg 0 = bit 0, leg 1 = bit 2.
        let a = splitter_ports_slice(&[Some(0), None, Some(1)]);
        let config = SplitterConfig::new(a, 2, 3).unwrap();
        let splitter = Splitter::new(config);
        let props = splitter.ports(&Default::default());

        assert_eq!(
            props[0],
            PortProperties {
                ty: PortType::Inout,
                bitsize: 3
            }
        );
        assert_eq!(
            props[1],
            PortProperties {
                ty: PortType::Inout,
                bitsize: 1
            }
        );
        assert_eq!(
            props[2],
            PortProperties {
                ty: PortType::Inout,
                bitsize: 1
            }
        );

        let mut old_ports = floating_ports(&props);
        // Seed the joined port's bit 1 to High before any leg activity,
        // so we can confirm it survives a leg-driven update untouched.
        let seeded = bitarr![0, 1, 0];
        assert!(old_ports[0].replace(seeded).is_ok());

        let mut new_ports = old_ports.clone();
        assert!(new_ports[1].replace(bitarr![1]).is_ok());
        assert!(new_ports[2].replace(bitarr![1]).is_ok());

        let actual = splitter.run(RunContext {
            graphs: &Default::default(),
            old_ports: &old_ports,
            new_ports: &new_ports,
            inner_state: None,
        });

        let expected_joined = bitarr![1, 1, 1];
        let expected = vec![
            PortUpdate {
                index: 0,
                value: expected_joined,
            },
            PortUpdate {
                index: 1,
                value: bitarr![Z; 1],
            },
            PortUpdate {
                index: 2,
                value: bitarr![Z; 1],
            },
        ];

        assert_port_updates_eq(
            actual,
            expected,
            "floating bit 1 should retain its prior value across a leg-driven join"
        );
    }
}
