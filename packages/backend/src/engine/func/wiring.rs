use crate::bitarr;
use crate::bitarray::BitArray;
use crate::engine::CircuitGraphMap;
use crate::engine::func::SplitterConfigError::AssignmentOutOfBitsize;
use crate::engine::func::{BitSize, Component, PortProperties, PortType, PortUpdate, RunContext, Sensitivity, port_list};
use thiserror::Error;

/// An input.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Input {
    bitsize: BitSize
}
impl Input {
    /// Creates a new instance of the tri-state buffer with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self {
            bitsize: BitSize::new_clamped(bitsize)
        }
    }

    /// Gets the bitsize of this component.
    pub fn get_bitsize(&self) -> u8 {
        self.bitsize.get()
    }
}
impl Component for Input {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.bitsize.get() }, 1),
        ])
    }

    fn run_inner(&self, _ctx: RunContext<'_>) -> Vec<PortUpdate> {
        vec![]
    }
}

/// An output.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Output {
    bitsize: BitSize
}
impl Output {
    /// Creates a new instance of the tri-state buffer with specified bitsize.
    pub fn new(bitsize: u8) -> Self {
        Self {
            bitsize: BitSize::new_clamped(bitsize)
        }
    }

    /// Gets the bitsize of this component.
    pub fn get_bitsize(&self) -> u8 {
        self.bitsize.get()
    }
}
impl Component for Output {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // output
            (PortProperties { ty: PortType::Input, bitsize: self.bitsize.get() }, 1),
        ])
    }

    fn run_inner(&self, _ctx: RunContext<'_>) -> Vec<PortUpdate> {
        vec![]
    }
}

/// A constant.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Constant {
    value: BitArray
}
impl Constant {
    /// Creates a new instance of the tri-state buffer with specified bitsize.
    pub fn new(value: BitArray) -> Self {
        Self { value }
    }

    /// Gets the value which this constant holds.
    pub fn get_value(&self) -> BitArray {
        self.value
    }
}
impl Component for Constant {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        port_list(&[
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.value.len() }, 1),
        ])
    }

    fn initialize_port_state(&self, state: &mut [BitArray]) {
        state[0] = self.value;
    }
    fn run_inner(&self, _ctx: RunContext<'_>) -> Vec<PortUpdate> {
        vec![]
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SplitterConfig{
    //mapping of bits to legs
    #[serde(with = "serde_arrays")]
    port_assignments: [Option<u8>; 64],
    num_legs: u8,
    bitsize: BitSize

}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SplitterConfigError {
    #[error("bit {0} is assigned but out of range (bitsize is {1})")]
    AssignmentOutOfBitsize(usize, u8),
    #[error("leg index {0} is out of range (only {1} legs configured)")]
    LegOutOfRange(u8, u8),
   
}


impl SplitterConfig{
    pub fn new(port_assignments: [Option<u8>; 64], num_legs: u8, bitsize:u8) -> Result<Self, SplitterConfigError> {
                   

        for (bit,&leg) in port_assignments.iter().enumerate() {
            let Some(leg) = leg else {continue};
            if bit >= bitsize as usize{
                return Err(AssignmentOutOfBitsize((bit), (bitsize)))
            }

            if leg > num_legs {
                return Err(SplitterConfigError::LegOutOfRange(leg, num_legs));
            }
        }
        // catch legs with zero width
        for leg in 0..num_legs {
            if !port_assignments.contains(&Some(leg)) {
                //TODO Somehow pass up if a leg isnt used so it can auto condense; maybe let frontend do this and only auto condense visually

            }
        }
        Ok(Self { port_assignments, num_legs, bitsize:BitSize::new_clamped(bitsize)})
    }

    fn bits_for_leg(&self, leg:u8)->impl Iterator<Item=usize>{
        self.port_assignments.iter().enumerate().filter_map(move |(bit, &f)| {(f==Some(leg)).then_some(bit)})
    }
    fn leg_width(&self, leg:u8)->usize{
        self.bits_for_leg(leg).count()
    }
    pub fn get_bitsize(&self) -> BitSize{
        self.bitsize
    }
     pub fn get_num_legs(&self)->u8{
        self.num_legs
    }
    pub fn get_num_active_legs(&self)->usize{
        (0..self.get_num_legs()).filter(|&l| self.leg_width(l)>0).count()
    }
}


/// A splitter component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Splitter {
    config: SplitterConfig
}
impl Splitter {
    /// Creates a new instance of the Splitter with specified bitsize.
    pub fn new(config: SplitterConfig) -> Self {
        Self {
            config
        }
    }

   
}

impl Component for Splitter {
    fn ports(&self, _: &CircuitGraphMap) -> Vec<PortProperties> {
        let mut ports = vec![PortProperties { ty: PortType::Inout, bitsize: self.config.get_bitsize().get() }];
        ports.extend((0..self.config.get_num_legs()).into_iter().filter_map(|leg| (self.config.leg_width(leg)>0).then_some(PortProperties { ty: PortType::Inout, bitsize: self.config.leg_width(leg) as u8 })));
        ports
    }

    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        //removes inactive legs and shifts from leg index to corresponding port index

        let active_legs:Vec<(u8,usize)> = (0..self.config.get_num_legs()).filter(|&l| self.config.leg_width(l)>0).enumerate().map(|(index, leg)| (leg, index+1)).collect();
        
        if Sensitivity::Anyedge.activated(ctx.old_ports[0], ctx.new_ports[0]) {
            //set the leg values
           let mut updates:Vec<PortUpdate> = active_legs.iter().map(|&(leg,index)|{
                PortUpdate{index:index, value:self.config.bits_for_leg(leg).map(|bit|ctx.new_ports[0].index(bit as u8)).collect()}
           }).collect();
           //set the joined value to unknow
           updates.push(PortUpdate{index:0, value:bitarr![Z;self.config.get_bitsize().get()]});
           updates

        } else if Sensitivity::Anyedge.any_activated(&ctx.old_ports[1..], &ctx.new_ports[1..]) {
           // combine each legs bits back into position
            let mut value = ctx.new_ports[0].clone();
            for leg in 0..self.config.get_num_legs() {
                for (i, bit) in self.config.bits_for_leg(leg).enumerate() {
                    value = value.with(bit as u8, ctx.new_ports[(leg as usize) + 1].index(i as u8));
                }
            }
            //Drive the joined value 
            let mut updates = vec![PortUpdate { index: 0, value }];
            //set the legs to unknow so that they dont drive backwards and cause a short circuit
            updates.extend((0..self.config.get_num_legs()).map(|leg| {
            PortUpdate { index: leg as usize + 1, value: bitarr![Z; self.config.leg_width(leg) as u8] }
        }));
        updates


        } else {
            vec![]
        }
    }
}

#[cfg(test)]
#[cfg(test)]
mod tests {
    use crate::bitarray::BitState;
    use crate::engine::func::floating_ports;

    use super::*;

    /// Builds a fixed-size assignment array from a slice, padding the rest with `None`.
    fn assignments(bits: &[Option<u8>]) -> [Option<u8>; 64] {
        let mut arr = [None; 64];
        arr[..bits.len()].copy_from_slice(bits);
        arr
    }

    fn alternating(bitsize: u8) -> Vec<BitState> {
        (0..bitsize)
            .map(|i| if i % 2 == 0 { BitState::High } else { BitState::Low })
            .collect()
    }
    /// Builds an all-Z BitArray of the given width, for asserting a passive
    /// port was reset after the splitter stopped driving it.
    fn imped(width: u8) -> BitArray {
        crate::bitarr![Z; width]
    }

    /// Sorts a Vec<PortUpdate> by index, for order-independent comparison.
    fn sorted(mut v: Vec<PortUpdate>) -> Vec<PortUpdate> {
        v.sort_by_key(|u| u.index);
        v
    }

    // ---------- SplitterConfig::new validation ----------

    #[test]
    fn config_rejects_leg_out_of_range() {
        let a = assignments(&[Some(5)]);
        let err = SplitterConfig::new(a, 2, 1).unwrap_err();
        assert_eq!(err, SplitterConfigError::LegOutOfRange(5, 2));
    }

    
    #[test]
    fn config_rejects_assignment_beyond_bitsize() {
        let a = assignments(&[Some(0), Some(0), None, Some(1)]);
        let err = SplitterConfig::new(a, 2, 2).unwrap_err();
        assert_eq!(err, SplitterConfigError::AssignmentOutOfBitsize(3, 2));
    }

    // ---------- identity split (mirrors old Splitter::new(bitsize) behavior) ----------

    fn identity_config(bitsize: u8) -> SplitterConfig {
        let mut a = [None; 64];
        for i in 0..bitsize {
            a[i as usize] = Some(i);
        }
        SplitterConfig::new(a, bitsize, bitsize)
            .expect("identity split should always be valid")
    }

    #[test]
    fn test_splitter_split_identity() {
        for bitsize in BitArray::MIN_BITSIZE..=BitArray::MAX_BITSIZE {
            let config = identity_config(bitsize);
            let splitter = Splitter::new(config);
            let props = splitter.ports(&Default::default());

            assert_eq!(props.len(), 1 + bitsize as usize);
            assert_eq!(props[0], PortProperties { ty: PortType::Inout, bitsize });
            assert_eq!(
                props[1..],
                vec![PortProperties { ty: PortType::Inout, bitsize: 1 }; bitsize as usize]
            );

            let old_ports = floating_ports(&props);
            let mut new_ports = floating_ports(&props);

            let data = alternating(bitsize);
            let joined = BitArray::from_iter(data.iter().copied());
            assert!(new_ports[0].replace(joined).is_ok());

            let actual = splitter.run(RunContext {
                graphs: &Default::default(),
                old_ports: &old_ports,
                new_ports: &new_ports,
                inner_state: None,
            });
            let mut expected: Vec<_> = data.iter().enumerate()
            .map(|(i, &st)| PortUpdate { index: 1 + i, value: BitArray::from(st) })
            .collect();
            expected.push(PortUpdate { index: 0, value: imped(bitsize) });

            assert_eq!(sorted(actual), sorted(expected), "identity splitter should split each bit to its own leg");
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

            let data = alternating(bitsize);
            let joined = BitArray::from_iter(data.iter().copied());
            let split: Vec<_> = data.into_iter().map(BitArray::from).collect();

            for (p, arr) in std::iter::zip(new_ports[1..].iter_mut(), split) {
                assert!(p.replace(arr).is_ok());
            }

            let actual = splitter.run(RunContext {
                graphs: &Default::default(),
                old_ports: &old_ports,
                new_ports: &new_ports,
                inner_state: None,
            });
            let mut expected = vec![PortUpdate { index: 0, value: joined }];
            expected.extend((0..bitsize).map(|i| PortUpdate { index: 1 + i as usize, value: imped(1) }));
            assert_eq!(sorted(actual), sorted(expected), "identity splitter should rejoin legs into the bus");
        }
    }

    // ---------- uneven split: new behavior the old splitter couldn't express ----------

    #[test]
    fn test_splitter_split_uneven_legs() {
        // 5-bit bus: leg 0 = bits [0,1,2], leg 1 = bits [3,4]
        let a = assignments(&[Some(0), Some(0), Some(0), Some(1), Some(1)]);
        let config = SplitterConfig::new(a, 2, 5).unwrap();
        let splitter = Splitter::new(config);
        let props = splitter.ports(&Default::default());

        assert_eq!(props.len(), 3);
        assert_eq!(props[0], PortProperties { ty: PortType::Inout, bitsize: 5 });
        assert_eq!(props[1], PortProperties { ty: PortType::Inout, bitsize: 3 });
        assert_eq!(props[2], PortProperties { ty: PortType::Inout, bitsize: 2 });

        let old_ports = floating_ports(&props);
        let mut new_ports = floating_ports(&props);

        // bits: [High, Low, High, Low, High] (0=H,1=L,2=H,3=L,4=H)
        let data = alternating(5);
        let joined = BitArray::from_iter(data.iter().copied());
        assert!(new_ports[0].replace(joined).is_ok());

        let actual = splitter.run(RunContext {
            graphs: &Default::default(),
            old_ports: &old_ports,
            new_ports: &new_ports,
            inner_state: None,
        });

       let expected = vec![
        PortUpdate { index: 1, value: BitArray::from_iter(data[0..3].iter().copied()) },
        PortUpdate { index: 2, value: BitArray::from_iter(data[3..5].iter().copied()) },
        PortUpdate { index: 0, value: imped(5) },
        ];

        assert_eq!(sorted(actual), sorted(expected),"uneven-leg join should reassemble the bus correctly");
    }

    #[test]
    fn test_splitter_join_uneven_legs() {
        let a = assignments(&[Some(0), Some(0), Some(0), Some(1), Some(1)]);
        let config = SplitterConfig::new(a, 2, 5).unwrap();
        let splitter = Splitter::new(config);
        let props = splitter.ports(&Default::default());

        let old_ports = floating_ports(&props);
        let mut new_ports = floating_ports(&props);

        let data = alternating(5);
        let joined = BitArray::from_iter(data.iter().copied());
        let leg0 = BitArray::from_iter(data[0..3].iter().copied());
        let leg1 = BitArray::from_iter(data[3..5].iter().copied());

        assert!(new_ports[1].replace(leg0).is_ok());
        assert!(new_ports[2].replace(leg1).is_ok());

        let actual = splitter.run(RunContext {
            graphs: &Default::default(),
            old_ports: &old_ports,
            new_ports: &new_ports,
            inner_state: None,
        });
        let expected = vec![
            PortUpdate { index: 0, value: joined },
            PortUpdate { index: 1, value: imped(3) },
            PortUpdate { index: 2, value: imped(2) },
        ];
        assert_eq!(sorted(actual), sorted(expected), "uneven-leg join should reassemble the bus correctly");
    }

    // ---------- floating bits ----------

    #[test]
    fn test_splitter_join_preserves_floating_bit() {
        // 3-bit bus, bit 1 unassigned (floating). leg 0 = bit 0, leg 1 = bit 2.
        let a = assignments(&[Some(0), None, Some(1)]);
        let config = SplitterConfig::new(a, 2, 3).unwrap();
        let splitter = Splitter::new(config);
        let props = splitter.ports(&Default::default());

        assert_eq!(props[0], PortProperties { ty: PortType::Inout, bitsize: 3 });
        assert_eq!(props[1], PortProperties { ty: PortType::Inout, bitsize: 1 });
        assert_eq!(props[2], PortProperties { ty: PortType::Inout, bitsize: 1 });

        let mut old_ports = floating_ports(&props);
        // Seed the joined port's bit 1 to High before any leg activity,
        // so we can confirm it survives a leg-driven update untouched.
        let seeded = BitArray::from_iter([BitState::Low, BitState::High, BitState::Low]);
        assert!(old_ports[0].replace(seeded.clone()).is_ok());

        let mut new_ports = old_ports.clone();
        assert!(new_ports[1].replace(BitArray::from(BitState::High)).is_ok());
        assert!(new_ports[2].replace(BitArray::from(BitState::High)).is_ok());

        let actual = splitter.run(RunContext {
            graphs: &Default::default(),
            old_ports: &old_ports,
            new_ports: &new_ports,
            inner_state: None,
        });

        let expected_joined = BitArray::from_iter([BitState::High, BitState::High, BitState::High]);
        let expected = vec![
            PortUpdate { index: 0, value: expected_joined },
            PortUpdate { index: 1, value: imped(1) },
            PortUpdate { index: 2, value: imped(1) },
        ];

        assert_eq!(sorted(actual), sorted(expected), "floating bit 1 should retain its prior value across a leg-driven join");
    }
}