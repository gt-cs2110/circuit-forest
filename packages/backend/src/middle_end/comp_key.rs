use slotmap::{SecondaryMap, SlotMap, new_key_type};

use crate::engine::FunctionKey;

new_key_type! {
    /// Key for UI components that are not linked to an engine function.
    pub struct UIKey;
}

/// Key for all middle-end components.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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

pub struct ComponentMap<V> {
    pub(crate) func: SecondaryMap<FunctionKey, V>,
    pub(crate) ui: SlotMap<UIKey, V>
}
impl<V> ComponentMap<V> {
    pub fn contains_key(&self, k: ComponentKey) -> bool {
        match k {
            ComponentKey::Function(gate) => self.func.contains_key(gate),
            ComponentKey::UI(ui_key) => self.ui.contains_key(ui_key),
        }
    }
    pub fn get(&self, k: ComponentKey) -> Option<&V> {
        match k {
            ComponentKey::Function(gate) => self.func.get(gate),
            ComponentKey::UI(ui_key) => self.ui.get(ui_key),
        }
    }
    pub fn remove(&mut self, k: ComponentKey) -> Option<V> {
        match k {
            ComponentKey::Function(gate) => self.func.remove(gate),
            ComponentKey::UI(ui_key) => self.ui.remove(ui_key),
        }
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        std::iter::chain(self.func.values(), self.ui.values())
    }
    pub fn iter(&self) -> impl Iterator<Item = (ComponentKey, &V)> {
        std::iter::chain(
            self.func.iter().map(|(k, v)| (k.into(), v)),
            self.ui.iter().map(|(k, v)| (k.into(), v)),
        )
    }
}
impl<V> std::ops::Index<ComponentKey> for ComponentMap<V> {
    type Output = V;

    fn index(&self, index: ComponentKey) -> &Self::Output {
        match index {
            ComponentKey::Function(k) => &self[k],
            ComponentKey::UI(k) => &self[k],
        }
    }
}
impl<V> std::ops::Index<FunctionKey> for ComponentMap<V> {
    type Output = V;

    fn index(&self, index: FunctionKey) -> &Self::Output {
        &self.func[index]
    }
}
impl<V> std::ops::Index<UIKey> for ComponentMap<V> {
    type Output = V;

    fn index(&self, index: UIKey) -> &Self::Output {
        &self.ui[index]
    }
}
impl<V> Default for ComponentMap<V> {
    fn default() -> Self {
        Self { func: Default::default(), ui: Default::default() }
    }
}
impl<V: std::fmt::Debug> std::fmt::Debug for ComponentMap<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.iter())
            .finish()
    }
}