/// A wrapper around a `&SlotMap` or `&SecondaryMap`
/// to make it debug format like a map.
pub(crate) struct DebugMap<M>(pub M);
impl<K, V, M> std::fmt::Debug for DebugMap<M>
where
    K: std::fmt::Debug,
    V: std::fmt::Debug,
    M: IntoIterator<Item = (K, V)> + Copy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.0).finish()
    }
}
