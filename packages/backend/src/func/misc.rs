use crate::circuit::state::{CircuitState, InnerFunctionState, TriggerState};
use crate::circuit::{CircuitGraphMap, CircuitKey};
use crate::func::{Component, ComponentFn, PortProperties, PortType, PortUpdate, RunContext};

/// A subcircuit component.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Subcircuit {
    key: CircuitKey
}
impl Subcircuit {
    /// Creates a new instance of the NOT gate with specified bitsize.
    pub fn new(key: CircuitKey) -> Self {
        Self { key }
    }
}
impl Component for Subcircuit {
    fn ports(&self, graphs: &CircuitGraphMap) -> Vec<PortProperties> {
        let graph = &graphs[self.key];

        let mut entries: Vec<_> = graph.functions.values()
            .filter_map(|f| match f.func {
                ComponentFn::Input(n)  => Some(PortProperties { ty: PortType::Input,  bitsize: n.get_bitsize() }),
                ComponentFn::Output(n) => Some(PortProperties { ty: PortType::Output, bitsize: n.get_bitsize() }),
                _ => None
            })
            .collect();
        // Fixed order
        entries.sort_by_key(|p| p.ty);

        entries
    }

    fn initialize_inner_state(&self, graphs: &CircuitGraphMap) -> Option<InnerFunctionState> {
        Some(InnerFunctionState::Subcircuit(CircuitState::init_from_graph(graphs, self.key)))
    }

    fn run_inner(&self, ctx: RunContext<'_>) -> Vec<PortUpdate> {
        // FIXME: This assumes that the order of inputs doesn't change between run.
        let Some(InnerFunctionState::Subcircuit(st)) = ctx.inner_state else {
            unreachable!("Subcircuit's inner state was unexpectedly not a CircuitState");
        };

        let mut i = 0;
        for ((fk, fst), &value) in st.functions.iter_mut()
            .filter(|&(k, _)| matches!(&ctx.graphs[self.key][k].func, ComponentFn::Input(_)))
            .zip(ctx.new_ports)
        {
            // FIXME: This is equivalent to [`Circuit::replace_port`].
            let result = fst.replace_port(0, value);
            if let Some(wire) = ctx.graphs[self.key].functions[fk].links[0] {
                st.transient.triggers.insert(wire, TriggerState { recalculate: true });
            }
            assert!(result.is_ok(), "Port update have the correct bitsize");

            i += 1;
        }

        st.propagate(ctx.graphs, self.key);
        st.functions.iter()
            .filter(|&(k, _)| matches!(&ctx.graphs[self.key][k].func, ComponentFn::Output(_)))
            .map(|(_, f)| f.get_port(0))
            .zip(i..)
            .map(|(value, index)| PortUpdate { index, value })
            .collect()
    }
}