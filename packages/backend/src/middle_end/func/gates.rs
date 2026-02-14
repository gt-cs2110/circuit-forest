use crate::engine::func;
use crate::middle_end::CoordDelta;
use crate::middle_end::func::{AbsoluteComponentBounds, PhysicalComponent, PhysicalInitContext, RelativeComponentBounds};

/// A macro to define gate components for AND, OR, XOR, NAND, NOR, and XNOR gates which all have same structure.
macro_rules! gates {
    ($($(#[$m:meta])? $Id:ident),*$(,)?) => {
        $(
            $(#[$m])?
            #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
            /// A gate component with a variable number of inputs.
           pub struct $Id {
                sim: func::$Id
            }
            impl PhysicalComponent for $Id {
                fn engine_component(&self) -> Option<func::ComponentFn> {
                    Some(self.sim.into())
                }

                fn component_name(&self) -> &'static str {
                    stringify!($Id)
                }

                fn bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {

                    //The origin is at the output port, which is at (4,2) in absolute coordinates.
                    let bounds = [(-4,-2),(0,2)];//Bounds of n input componet is always 4x4
                    let inputs = self.sim.n_inputs() as i32;
                    //generate input ports
                    let ports:Vec<CoordDelta> = (0..inputs).map(|i| (-4,2*i-(inputs-1))).chain(std::iter::once((0,0))).collect();
                    RelativeComponentBounds::from_bounds(bounds, ports)
                }


            }

        )*
    }
}
gates! {
    And,Or,Xor,Nand,Nor,Xnor,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
/// A NOT gate component.
pub struct Not {
    sim: func::Not,
}

impl PhysicalComponent for Not {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) -> &'static str {
        "Not"
    }

    fn bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        AbsoluteComponentBounds::from_bounds([(0, 0), (3, 2)], [(0, 1), (3, 1)]).into_relative((3,1))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
///
pub struct TriState {
    sim: func::TriState,
}

impl PhysicalComponent for TriState {
    fn engine_component(&self) -> Option<func::ComponentFn> {
        Some(self.sim.into())
    }

    fn component_name(&self) -> &'static str {
        "TriState"
    }

    fn bounds(&self, _: PhysicalInitContext<'_>) -> RelativeComponentBounds {
      
        AbsoluteComponentBounds::from_bounds([(0, 0), (2, 2)], [(0, 1), (1,2),(2,1)]).into_relative((2,1))
    }
}