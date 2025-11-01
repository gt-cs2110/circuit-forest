#![warn(missing_docs)]
//! Engine for CircuitSim.
// TODO: Add actual doc comment above

pub mod bitarray;
pub mod func;
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
        let (_, a_in) = circuit.add_input(BitArray::from(a));
        let (_, b_in) = circuit.add_input(BitArray::from(b));
        let (out_g, out) = circuit.add_output(64);
        // Gates
        let gate = circuit.add_function_node(func::Xor::new(64, 2));

        circuit.connect_all(gate, &[a_in, b_in, out]);
        circuit.run(&[a_in, b_in]);

        let left = circuit.get_output(out_g);
        let right = BitArray::from(a ^ b);
        assert_eq!(left, right);
    }

    #[test]
    fn dual() {
        let mut circuit = Circuit::new();
        let a = 0x9A3B2174_94093211;
        let b = 0x19182934_19AFFC94;
        let c = 0x92821734_182A9A9A;
        let d = 0xA8293129_FC03919D;

        // Wires
        let (_, a_in) = circuit.add_input(BitArray::from(a));
        let (_, b_in) = circuit.add_input(BitArray::from(b));
        let (_, c_in) = circuit.add_input(BitArray::from(c));
        let (_, d_in) = circuit.add_input(BitArray::from(d));
        let ab_mid = circuit.add_value_node();
        let cd_mid = circuit.add_value_node();
        let (out_g, out) = circuit.add_output(64);
        // Gates
        let gates = [
            circuit.add_function_node(func::Xor::new(64, 2)),
            circuit.add_function_node(func::Xor::new(64, 2)),
            circuit.add_function_node(func::Xor::new(64, 2)),
        ];

        circuit.connect_all(gates[0], &[a_in, b_in, ab_mid]);
        circuit.connect_all(gates[1], &[c_in, d_in, cd_mid]);
        circuit.connect_all(gates[2], &[ab_mid, cd_mid, out]);
        circuit.run(&[a_in, b_in, c_in, d_in]);

        let left = circuit.get_output(out_g);
        let right = BitArray::from(a ^ b ^ c ^ d);
        assert_eq!(left, right);
    }

    #[test]
    fn simple_io() {
        let mut circuit = Circuit::new();
        let inp = circuit.add_function_node(func::Input::new(1));
        let out = circuit.add_function_node(func::Output::new(1));
        let wire = circuit.add_value_node();

        circuit.connect_all(inp, &[wire]);
        circuit.connect_all(out, &[wire]);

        circuit.run(&[wire]);

        assert_eq!(circuit.state().value(wire), bitarr![Z]);
        assert_eq!(circuit.get_output(out), bitarr![Z]);

        assert!(circuit.set_input(inp, bitarr![0]).is_ok());
        assert_eq!(circuit.state().value(wire), bitarr![0]);
        assert_eq!(circuit.get_output(out), bitarr![0]);

        assert!(circuit.set_input(inp, bitarr![1]).is_ok());
        assert_eq!(circuit.state().value(wire), bitarr![1]);
        assert_eq!(circuit.get_output(out), bitarr![1]);
    }

    #[test]
    fn metastable() {
        let mut circuit = Circuit::new();
        let a = 0x98A85409_19182A9F;

        let wires = [
            circuit.add_value_node(),
            circuit.add_value_node(),
        ];
        let gates = [
            circuit.add_function_node(func::Not::new(64)),
            circuit.add_function_node(func::Not::new(64)),
        ];

        circuit.connect_all(gates[0], &[wires[0], wires[1]]);
        circuit.connect_all(gates[1], &[wires[1], wires[0]]);

        assert!(circuit.replace(wires[0], BitArray::from(a)).is_ok());
        circuit.run(&[wires[0]]);

        let (l1, r1) = (circuit.state().value(wires[0]), BitArray::from(a));
        let (l2, r2) = (circuit.state().value(wires[1]), BitArray::from(!a));
        assert_eq!(l1, r1);
        assert_eq!(l2, r2);
    }

    #[test]
    fn nand_propagate() {
        let mut circuit = Circuit::new();

        let (_, inp) = circuit.add_input(bitarr![0]);
        let (out0_g, out0) = circuit.add_output(1);
        let (out1_g, out1) = circuit.add_output(1);
        let gates = [
            circuit.add_function_node(func::Nand::new(1, 2)),
            circuit.add_function_node(func::Nand::new(1, 2)),
        ];

        circuit.connect_all(gates[0], &[inp, out1, out0]);
        circuit.connect_all(gates[1], &[inp, out0, out1]);
        circuit.run(&[inp]);

        assert_eq!(circuit.state().value(inp), bitarr![0]);
        assert_eq!(circuit.get_output(out0_g), bitarr![1]);
        assert_eq!(circuit.get_output(out1_g), bitarr![1]);
    }

    #[test]
    fn conflict_pass_z() {
        let mut circuit = Circuit::new();

        // Wires
        let (_, lo) = circuit.add_input(bitarr![0]);
        let (_, hi) = circuit.add_input(bitarr![1]);
        let (out_g, out) = circuit.add_output(1);
        // Gates
        let gates = [
            circuit.add_function_node(func::TriState::new(1)),
            circuit.add_function_node(func::TriState::new(1)),
        ];

        circuit.connect_all(gates[0], &[lo, lo, out]);
        circuit.connect_all(gates[1], &[hi, hi, out]);
        circuit.run(&[lo, hi]);

        assert_eq!(circuit.get_output(out_g), bitarr![1]);
    }

    #[test]
    fn conflict_fail() {
        let mut circuit = Circuit::new();

        // Wires
        let (_, lo) = circuit.add_input(bitarr![0]);
        let (_, hi) = circuit.add_input(bitarr![1]);
        let (_, out) = circuit.add_output(1);
        // Gates
        let gates = [
            circuit.add_function_node(func::TriState::new(1)),
            circuit.add_function_node(func::TriState::new(1)),
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
        let (_, inp) = circuit.add_input(bitarr![0]); 
        let (_, mid) = circuit.add_input(bitarr![0]);
        let (_, out) = circuit.add_output(1);
        // Gates
        let gates = [
            circuit.add_function_node(func::Not::new(1)),
            circuit.add_function_node(func::Not::new(1)),
            circuit.add_function_node(func::Not::new(1)),
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
        let [(r_g, r), (s_g, s), (q_g, q), (qp_g, qp)] = [
            circuit.add_input(bitarr![1]), // R
            circuit.add_input(bitarr![1]), // S
            circuit.add_output(1), // Q
            circuit.add_output(1), // Q'
        ];
        let [rnand, snand] = [
            circuit.add_function_node(func::Nand::new(1, 2)),
            circuit.add_function_node(func::Nand::new(1, 2)),
        ];

        // R = 1, S = 1
        circuit.connect_all(rnand, &[r, q, qp]);
        circuit.connect_all(snand, &[s, qp, q]);
        circuit.run(&[r, s]);

        // In an invalid state right now.
        assert_eq!(circuit.get_output(q_g), bitarr![X]);
        assert_eq!(circuit.get_output(qp_g), bitarr![X]);

        // R = 0, S = 1
        assert!(circuit.set_input(r_g, bitarr![0]).is_ok());
        println!("{}", circuit.state().value(r));
        println!("{}", circuit.state().value(s));
        circuit.run(&[r]);

        assert_eq!(circuit.get_output(q_g), bitarr![0]);
        assert_eq!(circuit.get_output(qp_g), bitarr![1]);

        // R = 1, S = 0
        assert!(circuit.set_input(r_g, bitarr![1]).is_ok());
        assert!(circuit.set_input(s_g, bitarr![0]).is_ok());
        circuit.run(&[r, s]);

        assert_eq!(circuit.get_output(q_g), bitarr![1]);
        assert_eq!(circuit.get_output(qp_g), bitarr![0]);
    }

    #[test]
    fn d_latch() {
        let mut circuit = Circuit::new();
        // external
        let [(din_g, din), (wen_g, wen), (dout_g, dout), (doutp_g, doutp)] = [
            circuit.add_input(bitarr![0]),
            circuit.add_input(bitarr![1]),
            circuit.add_output(1),
            circuit.add_output(1),
        ];
        // internal
        let [dinp, r, s] = [
            circuit.add_value_node(),
            circuit.add_value_node(),
            circuit.add_value_node(),
        ];
        // nodes
        let [dnot, dnand, dpnand, rnand, snand] = [
            circuit.add_function_node(func::Not::new(1)),
            circuit.add_function_node(func::Nand::new(1, 2)),
            circuit.add_function_node(func::Nand::new(1, 2)),
            circuit.add_function_node(func::Nand::new(1, 2)),
            circuit.add_function_node(func::Nand::new(1, 2)),
        ];

        circuit.connect_all(dnot, &[din, dinp]);
        circuit.connect_all(dnand, &[din, wen, s]);
        circuit.connect_all(dpnand, &[dinp, wen, r]);
        circuit.connect_all(rnand, &[r, dout, doutp]);
        circuit.connect_all(snand, &[s, doutp, dout]);
        
        circuit.run(&[din, wen]);
        assert_eq!(circuit.get_output(dout_g), bitarr![0]);
        assert_eq!(circuit.get_output(doutp_g), bitarr![1]);

        assert!(circuit.set_input(wen_g, bitarr![0]).is_ok());
        circuit.run(&[wen]);
        assert_eq!(circuit.get_output(dout_g), bitarr![0]);
        assert_eq!(circuit.get_output(doutp_g), bitarr![1]);

        assert!(circuit.set_input(din_g, bitarr![1]).is_ok());
        circuit.run(&[din]);
        assert_eq!(circuit.get_output(dout_g), bitarr![0]);
        assert_eq!(circuit.get_output(doutp_g), bitarr![1]);

        assert!(circuit.set_input(wen_g, bitarr![1]).is_ok());
        circuit.run(&[wen]);
        assert_eq!(circuit.get_output(dout_g), bitarr![1]);
        assert_eq!(circuit.get_output(doutp_g), bitarr![0]);
    }
    #[test]
    fn chain() {
        let mut circuit = Circuit::new();
        
        // Wires
        let (_, a_in) = circuit.add_input(bitarr![1]);
        let (_, b_in) = circuit.add_input(bitarr![0]);
        let (_, c_in) = circuit.add_input(bitarr![1]);
        let (_, d_in) = circuit.add_input(bitarr![0]);
        let (_, e_in) = circuit.add_input(bitarr![1]);
        let ab_mid = circuit.add_value_node();
        let abc_mid = circuit.add_value_node();
        let abcd_mid = circuit.add_value_node();
        let (out_g, out) = circuit.add_output(1);
        // Gates
        let gates = [
            circuit.add_function_node(func::Xor::new(1, 2)),
            circuit.add_function_node(func::Xor::new(1, 2)),
            circuit.add_function_node(func::Xor::new(1, 2)),
            circuit.add_function_node(func::Xor::new(1, 2)),
        ];

        circuit.connect_all(gates[0], &[a_in, b_in, ab_mid]);
        circuit.connect_all(gates[1], &[ab_mid, c_in, abc_mid]);
        circuit.connect_all(gates[2], &[abc_mid, d_in, abcd_mid]);
        circuit.connect_all(gates[3], &[abcd_mid, e_in, out]);
        circuit.run(&[a_in, b_in, c_in, d_in, e_in]);

        assert_eq!(circuit.get_output(out_g), bitarr![1]);
    }

    #[test]
    fn oscillate() {
        let mut circuit = Circuit::new();

        let a_in = circuit.add_value_node();
        let gate = circuit.add_function_node(func::Not::new(1));
        
        circuit.connect_all(gate, &[a_in, a_in]);
        assert!(circuit.replace(a_in, bitarr![1]).is_ok());
        circuit.run(&[a_in]);

        assert!(circuit.state().issues(a_in).contains(&ValueIssue::OscillationDetected), "Node 'in' should oscillate");
    }

    #[test]
    fn splitter() {
        let mut circuit = Circuit::new();
        let nodes: [_; 9] = std::array::from_fn(|_| circuit.add_value_node());
        let [joined_node, split_nodes @ ..] = nodes;
        let splitter = circuit.add_function_node(func::Splitter::new(8));
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
        assert!(circuit.replace(joined_node, joined).is_ok());
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
            assert!(circuit.replace(n, a).is_ok());
        }
        circuit.run(&split_nodes);
        assert_eq!(circuit.state().value(joined_node), joined);
    }

    #[test]
    fn register() {
        let mut circuit = Circuit::new();
        let inputs = bitarr![1, 0, 1, 0, 1, 0, 1, 0];
        let nodes @ [(din_g, din), (enable_g, enable), (clock_g, clock), (clear_g, clear), (dout_g, _)] = [
            circuit.add_input(inputs),
            circuit.add_input(bitarr![0]),
            circuit.add_input(bitarr![0]),
            circuit.add_input(bitarr![0]),
            circuit.add_output(8),
        ];
        let nodes = nodes.map(|(_, n)| n);

        let reg = circuit.add_function_node(func::Register::new(8));
        circuit.connect_all(reg, &nodes);

        // enable off, clock down
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.get_output(dout_g), bitarr![0; 8]);
        
        // enable on, clock down
        assert!(circuit.set_input(enable_g, bitarr![1]).is_ok());
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.get_output(dout_g), bitarr![0; 8]);
        
        // enable on, clock up
        assert!(circuit.set_input(clock_g, bitarr![1]).is_ok());
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.get_output(dout_g), inputs);
        
        // enable on, clock down
        assert!(circuit.set_input(clock_g, bitarr![0]).is_ok());
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.get_output(dout_g), inputs);
        
        // update din
        let neg_inputs = !inputs;
        assert!(circuit.set_input(din_g, BitArray::from_iter(neg_inputs)).is_ok());
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.get_output(dout_g), inputs);

        for _ in 0..3 {
            // enable off, clock down
            assert!(circuit.set_input(enable_g, bitarr![0]).is_ok());
            circuit.run(&[din, enable, clock, clear]);
            assert_eq!(circuit.get_output(dout_g), inputs);
    
            // enable off, clock up
            assert!(circuit.set_input(clock_g, bitarr![1]).is_ok());
            circuit.run(&[din, enable, clock, clear]);
            assert_eq!(circuit.get_output(dout_g), inputs);
        }

        // enable off, clock down
        assert!(circuit.set_input(clock_g, bitarr![0]).is_ok());
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.get_output(dout_g), inputs);

        // enable on, clock up
        assert!(circuit.set_input(enable_g, bitarr![1]).is_ok());
        assert!(circuit.set_input(clock_g, bitarr![1]).is_ok());
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.get_output(dout_g), neg_inputs);

        // reset
        assert!(circuit.set_input(clear_g, bitarr![1]).is_ok());
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.get_output(dout_g), bitarr![0; 8]);
        
        // reset, clock down
        assert!(circuit.set_input(clock_g, bitarr![0]).is_ok());
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.get_output(dout_g), bitarr![0; 8]);
        
        // reset, clock up
        assert!(circuit.set_input(clock_g, bitarr![1]).is_ok());
        circuit.run(&[din, enable, clock, clear]);
        assert_eq!(circuit.get_output(dout_g), bitarr![0; 8]);
    }
}
