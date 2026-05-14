use crate::engine::{CircuitKey, func};
use crate::middle_end::func::{PhysicalComponent, PhysicalInitContext, RelativeComponentBounds};

// FIXME: This deser def'n no longer works
/// A subcircuit component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Subcircuit {
    key: CircuitKey
}
impl PhysicalComponent for Subcircuit {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        Some(func::Subcircuit::new(self.key).into())
    }

    fn component_name(&self) ->  &'static str {
        "Subcircuit"
    }

    fn init_bounds(&self, ctx: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        todo!()
    }
}
#[cfg(feature="serde")]
mod subcircuit_serde {
    use serde::{Deserialize, Serialize};
    
    use crate::middle_end::MiddleRepr;
    use crate::middle_end::func::Subcircuit;
    use crate::middle_end::func::pcom_serde::PComDeserCtx;
    use crate::middle_end::serialize::{DeserializeWithCtx, SerializeWithCtx};

    #[derive(Serialize, Deserialize)]
    struct SubcircuitSer<'a> {
        subcircuit: &'a str
    }

    impl SerializeWithCtx<MiddleRepr> for Subcircuit {
        fn serialize_with_ctx<S>(&self, ctx: &MiddleRepr, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer
        {
            let subcircuit = &ctx.physical[self.key].name;
            SubcircuitSer { subcircuit }.serialize(serializer)
        }
    }

    impl<'de> DeserializeWithCtx<'de, PComDeserCtx<'de>> for Subcircuit {
        fn deserialize_with_ctx<D>(ctx: PComDeserCtx<'de>, deserializer: D) -> Result<Self, D::Error>
            where D: serde::Deserializer<'de>
        {
            let SubcircuitSer { subcircuit } = SubcircuitSer::deserialize(deserializer)?;
            ctx.circuit_map.get(subcircuit)
                .map(|&key| Subcircuit { key })
                .ok_or_else(|| serde::de::Error::custom(format!("no circuit exists with name {subcircuit}")))
        }
    }
}

/// Text.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Text;
impl PhysicalComponent for Text {
    fn init_engine(&self) -> Option<func::ComponentFn> {
        None
    }

    fn component_name(&self) ->  &'static str {
        "Text"
    }

    fn init_bounds(&self, ctx: PhysicalInitContext<'_>) -> RelativeComponentBounds {
        todo!()
    }
}