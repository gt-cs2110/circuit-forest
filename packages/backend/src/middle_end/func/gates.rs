use crate::engine::func::{self, BitSize, GateInputs};
use crate::middle_end::func::{AbsoluteComponentBounds, Handedness, Orientation, PhysicalComponent, PhysicalInitContext, RelativeComponentBounds};

pub use func::GateKind;

/// A gate component with a variable number of inputs.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Gate {
    kind: GateKind,
    bitsize: BitSize,
    n_inputs: GateInputs,
    orientation: Orientation
}
impl Gate {
    /// Creates a new instance of the gate with specified kind, bitsize, and input count.
    pub fn new(kind: GateKind, bitsize: BitSize, inputs: GateInputs,orientation: Orientation) -> Self {
        Self { kind, bitsize: bitsize, n_inputs: inputs, orientation: orientation }
    }
}
impl PhysicalComponent for Gate {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(func::Gate::new(self.kind, self.bitsize.get(), self.n_inputs.get()).into())
    }

    fn component_name(&self) -> &'static str {
        self.kind.into()
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        // The origin is at the output port, which is at (4,2) in absolute coordinates.
        let bounds = [(-4, -2), (0, 2)];
        let inputs = i32::from(self.n_inputs.get());
        
        let mut ports = vec![];
        // Input ports
        // For a 4-input gate, you have (-4, -3), (-4, -1), (-4, 1), (-4, 3)
        // For a 5-input gate, you have (-4, -4), (-4, -2), (-4, 0), (-4, 2), (-4, 4)
        ports.extend((0..inputs).map(|i| (-4, 2 * i - (inputs - 1))));
        ports.push((0, 0)); // Output port

        RelativeComponentBounds { bounds, ports }
            .orient(self.orientation, Default::default())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
/// A NOT gate component.
pub struct Not {
    bitsize: BitSize,
    orientation: Orientation
}
impl Not {
    /// Creates a new instance of the NOT gate with specified bitsize.
    pub fn new(bitsize: BitSize, orientation:Orientation) -> Self {
        Self { bitsize, orientation }
    }
}

impl PhysicalComponent for Not {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(func::Not::new(self.bitsize.get()).into())
    }

    fn component_name(&self) -> &'static str {
        "Not"
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        let origin = (3, 1);
        AbsoluteComponentBounds::new((3, 2), [(0, 1), origin])
            .into_relative(origin)
            .orient(self.orientation, Default::default())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
/// A tri-state buffer.
pub struct TriState {
    bitsize: BitSize,
    orientation: Orientation,
    handedness: Handedness
}
impl TriState {
    /// Creates a new instance of the tri-state buffer with specified bitsize.
    pub fn new(bitsize: BitSize, orientation:Orientation, handedness:Handedness) -> Self {
        Self { bitsize, orientation, handedness }
    }
}

impl PhysicalComponent for TriState {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(func::TriState::new(self.bitsize.get()).into())
    }

    fn component_name(&self) -> &'static str {
        "TriState"
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        let origin = (2, 1);
        // AbsoluteComponentBounds::new((2, 2), [(0, 1), (1, 2), origin])
        //     .into_relative(origin)
        //     .orient(self.orientation, Default::default())
        AbsoluteComponentBounds::new((2, 2), [(0, 1), (1, 2), origin])
        .into_relative(origin)
        .orient(self.orientation, self.handedness)
            
    }
}
