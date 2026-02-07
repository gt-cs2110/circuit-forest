use std::collections::HashMap;

/// An identifier for a tunnel. 
/// All tunnels with the same name have the same associated `TunnelSymbol`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct TunnelSymbol(usize);

/// Struct which creates a fixed symbol for each string value.
/// 
/// This struct also maintains a basic reference counter to free unused strings.
#[derive(Default)]
pub struct StringInterner {
    map: HashMap<String, (TunnelSymbol, usize)>,
    sym_counter: usize
}
fn process(s: &str) -> &str {
    s.trim()
}
impl StringInterner {
    /// Gets the symbol for the tunnel (if it is defined).
    pub fn get(&self, s: &str) -> Option<TunnelSymbol> {
        self.map.get(process(s)).map(|&(s, _)| s)
    }

    /// Adds a new reference to the symbol corresponding to this string.
    /// 
    /// If the string does not already have a symbol, a new one will be created (with a new reference).
    /// Otherwise, the same symbol will be returned and increments the reference counter.
    pub fn add_ref(&mut self, s: &str) -> TunnelSymbol {
        let (sym, ref_ctr) = self.map.entry(process(s).to_string()).or_insert_with(|| {
            let sym = TunnelSymbol(self.sym_counter);
            self.sym_counter = self.sym_counter.strict_add(1);
            (sym, 0)
        });
        *ref_ctr += 1;
        *sym
    }

    /// Removes a reference to the symbol corresponding to this string.
    /// 
    /// If the string does not already have a symbol, this returns `None`.
    /// Otherwise, this returns the symbol that the string is associated with and decrements the reference counter.
    /// (If the reference counter becomes 0, the reference is removed from the map
    ///     and the symbol will no longer be used.)
    pub fn del_ref(&mut self, s: &str) -> Option<TunnelSymbol> {
        let k = process(s);
        let (_, ref_ctr) = self.map.get_mut(k)?;

        *ref_ctr -= 1;
        let sym = match *ref_ctr {
            0 => self.map.remove(k).unwrap().0,
            _ => self.map[k].0
        };
        
        Some(sym)
    }
}
impl std::fmt::Debug for StringInterner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.map.fmt(f)
    }
}