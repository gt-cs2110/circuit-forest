#![warn(missing_docs)]
//! Engine for CircuitSim.
// TODO: Add actual doc comment above

pub mod bitarray;
pub mod node;
pub mod circuit;

#[cfg(test)]
mod tests {
    use crate::bitarray::{bitarr, BitArray, BitState};
    use crate::circuit::{Circuit, ValueIssue};

    use super::*;

    #[test]
    fn simple() {
        let mut circuit = Circuit::new();
        let a = 0x9A3B2174_94093211;
        let b = 0x19182934_19AFFC94;

        // Wires
        let a_in = circuit.add_input(BitArray::from(a));
        let b_in = circuit.add_input(BitArray::from(b));
        let out  = circuit.add_value_node(bitarr![Z; 64]);
        // Gates
        let gate = circuit.add_function_node(node::Xor::new(64, 2));

        circuit.connect_all(gate, &[a_in, b_in, out]);
        circuit.run(&[a_in, b_in]);

        let left = a ^ b;
        let right = u64::try_from(circuit.state().value(out)).unwrap();
        assert_eq!(left, right, "0x{left:016X} != 0x{right:016X}");
    }

    #[test]
    fn dual() {
        let mut circuit = Circuit::new();
        let a = 0x9A3B2174_94093211;
        let b = 0x19182934_19AFFC94;
        let c = 0x92821734_182A9A9A;
        let d = 0xA8293129_FC03919D;

        // Wires
        let a_in = circuit.add_input(BitArray::from(a));
        let b_in = circuit.add_input(BitArray::from(b));
        let c_in = circuit.add_input(BitArray::from(c));
        let d_in = circuit.add_input(BitArray::from(d));
        let ab_mid = circuit.add_value_node(bitarr![Z; 64]);
        let cd_mid = circuit.add_value_node(bitarr![Z; 64]);
        let out = circuit.add_value_node(bitarr![Z; 64]);
        // Gates
        let gates = [
            circuit.add_function_node(node::Xor::new(64, 2)),
            circuit.add_function_node(node::Xor::new(64, 2)),
            circuit.add_function_node(node::Xor::new(64, 2)),
        ];

        circuit.connect_all(gates[0], &[a_in, b_in, ab_mid]);
        circuit.connect_all(gates[1], &[c_in, d_in, cd_mid]);
        circuit.connect_all(gates[2], &[ab_mid, cd_mid, out]);
        circuit.run(&[a_in, b_in, c_in, d_in]);

        let left = a ^ b ^ c ^ d;
        let right = u64::try_from(circuit.state().value(out)).unwrap();
        assert_eq!(left, right, "0x{left:016X} != 0x{right:016X}");
    }

    #[test]
    fn metastable() {
        let mut circuit = Circuit::new();
        let a = 0x98A85409_19182A9F;

        let wires = [
            circuit.add_value_node(bitarr![Z; 64]),
            circuit.add_value_node(bitarr![Z; 64]),
        ];
        let gates = [
            circuit.add_function_node(node::Not::new(64)),
            circuit.add_function_node(node::Not::new(64)),
        ];

        circuit.connect_all(gates[0], &[wires[0], wires[1]]);
        circuit.connect_all(gates[1], &[wires[1], wires[0]]);

        circuit.set(wires[0], BitArray::from(a));
        circuit.run(&[wires[0]]);

        let (l1, r1) = (a, u64::try_from(circuit.state().value(wires[0])).unwrap());
        let (l2, r2) = (!a, u64::try_from(circuit.state().value(wires[1])).unwrap());
        assert_eq!(l1, r1, "0x{l1:016X} != 0x{r1:016X}");
        assert_eq!(l2, r2, "0x{l2:016X} != 0x{r2:016X}");
    }

    #[test]
    fn nand_propagate() {
        let mut circuit = Circuit::new();

        let inp  = circuit.add_input(bitarr![0]);
        let out0 = circuit.add_value_node(bitarr![Z]);
        let out1 = circuit.add_value_node(bitarr![Z]);
        let gates = [
            circuit.add_function_node(node::Nand::new(1, 2)),
            circuit.add_function_node(node::Nand::new(1, 2)),
        ];

        circuit.connect_all(gates[0], &[inp, out1, out0]);
        circuit.connect_all(gates[1], &[inp, out0, out1]);
        circuit.run(&[inp]);

        assert_eq!(0, u64::try_from(circuit.state().value(inp)).unwrap());
        assert_eq!(1, u64::try_from(circuit.state().value(out0)).unwrap());
        assert_eq!(1, u64::try_from(circuit.state().value(out1)).unwrap());
    }

    #[test]
    fn conflict_pass_z() {
        let mut circuit = Circuit::new();

        // Wires
        let lo = circuit.add_input(bitarr![0]);
        let hi = circuit.add_input(bitarr![1]);
        let out = circuit.add_value_node(bitarr![Z]);
        // Gates
        let gates = [
            circuit.add_function_node(node::TriState::new(1)),
            circuit.add_function_node(node::TriState::new(1)),
        ];

        circuit.connect_all(gates[0], &[lo, lo, out]);
        circuit.connect_all(gates[1], &[hi, hi, out]);
        circuit.run(&[lo, hi]);

        assert_eq!(1, u64::try_from(circuit.state().value(out)).unwrap());
    }

    #[test]
    fn conflict_fail() {
        let mut circuit = Circuit::new();

        // Wires
        let lo = circuit.add_input(bitarr![0]);
        let hi = circuit.add_input(bitarr![1]);
        let out = circuit.add_value_node(bitarr![Z]);
        // Gates
        let gates = [
            circuit.add_function_node(node::TriState::new(1)),
            circuit.add_function_node(node::TriState::new(1)),
        ];

        circuit.connect_all(gates[0], &[hi, lo, out]);
        circuit.connect_all(gates[1], &[hi, hi, out]);
        circuit.run(&[lo, hi]);

        assert!(circuit.state().issues(out).contains(&ValueIssue::ShortCircuit), "Node 'out' should short circuit");
    }

    #[test]
    fn delay_conflict() {
        let mut circuit = Circuit::new();
        
        // Wires
        let inp = circuit.add_input(bitarr![0]); 
        let mid = circuit.add_input(bitarr![0]);
        let out = circuit.add_value_node(bitarr![0]);
        // Gates
        let gates = [
            circuit.add_function_node(node::Not::new(1)),
            circuit.add_function_node(node::Not::new(1)),
            circuit.add_function_node(node::Not::new(1)),
        ];

        circuit.connect_all(gates[0], &[inp, mid]);
        circuit.connect_all(gates[1], &[mid, out]);
        circuit.connect_all(gates[2], &[inp, out]);
        circuit.run(&[inp]);

        assert!(circuit.state().issues(out).contains(&ValueIssue::ShortCircuit), "Node 'out' should short circuit");
    }

    #[test]
    fn rs_latch() {
        let mut circuit = Circuit::new();
        let [r, s, q, qp] = [
            circuit.add_input(bitarr![1]), // R
            circuit.add_input(bitarr![1]), // S
            circuit.add_value_node(bitarr![1]), // Q
            circuit.add_value_node(bitarr![0]), // Q'
        ];
        let [rnand, snand] = [
            circuit.add_function_node(node::Nand::new(1, 2)),
            circuit.add_function_node(node::Nand::new(1, 2)),
        ];

        // R = 1, S = 1
        circuit.connect_all(rnand, &[r, q, qp]);
        circuit.connect_all(snand, &[s, qp, q]);
        circuit.run(&[r, s]);

        // In an invalid state right now.
        assert_eq!(circuit.state().value(q), bitarr![X]);
        assert_eq!(circuit.state().value(qp), bitarr![X]);

        // R = 0, S = 1
        circuit.set(r, bitarr![0]);
        circuit.run(&[r]);

        assert_eq!(u64::try_from(circuit.state().value(q)).unwrap(), 0);
        assert_eq!(u64::try_from(circuit.state().value(qp)).unwrap(), 1);

        // R = 1, S = 0
        circuit.set(r, bitarr![1]);
        circuit.set(s, bitarr![0]);
        circuit.run(&[r, s]);

        assert_eq!(u64::try_from(circuit.state().value(q)).unwrap(), 1);
        assert_eq!(u64::try_from(circuit.state().value(qp)).unwrap(), 0);
    }

    #[test]
    fn d_latch() {
        let mut circuit = Circuit::new();
        // external
        let [din, wen, dout, doutp] = [
            circuit.add_input(bitarr![0]),
            circuit.add_input(bitarr![1]),
            circuit.add_value_node(bitarr![Z]),
            circuit.add_value_node(bitarr![Z]),
        ];
        // internal
        let [dinp, r, s] = [
            circuit.add_value_node(bitarr![Z]),
            circuit.add_value_node(bitarr![Z]),
            circuit.add_value_node(bitarr![Z]),
        ];
        // nodes
        let [dnot, dnand, dpnand, rnand, snand] = [
            circuit.add_function_node(node::Not::new(1)),
            circuit.add_function_node(node::Nand::new(1, 2)),
            circuit.add_function_node(node::Nand::new(1, 2)),
            circuit.add_function_node(node::Nand::new(1, 2)),
            circuit.add_function_node(node::Nand::new(1, 2)),
        ];

        circuit.connect_all(dnot, &[din, dinp]);
        circuit.connect_all(dnand, &[din, wen, s]);
        circuit.connect_all(dpnand, &[dinp, wen, r]);
        circuit.connect_all(rnand, &[r, dout, doutp]);
        circuit.connect_all(snand, &[s, doutp, dout]);
        
        circuit.run(&[din, wen]);
        assert_eq!(u64::try_from(circuit.state().value(dout)).unwrap(), 0);
        assert_eq!(u64::try_from(circuit.state().value(doutp)).unwrap(), 1);

        circuit.set(wen, bitarr![0]);
        circuit.run(&[wen]);
        assert_eq!(u64::try_from(circuit.state().value(dout)).unwrap(), 0);
        assert_eq!(u64::try_from(circuit.state().value(doutp)).unwrap(), 1);

        circuit.set(din, bitarr![1]);
        circuit.run(&[din]);
        assert_eq!(u64::try_from(circuit.state().value(dout)).unwrap(), 0);
        assert_eq!(u64::try_from(circuit.state().value(doutp)).unwrap(), 1);

        circuit.set(wen, bitarr![1]);
        circuit.run(&[wen]);
        assert_eq!(u64::try_from(circuit.state().value(dout)).unwrap(), 1);
        assert_eq!(u64::try_from(circuit.state().value(doutp)).unwrap(), 0);
    }
    #[test]
    fn chain() {
        let mut circuit = Circuit::new();
        
        // Wires
        let a_in = circuit.add_input(bitarr![1]);
        let b_in = circuit.add_input(bitarr![0]);
        let c_in = circuit.add_input(bitarr![1]);
        let d_in = circuit.add_input(bitarr![0]);
        let e_in = circuit.add_input(bitarr![1]);
        let ab_mid = circuit.add_value_node(bitarr![Z; 1]);
        let abc_mid = circuit.add_value_node(bitarr![Z; 1]);
        let abcd_mid = circuit.add_value_node(bitarr![Z; 1]);
        let out = circuit.add_value_node(bitarr![Z; 1]);
        // Gates
        let gates = [
            circuit.add_function_node(node::Xor::new(1, 2)),
            circuit.add_function_node(node::Xor::new(1, 2)),
            circuit.add_function_node(node::Xor::new(1, 2)),
            circuit.add_function_node(node::Xor::new(1, 2)),
        ];

        circuit.connect_all(gates[0], &[a_in, b_in, ab_mid]);
        circuit.connect_all(gates[1], &[ab_mid, c_in, abc_mid]);
        circuit.connect_all(gates[2], &[abc_mid, d_in, abcd_mid]);
        circuit.connect_all(gates[3], &[abcd_mid, e_in, out]);
        circuit.run(&[a_in, b_in, c_in, d_in, e_in]);

        assert_eq!(u64::try_from(circuit.state().value(out)).unwrap(), 1);
    }

    #[test]
    fn oscillate() {
        let mut circuit = Circuit::new();

        let a_in = circuit.add_value_node(bitarr![Z]);
        let gate = circuit.add_function_node(node::Not::new(1));
        
        circuit.connect_all(gate, &[a_in, a_in]);
        circuit.set(a_in, bitarr![1]);
        circuit.run(&[a_in]);

        assert!(circuit.state().issues(a_in).contains(&ValueIssue::OscillationDetected), "Node 'in' should oscillate");
    }

    #[test]
    fn splitter() {
        let mut circuit = Circuit::new();
        let nodes: [_; 9] = std::array::from_fn(|i| circuit.add_value_node(match i {
            0 => bitarr![Z; 8],
            _ => bitarr![Z; 1]
        }));
        let [joined_node, split_nodes @ ..] = nodes;
        let splitter = circuit.add_function_node(node::Splitter::new(8));
        circuit.connect_all(splitter, &nodes);

        // joined -> split
        let inputs = [
            BitState::Low,
            BitState::High,
            BitState::Imped,
            BitState::Unk,
            BitState::Low,
            BitState::High,
            BitState::Imped,
            BitState::Unk,
        ];
        let joined = BitArray::from_iter(inputs);
        let split = inputs.map(|st| BitArray::from_iter([st]));
        circuit.set(joined_node, joined);
        circuit.run(&[nodes[0]]);
        assert_eq!(split_nodes.map(|n| circuit.state().value(n)), split);

        // split -> joined
        let inputs = [
            BitState::Unk,
            BitState::Imped,
            BitState::High,
            BitState::Low,
            BitState::Unk,
            BitState::Imped,
            BitState::High,
            BitState::Low,
        ];
        let joined = BitArray::from_iter(inputs);
        let split = inputs.map(|st| BitArray::from_iter([st]));
        for (n, a) in std::iter::zip(split_nodes, split) {
            circuit.set(n, a);
        }
        circuit.run(&split_nodes);
        assert_eq!(circuit.state().value(joined_node), joined);
    }

    #[test]
    fn register() {
        let mut circuit = Circuit::new();
        let inputs = bitarr![1, 0, 1, 0, 1, 0, 1, 0];
        let nodes @ [din, enable, clock, clear, dout] = [
            circuit.add_input(inputs),
            circuit.add_input(bitarr![0]),
            circuit.add_input(bitarr![0]),
            circuit.add_input(bitarr![0]),
            circuit.add_value_node(bitarr![Z; 8]),
        ];
        let reg = circuit.add_function_node(node::Register::new(8));
        circuit.connect_all(reg, &nodes);

        // enable off, clock down
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.state().value(dout), bitarr![0; 8]);
        
        // enable on, clock down
        circuit.set(enable, bitarr![1]);
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.state().value(dout), bitarr![0; 8]);
        
        // enable on, clock up
        circuit.set(clock, bitarr![1]);
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.state().value(dout), inputs);
        
        // enable on, clock down
        circuit.set(clock, bitarr![0]);
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.state().value(dout), inputs);
        
        // update din
        let neg_inputs = !inputs;
        circuit.set(din, BitArray::from_iter(neg_inputs));
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.state().value(dout), inputs);

        for _ in 0..3 {
            // enable off, clock down
            circuit.set(enable, bitarr![0]);
            circuit.run(&[din, enable, clock, clear]);
            assert_eq!(circuit.state().value(dout), inputs);
    
            // enable off, clock up
            circuit.set(clock, bitarr![1]);
            circuit.run(&[din, enable, clock, clear]);
            assert_eq!(circuit.state().value(dout), inputs);
        }

        // enable off, clock down
        circuit.set(clock, bitarr![0]);
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.state().value(dout), inputs);

        // enable on, clock up
        circuit.set(enable, bitarr![1]);
        circuit.set(clock, bitarr![1]);
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.state().value(dout), neg_inputs);

        // reset
        circuit.set(clear, bitarr![1]);
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.state().value(dout), bitarr![0; 8]);
        
        // reset, clock down
        circuit.set(clock, bitarr![0]);
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.state().value(dout), bitarr![0; 8]);
        
        // reset, clock up
        circuit.set(clock, bitarr![1]);
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.state().value(dout), bitarr![0; 8]);
    }
}
