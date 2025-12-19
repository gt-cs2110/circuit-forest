use crate::{bitarr, func};
use crate::middle_end::func::{PhysicalComponent, RelativeComponentBounds};

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

    fn bounds(&self) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_from_bitsize(self.sim.get_bitsize())
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

    fn bounds(&self) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_from_bitsize(self.sim.get_bitsize())
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

    fn bounds(&self) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_from_bitsize(self.sim.get_value().len())
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

    fn bounds(&self) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_with_origin(2, 3, (1, 3))
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

    fn bounds(&self) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_with_origin(2, 3, (1, 0))
    }
}

/// A tunnel.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Tunnel;
impl PhysicalComponent for Tunnel {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        None
    }

    fn component_name(&self) ->  &'static str {
        "Tunnel"
    }

    fn bounds(&self) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_with_origin(3, 2, (3, 1))
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

    fn bounds(&self) -> RelativeComponentBounds {
        let bitsize = i32::from(self.sim.get_bitsize());
        let ports = [(0, 0)].into_iter()
            .chain((1..=bitsize).map(|i| (2 * i, 2)));

        RelativeComponentBounds::new((bitsize * 2, 2), ports)
    }
}

#[cfg(test)]
mod tests {}
