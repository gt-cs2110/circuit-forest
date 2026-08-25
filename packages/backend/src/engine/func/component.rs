use crate::bitarray::BitArray;
use crate::engine::func::{self, Divider, Multiplier, PortUpdate, RunContext};
use crate::engine::state::InnerFunctionState;
use crate::engine::{CircuitGraphMap, PortProperties};

/// Helper function to validate ports.
///
/// This panics if the ports do not align with the port properties.
fn validate_ports(props: &[PortProperties], ports: &[BitArray]) {
    assert_eq!(ports.len(), props.len(), "Expected correct number of ports");
    for (i, (bit_vec, port)) in ports.iter().zip(props).enumerate() {
        assert_eq!(
            bit_vec.len(),
            port.bitsize,
            "Port {i} has incorrect bit width"
        );
    }
}

/// The interface defining how a digital logic component operates.
pub trait Component {
    /// Returns the vector holding the properties of all ports associated with the component.
    ///
    /// This is called only once during initialization.
    /// It is assumed that the result of this function will not change when called multiple times.
    fn ports(&self, graphs: &CircuitGraphMap) -> Vec<PortProperties>;

    /// Initializes the port state of the component.
    ///
    /// If not specified, by default, the initial port state is set to all floating.
    fn initialize_port_state(&self, _state: &mut [BitArray]) {}

    /// Initializes the internal state of the component.
    ///
    /// If not specified, by default, this is `Default::default()`.
    fn initialize_inner_state(&self, _graphs: &CircuitGraphMap) -> Option<InnerFunctionState> {
        None
    }

    /// "Runs" the component's function on a set of inputs, outputting a vector of updated ports
    /// after the function is applied.
    ///
    /// This function is called after an update is propagated to this component.
    /// When that occurs, this function is called with the original state and updated state
    /// of this component's ports.
    ///
    /// This function may also panic if the port fields of [`RunContext`] do not match the port properties
    /// specified by [`Component::ports`].
    #[must_use]
    fn run(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        // Only run in debug mode
        if cfg!(debug_assertions) {
            let props = self.ports(ctx.graphs);
            validate_ports(&props, ctx.old_ports);
            validate_ports(&props, ctx.new_ports);
        }
        self.run_inner(ctx)
    }

    /// Inner run function that, given a set of inputs, applies its modifications to output a vector
    /// of updated ports. This function is wrapped by run to ensure input validation
    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate>;
}

// Note: We don't use the `enum_dispatch` crate because it's old and has limitations
// (e.g., can only implement one trait at a time and the structure of the trait is limited).
//
// Though this adds more code, it doesn't add too much more, so we'll run with it.

macro_rules! enum_dispatch {
    ($(#[$m:meta])* $vis:vis enum $Enum:ident { $($Variant:ident),*$(,)? }) => {
        $(#[$m])*
        $vis enum $Enum {
            $($Variant(func::$Variant)),*
        }

        // From and TryFrom traits
        $(
            impl From<func::$Variant> for $Enum {
                fn from(value: func::$Variant) -> Self {
                    $Enum::$Variant(value)
                }
            }
            impl TryFrom<$Enum> for func::$Variant {
                type Error = &'static str;
                fn try_from(value: $Enum) -> Result<Self, Self::Error> {
                    match value {
                        $Enum::$Variant(value) => Ok(value),
                        _ => Err(concat!("could not convert ", stringify!($Enum), " to ", stringify!($Variant)))
                    }
                }
            }
        )*
    }
}
pub(crate) use enum_dispatch;

macro_rules! component_enum {
    ($(#[$m:meta])* $vis:vis enum $Enum:ident { $($Variant:ident),*$(,)? }) => {
        enum_dispatch! { $(#[$m])* $vis enum $Enum { $($Variant),* } }
        
        // Implement Component for this type
        impl Component for $Enum {
            fn ports(&self, graphs: &CircuitGraphMap) -> Vec<PortProperties> {
                match self {
                    $(Self::$Variant(c) => Component::ports(c, graphs)),*
                }
            }
            fn initialize_port_state(&self, state: &mut [BitArray]) {
                match self {
                    $(Self::$Variant(c) => Component::initialize_port_state(c, state)),*
                }
            }
            fn initialize_inner_state(&self, graphs: &CircuitGraphMap) -> Option<InnerFunctionState> {
                match self {
                    $(Self::$Variant(c) => Component::initialize_inner_state(c, graphs)),*
                }
            }
            fn run(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
                match self {
                    $(Self::$Variant(c) => Component::run(c, ctx)),*
                }
            }
            fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
                match self {
                    $(Self::$Variant(c) => Component::run_inner(c, ctx)),*
                }
            }
        }
    }
}

component_enum! {
    /// An enum that represents all supported digital logic components.
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
    #[allow(missing_docs)]
    pub enum ComponentFn {
        // Gates
        Gate, Not, TriState,
        // Wiring
        Input, Output, Constant, Splitter,
        // Muxes
        Mux, Demux, Decoder,
        // Memory
        Register,
        // Misc
        Subcircuit,
        // Arithmetic
        Adder, Subtractor, Multiplier, Divider

    }
}
