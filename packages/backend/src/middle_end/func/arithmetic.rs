use crate::{engine::func::{self, BitSize, SignType}, middle_end::func::{Orientation, PhysicalComponent, RelativeComponentBounds}};



/// Adder with carryin and input a,b
 #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Adder{
    bitsize: BitSize,
    orientation: Orientation,
}
/// adder construcor
impl Adder{
    /// constructor
    pub fn new(bitsize:u8, orientation: Orientation )->Self{
        Self { bitsize:BitSize::new_clamped(bitsize), orientation }
    }
}

impl PhysicalComponent for Adder{

    
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(func::Adder::new(self.bitsize.get()).into())
    }

    fn component_name(&self) -> &'static str {
        "Adder"
    }
    /// Init bounds, identical to most gates
    fn init_bounds(&self, ctx: super::PhysicalInitContext<'_>) -> super::RelativeComponentBounds {
        // The origin is at the output port, which is at (4,2) in absolute coordinates.
        let bounds = [(-4, -2), (0, 2)];
        
        let ports = vec![(-4,-1),(-4,1),(-2,-2), (-2,2), (0,0)];
        
        RelativeComponentBounds { bounds, ports }
            .orient(self.orientation, Default::default())
    }
}

/// subtractor with carryin and input a,b
 #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Subtractor{
    bitsize: BitSize,
    orientation: Orientation,
}
/// adder construcor
impl Subtractor{
    /// constructor
    pub fn new(bitsize:u8, orientation: Orientation )->Self{
        Self { bitsize:BitSize::new_clamped(bitsize), orientation }
    }
}

impl PhysicalComponent for Subtractor{

    
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(func::Subtractor::new(self.bitsize.get()).into())
    }

    fn component_name(&self) -> &'static str {
        "Subtractor"
    }
    /// Init bounds, identical to most gates
    fn init_bounds(&self, ctx: super::PhysicalInitContext<'_>) -> super::RelativeComponentBounds {
        // The origin is at the output port, which is at (4,2) in absolute coordinates.
        let bounds = [(-4, -2), (0, 2)];
        
        let ports = vec![(-4,-1),(-4,1),(-2,-2), (-2,2), (0,0)];
        
        RelativeComponentBounds { bounds, ports }
            .orient(self.orientation, Default::default())
    }
}

/// multiplier with carryin and input a,b
 #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Multiplier{
    bitsize: BitSize,
    signedness: SignType,
    orientation: Orientation
}
/// multiplier construcor
impl Multiplier{
    /// constructor
    pub fn new(bitsize:u8, orientation: Orientation, signedness: SignType )->Self{
        Self { bitsize:BitSize::new_clamped(bitsize), orientation, signedness }
    }
}

impl PhysicalComponent for Multiplier{

    
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(func::Multiplier::new(self.bitsize.get(), self.signedness).into())
    }

    fn component_name(&self) -> &'static str {
        "Multiplier"
    }
    /// Init bounds, identical to most gates
    fn init_bounds(&self, ctx: super::PhysicalInitContext<'_>) -> super::RelativeComponentBounds {
        // The origin is at the output port, which is at (4,2) in absolute coordinates.
        let bounds = [(-4, -2), (0, 2)];
        
        let ports = vec![(-4,-1),(-4,1),(-2,-2), (0,0), (-2,2)];
        
        RelativeComponentBounds { bounds, ports }
            .orient(self.orientation, Default::default())
    }
}


/// Adder with carryin and input a,b
 #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Divider{
    bitsize: BitSize,
    signedness: SignType,
    orientation: Orientation
}
/// multiplier construcor
impl Divider{
    /// constructor
    pub fn new(bitsize:u8, orientation: Orientation, signedness: SignType )->Self{
        Self { bitsize:BitSize::new_clamped(bitsize), orientation, signedness }
    }
}

impl PhysicalComponent for Divider{

    
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(func::Divider::new(self.bitsize.get(), self.signedness).into())
    }

    fn component_name(&self) -> &'static str {
        "Divider"
    }
    /// Init bounds, identical to most gates
    fn init_bounds(&self, ctx: super::PhysicalInitContext<'_>) -> super::RelativeComponentBounds {
        // The origin is at the output port, which is at (4,2) in absolute coordinates.
        let bounds = [(-4, -2), (0, 2)];
        
        let ports = vec![(-4,-1),(-4,1),(-2,-2), (0,0),(-2,2)];
        
        RelativeComponentBounds { bounds, ports }
            .orient(self.orientation, Default::default())
    }
}