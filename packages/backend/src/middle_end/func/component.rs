use crate::engine::ComponentFn;
use crate::engine::func::enum_dispatch;
use crate::middle_end::func::{self, PhysicalInitContext, RelativeComponentBounds};

/// A component that can be added in a [middle-end circuit](`crate::middle_end::MiddleRepr`).
pub trait PhysicalComponent {
    /// Initializes the component which represents the engine logic of this physical component,
    /// based on the properties of the physical component.
    /// 
    /// This can be `None` if this component has no engine logic.
    fn init_engine(&self) -> Option<ComponentFn>;

    /// The name of the component.
    fn component_name(&self) -> &'static str;

    /// Initializes the bounds of the physical component,
    /// based on its properties.
    /// 
    /// The bounds are defined as:
    ///   - The area encompassed of the component
    ///     (defined by the top-leftmost point and the bottom-rightmost point)
    ///   - The position of the ports
    /// 
    /// These components are relative to the origin (0, 0),
    /// meaning that when placed, the locations are relative
    /// to the point the component is placed.
    fn init_bounds(&self, ctx: PhysicalInitContext<'_>) -> RelativeComponentBounds;
}

macro_rules! component_enum {
    ($(#[$m:meta])* $vis:vis enum $Enum:ident { $($Variant:ident),*$(,)? }) => {
        enum_dispatch! { $(#[$m])* $vis enum $Enum { $($Variant),* } }
        
        // Implement PhysicalComponent for this type
        impl PhysicalComponent for $Enum {
            fn init_engine(&self) -> Option<ComponentFn> {
                match self { $(Self::$Variant(c) => PhysicalComponent::init_engine(c)),* }
            }
            fn component_name(&self) -> &'static str {
                match self { $(Self::$Variant(c) => PhysicalComponent::component_name(c)),* }
            }
            fn init_bounds(&self, ctx: PhysicalInitContext<'_>) -> RelativeComponentBounds {
                match self { $(Self::$Variant(c) => PhysicalComponent::init_bounds(c, ctx)),* }
            }
        }

        #[cfg(feature="serde")]
        mod pcom_serde {
            use std::collections::HashMap;

            use crate::engine::CircuitKey;
            use crate::middle_end::MiddleRepr;
            use crate::middle_end::func::{self, PhysicalComponentEnum, PhysicalComponentKind};
            use crate::middle_end::serialize::{DeserializeWithCtx, SerializeWithCtx};

            impl SerializeWithCtx<MiddleRepr> for PhysicalComponentEnum {
                fn serialize_with_ctx<S>(&self, ctx: &MiddleRepr, serializer: S) -> Result<S::Ok, S::Error>
                    where S: serde::Serializer
                {
                    // Equivalent to an untagged serialization
                    match self {
                        $(Self::$Variant(c) => SerializeWithCtx::serialize_with_ctx(c, ctx, serializer)),*
                    }
                }
            }

            pub struct PComDeserCtx<'a> {
                pub kind: PhysicalComponentKind,
                pub circuit_map: &'a HashMap<String, CircuitKey>
            }
            impl<'de> DeserializeWithCtx<'de, PComDeserCtx<'de>> for PhysicalComponentEnum {
                fn deserialize_with_ctx<D>(ctx: PComDeserCtx<'de>, deserializer: D) -> Result<Self, D::Error>
                    where D: serde::Deserializer<'de>
                {
                    match ctx.kind {
                        $(PhysicalComponentKind::$Variant => func::$Variant::deserialize_with_ctx(ctx, deserializer).map(Into::into)),*
                    }
                }
            }
        }
        #[cfg(feature="serde")]
        pub(crate) use pcom_serde::PComDeserCtx;
    }
}

component_enum! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, strum::EnumDiscriminants, strum::IntoStaticStr)]
    #[expect(missing_docs)]
    #[strum_discriminants(
        name(PhysicalComponentKind),
        expect(missing_docs),
        derive(strum::IntoStaticStr),
        cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))
    )]
    pub enum PhysicalComponentEnum {
        // Wiring
        Pin, Constant, Splitter, Power, Ground, Tunnel, Probe,
        // Muxes
        Mux, Demux, Decoder,
        // Misc
        Text, Subcircuit,
        //Gates
        Gate, Not, TriState,
    }
}