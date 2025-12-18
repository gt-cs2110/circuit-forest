use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct TunnelSymbol(usize);
#[derive(Default)]
pub struct StringInterner {
    map: HashMap<String, TunnelSymbol>,
    counter: usize
}
impl StringInterner {
    pub fn intern(&mut self, k: &str) -> TunnelSymbol {
        *self.map.entry(k.to_string()).or_insert_with(|| {
            let sym = TunnelSymbol(self.counter);
            self.counter = self.counter.strict_add(1);
            sym
        })
    }
    pub fn remove(&mut self, k: &str) -> bool {
        self.map.remove(k).is_some()
    }
}
impl std::fmt::Debug for StringInterner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.map.fmt(f)
    }
}