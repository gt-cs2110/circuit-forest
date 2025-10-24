use crate::bitarray::{BitArray, BitState};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
pub enum PortType { Input, Output, Inout }
impl PortType {
    pub fn accepts_input(self) -> bool {
        matches!(self, PortType::Input | PortType::Inout)
    }
    pub fn accepts_output(self) -> bool {
        matches!(self, PortType::Output | PortType::Inout)
    }
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
pub struct PortProperties {
    pub ty: PortType,
    pub bitsize: u8
}

#[derive(Debug, PartialEq, Eq)]
pub struct PortUpdate {
    pub index: usize,
    pub value: BitArray
}

pub trait Component {
    fn ports(&self) -> Vec<PortProperties>;
    fn initialize(&self, _state: &mut [BitArray]) {}
    #[must_use]
    fn run(&self, old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate>;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sensitivity {
    Anyedge, Posedge, Negedge, DontCare
}
impl Sensitivity {
    pub fn activated(self, old: BitArray, new: BitArray) -> bool {
        assert_eq!(old.len(), new.len(), "Bit length should be the same");
        match self {
            Sensitivity::Anyedge  => old != new,
            Sensitivity::Posedge  => old.all(BitState::Low) && new.all(BitState::High),
            Sensitivity::Negedge  => old.all(BitState::High) && new.all(BitState::Low),
            Sensitivity::DontCare => false,
        }
    }
    pub fn any_activated(self, old: &[BitArray], new: &[BitArray]) -> bool {
        assert_eq!(old.len(), new.len(), "Array size should be the same");
        std::iter::zip(old, new)
            .any(|(&o, &n)| self.activated(o, n))
    }
}

fn port_list(config: &[(PortProperties, u8)]) -> Vec<PortProperties> {
    config.iter()
        .flat_map(|&(props, ct)| std::iter::repeat_n(props, usize::from(ct)))
        .collect()
}
macro_rules! decl_component_enum {
    ($ComponentEnum:ident: $($Component:ident),*$(,)?) => {
        pub enum $ComponentEnum {
            $($Component($Component)),*
        }
        impl Component for $ComponentEnum {
            fn ports(&self) -> Vec<PortProperties> {
                match self {
                    $(
                        Self::$Component(c) => c.ports(),
                    )*
                }
            }
            fn initialize(&self, state: &mut [BitArray]) {
                match self {
                    $(
                        Self::$Component(c) => c.initialize(state),
                    )*
                }
            }
            fn run(&self, old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
                match self {
                    $(
                        Self::$Component(c) => c.run(old_inp, inp),
                    )*
                }
            }
        }
        $(
            impl From<$Component> for $ComponentEnum {
                fn from(value: $Component) -> Self {
                    Self::$Component(value)
                }
            }
        )*
    }
}
decl_component_enum!(ComponentFn: 
    And, Or, Xor, Nand, Nor, Xnor, Not, TriState, 
    Mux, Demux, Decoder, Splitter, Register
);

pub const MIN_GATE_INPUTS: u8 = 2;
pub const MAX_GATE_INPUTS: u8 = 64;
pub struct GateProperties {
    bitsize: u8,
    n_inputs: u8
}

macro_rules! gates {
    ($($Id:ident: $f:expr; invert: $Invert:expr),*$(,)?) => {
        $(
            pub struct $Id {
                props: GateProperties
            }
            impl $Id {
                pub fn new(mut bitsize: u8, mut n_inputs: u8) -> Self {
                    bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
                    n_inputs = n_inputs.clamp(MIN_GATE_INPUTS, MAX_GATE_INPUTS);
                    Self { props: GateProperties { bitsize, n_inputs }}
                }
            }
            impl Component for $Id {
                fn ports(&self) -> Vec<PortProperties> {
                    port_list(&[
                        // inputs
                        (PortProperties { ty: PortType::Input, bitsize: self.props.bitsize }, self.props.n_inputs),
                        // outputs
                        (PortProperties { ty: PortType::Output, bitsize: self.props.bitsize }, 1),
                    ])
                }
                fn run(&self, _old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
                    let value = inp[..usize::from(self.props.n_inputs)].iter()
                        .cloned()
                        .reduce($f)
                        .unwrap_or_else(|| BitArray::unknown(self.props.bitsize));
                    let value = if $Invert { !value } else { value };
    
                    vec![PortUpdate { index: usize::from(self.props.n_inputs), value }]
                }
            }
        )*
    }
}

gates! {
    And:  |a, b| a & b; invert: false,
    Or:   |a, b| a | b; invert: false,
    Xor:  |a, b| a ^ b; invert: false,
    Nand: |a, b| a & b; invert: true,
    Nor:  |a, b| a | b; invert: true,
    Xnor: |a, b| a ^ b; invert: true,
}

pub struct BufNotProperties {
    bitsize: u8
}
pub struct Not {
    props: BufNotProperties
}
impl Not {
    pub fn new(mut bitsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        Self { props: BufNotProperties { bitsize }}
    }
}
impl Component for Not {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // input
            (PortProperties { ty: PortType::Input, bitsize: self.props.bitsize }, 1),
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.props.bitsize }, 1),
        ])
    }

    fn run(&self, _old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        vec![PortUpdate { index: 1, value: !inp[0] }]
    }
}

pub struct TriState {
    props: BufNotProperties
}
impl TriState {
    pub fn new(mut bitsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        Self { props: BufNotProperties { bitsize }}
    }
}
impl Component for TriState {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // selector
            (PortProperties { ty: PortType::Input, bitsize: 1 }, 1),
            // input
            (PortProperties { ty: PortType::Input, bitsize: self.props.bitsize }, 1),
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.props.bitsize }, 1),
        ])
    }

    fn run(&self, _old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        let gate = inp[0].index(0);
        let result = match gate {
            BitState::High => inp[1],
            BitState::Low | BitState::Imped => BitArray::floating(self.props.bitsize),
            BitState::Unk => BitArray::unknown(self.props.bitsize),
        };
        vec![PortUpdate { index: 2, value: result }]
    }
}

pub const MIN_SELSIZE: u8 = 1;
pub const MAX_SELSIZE: u8 = 6;
pub struct MuxProperties {
    bitsize: u8,
    selsize: u8
}
pub struct Mux {
    props: MuxProperties
}
impl Mux {
    pub fn new(mut bitsize: u8, mut selsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        selsize = selsize.clamp(MIN_SELSIZE, MAX_SELSIZE);
        Self { props: MuxProperties { bitsize, selsize }}
    }
}
impl Component for Mux {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // selector
            (PortProperties { ty: PortType::Input, bitsize: self.props.selsize }, 1),
            // inputs
            (PortProperties { ty: PortType::Input, bitsize: self.props.bitsize }, 1 << self.props.selsize),
            // output
            (PortProperties { ty: PortType::Output, bitsize: self.props.bitsize }, 1),
        ])
    }

    fn run(&self, _old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        let m_sel = u64::try_from(inp[0]);
        let result = match m_sel {
            Ok(sel) => inp[sel as usize + 1],
            Err(e) => BitArray::repeat(e.bit_state(), self.props.bitsize),
        };
        vec![PortUpdate { index: (1 << self.props.selsize) + 1, value: result }]
    }
}
pub struct Demux {
    props: MuxProperties
}
impl Demux {
    pub fn new(mut bitsize: u8, mut selsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        selsize = selsize.clamp(MIN_SELSIZE, MAX_SELSIZE);
        Self { props: MuxProperties { bitsize, selsize }}
    }
}
impl Component for Demux {
    fn ports(&self) -> Vec<PortProperties> {
            port_list(&[
            // selector
            (PortProperties { ty: PortType::Input, bitsize: self.props.selsize }, 1),
            // input
            (PortProperties { ty: PortType::Input, bitsize: self.props.bitsize }, 1),
            // outputs
            (PortProperties { ty: PortType::Output, bitsize: self.props.bitsize }, 1 << self.props.selsize),
        ])
    }
    fn run(&self, _old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        let m_sel = u64::try_from(inp[0]);
        let result = match m_sel {
            Ok(sel) => {
                let mut result = vec![BitArray::repeat(BitState::Low, self.props.bitsize); 1 << self.props.selsize];
                result[sel as usize] = inp[1];
                result
            },
            Err(e) => vec![BitArray::repeat(e.bit_state(), self.props.bitsize); 1 << self.props.selsize],
        };

        result.into_iter()
            .enumerate()
            .map(|(i, value)| PortUpdate { index: 2 + i, value })
            .collect()
    }
}

pub struct DecoderProperties {
    selsize: u8
}
pub struct Decoder {
    props: DecoderProperties
}
impl Decoder {
    pub fn new(mut selsize: u8) -> Self {
        selsize = selsize.clamp(MIN_SELSIZE, MAX_SELSIZE);
        Self { props: DecoderProperties { selsize }}
    }
}
impl Component for Decoder {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // selector
            (PortProperties { ty: PortType::Input, bitsize: self.props.selsize }, 1),
            // outputs
            (PortProperties { ty: PortType::Output, bitsize: 1 }, 1 << self.props.selsize),
        ])
    }

    fn run(&self, _old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        let m_sel = u64::try_from(inp[0]);
        let result = match m_sel {
            Ok(sel) => {
                let mut result = vec![BitArray::from_iter([BitState::Low]); 1 << self.props.selsize];
                result[sel as usize] = BitArray::from_iter([BitState::High]);
                result
            },
            Err(e) => vec![BitArray::from_iter([e.bit_state()]); 1 << self.props.selsize],
        };

        result.into_iter()
            .enumerate()
            .map(|(i, value)| PortUpdate { index: 1 + i, value })
            .collect()
    }
}

pub struct Splitter {
    props: BufNotProperties
}
impl Splitter {
    pub fn new(mut bitsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        Self { props: BufNotProperties { bitsize } }
    }
}
impl Component for Splitter {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // joined
            (PortProperties { ty: PortType::Inout, bitsize: self.props.bitsize }, 1),
            // split
            (PortProperties { ty: PortType::Inout, bitsize: 1 }, self.props.bitsize),
        ])
    }

    fn run(&self, old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        if Sensitivity::Anyedge.activated(old_inp[0], inp[0]) {
            std::iter::zip(1.., inp[0])
                .map(|(index, bit)| PortUpdate { index, value: BitArray::from_iter([bit]) })
                .collect()
        } else if Sensitivity::Anyedge.any_activated(&old_inp[1..], &inp[1..]) {
            let value = inp[1..].iter()
                .map(|b| b.index(0))
                .collect();
            vec![PortUpdate { index: 0, value }]
        } else {
            vec![]
        }
    }
}

pub struct Register {
    props: BufNotProperties
}
impl Register {
    pub fn new(mut bitsize: u8) -> Self {
        bitsize = bitsize.clamp(BitArray::MIN_BITSIZE, BitArray::MAX_BITSIZE);
        Self { props: BufNotProperties { bitsize } }
    }
}
impl Component for Register {
    fn ports(&self) -> Vec<PortProperties> {
        port_list(&[
            // din
            (PortProperties { ty: PortType::Input, bitsize: self.props.bitsize }, 1),
            // enable, clock, clear
            (PortProperties { ty: PortType::Input, bitsize: 1 }, 3),
            // dout
            (PortProperties { ty: PortType::Output, bitsize: self.props.bitsize }, 1),
        ])
    }
    fn initialize(&self, state: &mut [BitArray]) {
        state[4] = BitArray::repeat(BitState::Low, self.props.bitsize);
    }
    fn run(&self, old_inp: &[BitArray], inp: &[BitArray]) -> Vec<PortUpdate> {
        if inp[3].all(BitState::High) {
            vec![PortUpdate { index: 4, value: BitArray::repeat(BitState::Low, self.props.bitsize) }]
        } else if Sensitivity::Posedge.activated(old_inp[2], inp[2]) && inp[1].all(BitState::High) {
            vec![PortUpdate { index: 4, value: inp[0] }]
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod muxes {
        use super::*;
        use crate::bitarray::BitArray;

        #[test]
        fn test_mux() {
            // use all possible selector sizes
            for selsize in MIN_SELSIZE..=MAX_SELSIZE {
                // 2^selsize *data* inputs
                let input_count = 1 << selsize;

                // create mux
                let mux = Mux::new(4, selsize);
                let ports = mux.ports();
    
                assert_eq!(ports.len(), input_count + 2, "Mux with selsize {selsize} should have {} ports", input_count + 2);
                assert_eq!(ports[0], PortProperties { ty: PortType::Input, bitsize: selsize }, "First Mux port should be an input selector of bitsize {selsize}");
                assert_eq!(ports[input_count + 1], PortProperties { ty: PortType::Output, bitsize: 4 }, "Last Mux port should be an output of bitsize 4");
                assert_eq!(
                    ports[1..=input_count],
                    vec![PortProperties { ty: PortType::Input, bitsize: 4 }; input_count],
                    "Mux with selsize {selsize} should have {input_count} input ports"
                );

                // inputs are random-ish values
                let inputs: Vec<BitArray> = (0..input_count)
                    .map(|i| (i + 1) * 13)
                    .map(|val| BitArray::from(val as u64 & 0xF))
                    .collect();

                // test all possible selector values
                for sel in 0..selsize {
                    let mut inp = vec![BitArray::from(sel as u64)];
                    inp.push(inputs[sel as usize]);

                    let outputs = mux.run(&inp, &inp);
    
                    assert_eq!(
                        outputs,
                        vec![PortUpdate { index: 1 + input_count, value: inputs[sel as usize] }],
                        "Mux with selsize {selsize} and selector {sel} should output correct value"
                    )
                }
            }
        }

        #[test]
        fn test_demux() {
            // use all possible selector sizes
            for selsize in MIN_SELSIZE..=MAX_SELSIZE {
                // 2^selsize *data* inputs
                let input_count = 1 << selsize;

                // create demux
                let demux = Demux::new(4, selsize);
                let ports = demux.ports();

                assert_eq!(ports.len(), input_count + 2, "Demux with selsize {selsize} should have {} ports", input_count + 2);
                assert_eq!(ports[0], PortProperties { ty: PortType::Input, bitsize: selsize }, "First Demux port should be an input selector of bitsize {selsize}");
                assert_eq!(ports[1], PortProperties { ty: PortType::Input, bitsize: 4 }, "Last Demux port should be an output of bitsize 4");
                assert_eq!(
                    ports[2..=input_count + 1],
                    vec![PortProperties { ty: PortType::Output, bitsize: 4 }; input_count],
                    "Demux with selsize {selsize} should have {input_count} output ports"
                );

                // inputs are random-ish values
                let inputs: Vec<BitArray> = (0..input_count)
                    .map(|i| (i + 1) * 13)
                    .map(|val| BitArray::from(val as u64 & 0xF))
                    .collect();

                // test all possible selector values
                for sel in 0..=selsize {
                    let mut inp = vec![BitArray::from(sel as u64)];
                    inp.push(inputs[sel as usize]);
    
                    let outputs = demux.run(&inp, &inp);

                    let expected = (0..input_count)
                        .map(|i| {
                            if i == sel as usize {
                                inputs[sel as usize]
                            } else {
                                BitArray::repeat(BitState::Low, 4)
                                // BitArray::from(0u64)
                            }
                        })
                        .enumerate()
                        .map(|(i, value)| PortUpdate { index: 2 + i, value })
                        .collect::<Vec<_>>();

                    assert_eq!(
                        outputs,
                        expected,
                        "Demux with selsize {selsize} and selector {sel} should output correct value"
                    );
                }
            }
        }

        #[test]
        fn test_decoder() {
            // use all possible selector sizes
            for selsize in MIN_SELSIZE..=MAX_SELSIZE {
                // 2^selsize outputs
                let output_count = 1 << selsize;

                // create decoder
                let decoder = Decoder::new(selsize);
                let ports = decoder.ports();

                assert_eq!(ports.len(), output_count + 1, "Decoder with selsize {selsize} should have {} ports", output_count + 1);
                assert_eq!(ports[0], PortProperties { ty: PortType::Input, bitsize: selsize }, "First Decoder port should be an input selector of bitsize {selsize}");
                assert_eq!(
                    ports[1..],
                    vec![PortProperties { ty: PortType::Output, bitsize: 1 }; output_count],
                    "Decoder with selsize {selsize} should have {output_count} output ports"
                );

                // test all possible selector values
                for sel in 0..=selsize {
                    let inp = vec![BitArray::from(sel as u64)];
    
                    let outputs = decoder.run(&inp, &inp);

                    let expected = (0..output_count)
                        .map(|i| {
                            if i == sel as usize {
                                BitArray::from_iter([BitState::High])
                            } else {
                                BitArray::from_iter([BitState::Low])
                            }
                        })
                        .enumerate()
                        .map(|(i, value)| PortUpdate { index: 1 + i, value })
                        .collect::<Vec<_>>();

                    assert_eq!(
                        outputs,
                        expected,
                        "Decoder with selsize {selsize} and selector {sel} should output correct value"
                    );
                }
            }
        }
    }

    mod splitter {
        use super::*;
        use crate::bitarray::BitArray;

        #[test]
        fn test_splitter() {
            for bitsize in BitArray::MIN_BITSIZE..=BitArray::MAX_BITSIZE {
                let splitter = Splitter::new(bitsize);
                let ports = splitter.ports();

                assert_eq!(ports.len(), 1 + bitsize as usize, "Splitter with bitsize {bitsize} should have {} ports", 1 + bitsize as usize);
                assert_eq!(ports[0], PortProperties { ty: PortType::Inout, bitsize }, "First Splitter port should be an inout of bitsize {bitsize}");
                assert_eq!(
                    ports[1..],
                    vec![PortProperties { ty: PortType::Inout, bitsize: 1 }; bitsize as usize],
                    "Splitter with bitsize {bitsize} should have {bitsize} split ports"
                );

                let old_inp = vec![BitArray::from_iter(vec![BitState::Low; bitsize as usize])];
                let inp = vec![BitArray::from_iter(
                    (0..bitsize).map(|i| if i % 2 == 0 { BitState::High } else { BitState::Low })
                )];

                let outputs = splitter.run(&old_inp, &inp);

                let expected: Vec<PortUpdate> = (0..bitsize)
                    .map(|i| {
                        let bit = inp[0].index(i);
                        PortUpdate { index: 1 + i as usize, value: BitArray::from_iter([bit]) }
                    })
                    .collect();

                assert_eq!(
                    outputs,
                    expected,
                    "Splitter should correctly split the input bits"
                );
            }
        }
    }
}
