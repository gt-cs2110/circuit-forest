use slotmap::new_key_type;

use crate::circuit::graph::FunctionKey;

new_key_type! {
    /// Key for UI components that are not linked to an engine function.
    pub struct UIKey;
}
/// Key for all middle-end components.
pub enum ComponentKey {
    /// Component associated with engine function node.
    Function(FunctionKey),
    /// Middle-end only component (e.g., tunnel, probe).
    UI(UIKey)
}
impl From<FunctionKey> for ComponentKey {
    fn from(value: FunctionKey) -> Self {
        Self::Function(value)
    }
}
impl From<UIKey> for ComponentKey {
    fn from(value: UIKey) -> Self {
        Self::UI(value)
    }
}