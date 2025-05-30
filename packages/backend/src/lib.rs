
use bitarray::BitArray;
use node::{Component, Node, NodeType};
use petgraph::csr::DefaultIx;
use petgraph::graph::NodeIndex;
use petgraph::visit::{EdgeRef, IntoNodeReferences};
use petgraph::{Directed, Graph};

pub mod bitarray;
pub mod node;

type CircuitIndex = DefaultIx;

struct Circuit {
    inputs: Vec<NodeIndex<CircuitIndex>>,
    outputs: Vec<NodeIndex<CircuitIndex>>,
    graph: Graph<Node, Edge, Directed, CircuitIndex>,
}
impl From<Graph<Node, Edge, Directed, CircuitIndex>> for Circuit {
    fn from(graph: Graph<Node, Edge, Directed, CircuitIndex>) -> Self {
        let inputs = graph.node_references()
            .filter(|(_, n)| matches!(n.ty, NodeType::Input(_)))
            .map(|(id, _)| id)
            .collect();
        let outputs = graph.node_references()
            .filter(|(_, n)| matches!(n.ty, NodeType::Output))
            .map(|(id, _)| id)
            .collect();
        
        Self { inputs, outputs, graph }
    }
}
impl Circuit {
    fn set_inputs(&mut self, values: Vec<BitArray>) {
        for (&i, val) in std::iter::zip(&self.inputs, values) {
            let Node { ty: NodeType::Input(inp), .. } = &mut self.graph[i] else { unreachable!() };
            *inp = val;
        }
    }
    fn get_outputs(&self) -> Vec<BitArray> {
        self.outputs.iter()
            .map(|&n| {
                let node = &self.graph[n];
                if matches!(node.ty, NodeType::Output) {
                    if let Some([inp]) = node.get_inputs() {
                        return inp.clone();
                    }
                }

                unreachable!()
            })
            .collect()
    }
    fn run(&mut self) {
        for i in 0..self.outputs.len() {
            let out_ix = self.outputs[i];
            let [output] = <[_; 1]>::try_from(self.resolve_inputs(out_ix)).unwrap();

            let node = &mut self.graph[out_ix];
            if matches!(node.ty, NodeType::Output) {
                node.invalidate();
                node.run(&[output]);
            }
        }
    }
    fn resolve_inputs(&mut self, idx: NodeIndex<CircuitIndex>) -> Vec<BitArray> {
        let mut ports: Vec<_> = self.graph.edges_directed(idx, petgraph::Direction::Incoming)
            .map(|e| (e.source(), e.weight().input, e.weight().output))
            .collect();
        ports.sort_by_key(|&(_, _, out)| out);

        ports.into_iter()
            .map(|(n, p, _)| self.resolve_outputs(n).remove(p))
            .collect()
    }
    fn resolve_outputs(&mut self, idx: NodeIndex<CircuitIndex>) -> Vec<BitArray> {
        let inputs = self.resolve_inputs(idx);
        self.graph[idx].ty.run(&inputs)
    }
}
struct Edge {
    input: usize,
    output: usize
}
// struct Refresher {
//     update: VecDeque<Update>
// }

// trait Readable {
//     type Item;
//     fn get(&self) -> Self::Item;
// }
// trait Writable: Readable {
//     fn set(&self, val: Self::Item);
// }
// pub struct Signal<T> {
//     value: Option<T>,
//     invalidator: Arc<AtomicBool>,
//     dependents: Vec<Weak<AtomicBool>>
// }
// impl<T> Signal<T> {
//     pub fn new() -> Self {
//         Self {
//             value: None,
//             invalidator: Arc::default(),
//             dependents: vec![]
//         }
//     }
//     pub fn get(&self) -> Option<&T> {
//         self.value.as_ref()
//             .filter(|_| self.invalidator.load(Ordering::Acquire))
//     }
//     pub fn set(&mut self, val: T) {
//         self.dependents.retain_mut(|w| match w.upgrade() {
//             Some(inval) => {
//                 inval.store(true, Ordering::Release);
//                 true
//             },
//             None => false,
//         });
//         self.value.replace(val);
//     }
//     pub fn subscribe_to<U>(&self, signal: &mut Signal<U>) {
//         signal.dependents.push(Arc::downgrade(&self.invalidator));
//     }
// }
// type SignalLock<T> = Arc<RwLock<Signal<T>>>; 
// pub struct Ref<T>(SignalLock<T>);
// impl<T> Ref<T> {
//     pub fn new(value: T) -> Self {
//         let mut signal = Signal::new();
//         signal.set(value);
//         Self(Arc::new(RwLock::new(signal)))
//     }
// }
// impl<T: Copy> Readable for Ref<T> {
//     type Item = T;

//     fn get(&self) -> Self::Item {
//         *self.0.read().unwrap().get().unwrap()
//     }
// }
// impl<T: Copy> Writable for Ref<T> {
//     fn set(&self, val: Self::Item) {
//         self.0.write().unwrap().set(val);
//     }
// }
// pub struct Computed<T> {
//     signal: SignalLock<T>,
//     recomputer: Box<dyn Fn() -> T>
// }
// impl<T> Computed<T> {
//     pub fn new<const N: usize>(data: [Arc<dyn Readable>; N])
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut graph = Graph::new();
        let a = 0x9A3B2174_94093211;
        let b = 0x19182934_19AFFC94;
        let nodes = [
            graph.add_node(Node::from(NodeType::Input(BitArray::from_u64(a)))),
            graph.add_node(Node::from(NodeType::Input(BitArray::from_u64(b)))),
            graph.add_node(Node::from(NodeType::Xor)),
            graph.add_node(Node::from(NodeType::Output)),
        ];
        graph.add_edge(nodes[0], nodes[2], Edge { input: 0, output: 0 });
        graph.add_edge(nodes[1], nodes[2], Edge { input: 0, output: 1 });
        graph.add_edge(nodes[2], nodes[3], Edge { input: 0, output: 0 });

        let mut circuit = Circuit::from(graph);
        circuit.run();

        let left = a ^ b;
        let right = circuit.get_outputs()[0].to_u64();
        assert_eq!(left, right, "0x{left:16X} != 0x{right:16X}");
    }

    #[test]
    fn dual() {
        let mut graph = Graph::new();
        let a = 0x9A3B2174_94093211;
        let b = 0x19182934_19AFFC94;
        let c = 0x92821734_182A9A9A;
        let d = 0xA8293129_FC03919D;
        let nodes = [
            graph.add_node(Node::from(NodeType::Input(BitArray::from_u64(a)))),
            graph.add_node(Node::from(NodeType::Input(BitArray::from_u64(b)))),
            graph.add_node(Node::from(NodeType::Input(BitArray::from_u64(c)))),
            graph.add_node(Node::from(NodeType::Input(BitArray::from_u64(d)))),
            graph.add_node(Node::from(NodeType::Xor)),
            graph.add_node(Node::from(NodeType::Xor)),
            graph.add_node(Node::from(NodeType::Xor)),
            graph.add_node(Node::from(NodeType::Output)),
        ];
        graph.add_edge(nodes[0], nodes[4], Edge { input: 0, output: 0 });
        graph.add_edge(nodes[1], nodes[4], Edge { input: 0, output: 1 });
        graph.add_edge(nodes[2], nodes[5], Edge { input: 0, output: 0 });
        graph.add_edge(nodes[3], nodes[5], Edge { input: 0, output: 1 });
        graph.add_edge(nodes[4], nodes[6], Edge { input: 0, output: 0 });
        graph.add_edge(nodes[5], nodes[6], Edge { input: 0, output: 1 });
        graph.add_edge(nodes[6], nodes[7], Edge { input: 0, output: 0 });

        let mut circuit = Circuit::from(graph);
        circuit.run();

        let left = a ^ b ^ c ^ d;
        let right = circuit.get_outputs()[0].to_u64();
        assert_eq!(left, right, "0x{left:16X} != 0x{right:16X}");
    }
}
