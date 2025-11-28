use crate::func;
use crate::middle_end::func::PhysicalComponent;

use super::ComponentBounds;

/// An input.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Input {
    sim: func::Input
}
impl PhysicalComponent for Input {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) ->  &'static str {
        "Input"
    }

    fn bounds(&self) -> ComponentBounds {
        ComponentBounds::single_port_from_bitsize(self.sim.get_bitsize())
    }
}

/// An output.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Output {
    sim: func::Output
}
impl PhysicalComponent for Output {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) ->  &'static str {
        "Output"
    }

    fn bounds(&self) -> ComponentBounds {
        ComponentBounds::single_port_from_bitsize(self.sim.get_bitsize())
    }
}

/// A constant.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Constant {
    sim: func::Constant
}
impl PhysicalComponent for Constant {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) ->  &'static str {
        "Constant"
    }

    fn bounds(&self) -> ComponentBounds {
        ComponentBounds::single_port_from_bitsize(self.sim.get_value().len())
    }
}

/// A splitter component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Splitter {
    sim: func::Splitter
}
impl PhysicalComponent for Splitter {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) ->  &'static str {
        "Splitter"
    }

    fn bounds(&self) -> ComponentBounds {
        let bitsize = i32::from(self.sim.get_bitsize());
        let mut ports = vec![(0, 0)];
        ports.extend((1..=bitsize).map(|i| (2 * i, 2)));

        ComponentBounds {
            bounds: [(0, 0), (bitsize * 2, 2)],
            ports,
        }
    }
}

#[cfg(test)]
mod tests {}
