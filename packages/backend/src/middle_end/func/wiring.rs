use crate::{bitarr, func};
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

    fn component_name(&self) -> &'static str {
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

    fn component_name(&self) -> &'static str {
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

    fn component_name(&self) -> &'static str {
        "Constant"
    }

    fn bounds(&self) -> ComponentBounds {
        ComponentBounds::single_port_from_bitsize(self.sim.get_value().len())
    }
}

/// Power (essentially a constant 1).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Power;
impl PhysicalComponent for Power {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(func::Constant::new(bitarr![1]).into())
    }

    fn component_name(&self) -> &'static str {
        "Power"
    }

    fn bounds(&self) -> ComponentBounds {
        let origin = (1, 3);
        ComponentBounds::new_absolute((2, 3), vec![origin])
            .into_relative(origin)
    }
}

/// Ground (essentially a constant 0).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Ground;
impl PhysicalComponent for Ground {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(func::Constant::new(bitarr![0]).into())
    }

    fn component_name(&self) -> &'static str {
        "Ground"
    }

    fn bounds(&self) -> ComponentBounds {
        let origin = (1, 0);
        ComponentBounds::new_absolute((2, 3), vec![origin])
            .into_relative(origin)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Tunnel;
impl PhysicalComponent for Tunnel {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        None
    }

    fn component_name(&self) ->  &'static str {
        "Tunnel"
    }

    fn bounds(&self) -> ComponentBounds {
        let origin = (3, 1);
        ComponentBounds::new_absolute((3, 2), vec![origin])
            .into_relative(origin)
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

    fn component_name(&self) -> &'static str {
        "Splitter"
    }

    fn bounds(&self) -> ComponentBounds {
        let bitsize = i32::from(self.sim.get_bitsize());
        let ports = [(0, 0)].into_iter()
            .chain((1..=bitsize).map(|i| (2 * i, 2)))
            .collect();

        ComponentBounds {
            bounds: [(0, 0), (bitsize * 2, 2)],
            ports,
        }
    }
}

#[cfg(test)]
mod tests {}
