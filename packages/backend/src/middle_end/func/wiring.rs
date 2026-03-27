use crate::bitarray::BitArray;
use crate::engine::func;
use crate::bitarr;
use crate::middle_end::func::{Handedness, Orientation, PhysicalComponent, PhysicalInitContext, RelativeComponentBounds};

/// An input.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Pin {
    bitsize: u8,
    is_input: bool,
    orientation: Orientation
}
impl PhysicalComponent for Pin {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(match self.is_input {
            true  => func::Input::new(self.bitsize).into(),
            false => func::Output::new(self.bitsize).into()
        })
    }

    fn component_name(&self) -> &'static str {
        match self.is_input {
            true  => "Input",
            false => "Output"
        }
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_from_bitsize(self.bitsize)
            .orient(self.orientation, Default::default())
    }
}

/// A constant.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Constant {
    value: BitArray,
    orientation: Orientation
}
impl PhysicalComponent for Constant {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(func::Constant::new(self.value).into())
    }

    fn component_name(&self) -> &'static str {
        "Constant"
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_from_bitsize(self.value.len())
            .orient(self.orientation, Default::default())
    }
}

/// Power (essentially a constant 1).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Power;
impl PhysicalComponent for Power {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(func::Constant::new(bitarr![1]).into())
    }

    fn component_name(&self) -> &'static str {
        "Power"
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_with_origin(2, 3, (1, 3))
    }
}

/// Ground (essentially a constant 0).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Ground;
impl PhysicalComponent for Ground {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(func::Constant::new(bitarr![0]).into())
    }

    fn component_name(&self) -> &'static str {
        "Ground"
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_with_origin(2, 3, (1, 0))
    }
}

/// A splitter component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Splitter {
    bitsize: u8,
    orientation: Orientation,
    handedness: Handedness
}
impl PhysicalComponent for Splitter {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(func::Splitter::new(self.bitsize).into())
    }

    fn component_name(&self) -> &'static str {
        "Splitter"
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        let bitsize = i32::from(self.bitsize);
        let mut ports = vec![(0, 0)];
        ports.extend((1..=bitsize).map(|i| (2 * i, 2)));

        RelativeComponentBounds::new((bitsize * 2, 2), ports)
            .orient(self.orientation, self.handedness)
    }
}

/// A tunnel.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Tunnel {
    orientation: Orientation
}
impl PhysicalComponent for Tunnel {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        None
    }

    fn component_name(&self) ->  &'static str {
        "Tunnel"
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port_with_origin(3, 2, (3, 1))
            .orient(self.orientation, Default::default())
    }
}

/// A probe.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Probe {
    orientation: Orientation
}
impl PhysicalComponent for Probe {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        None
    }

    fn component_name(&self) ->  &'static str {
        "Probe"
    }

    fn init_bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        RelativeComponentBounds::single_port(2, 2)
            .orient(self.orientation, Default::default())
    }
}

#[cfg(test)]
mod tests {}
