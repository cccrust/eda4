use std::collections::HashMap;
use ruspice::{Circuit, analyze_dc, analyze_ac, analyze_transient};

fn main() {
    println!("ruspice basic examples\n");

    example_resistor_divider();
    example_wheatstone_bridge();
    example_rc_charging();
    example_rlc_circuit();
}

fn example_resistor_divider() {
    println!("1. Resistor Divider");
    println!("   V_in --[R1=1k]--+--[R2=1k]-- GND");
    println!("                   |");
    println!("                   V_out");
    println!();

    let mut circuit = Circuit::new("Resistor Divider");
    circuit.add_node("vin");
    circuit.add_node("vout");
    circuit.add_resistor("R1", "vin", "vout", 1000.0);
    circuit.add_resistor("R2", "vout", "gnd", 1000.0);
    circuit.add_voltage_source("V1", "vin", "gnd", 10.0);

    let result = analyze_dc(&circuit);
    println!("   Input: 10V, Expected output: 5V");
    println!("   Simulated output: {:.3}V\n", result.node_voltages[2]);
}

fn example_wheatstone_bridge() {
    println!("2. Wheatstone Bridge (balanced)");
    println!();

    let mut circuit = Circuit::new("Wheatstone Bridge");
    circuit.add_node("a");
    circuit.add_node("b");
    circuit.add_node("c");
    circuit.add_node("d");
    circuit.add_voltage_source("V1", "a", "d", 10.0);
    circuit.add_resistor("R1", "a", "b", 100.0);
    circuit.add_resistor("R2", "b", "d", 100.0);
    circuit.add_resistor("R3", "a", "c", 100.0);
    circuit.add_resistor("R4", "c", "d", 100.0);
    circuit.add_resistor("R5", "b", "c", 100.0);

    let result = analyze_dc(&circuit);
    let node_ids: HashMap<_, _> = circuit.nodes.iter().collect();
    let a_idx = *node_ids.get(&"a".to_string()).unwrap();
    let b_idx = *node_ids.get(&"b".to_string()).unwrap();
    let c_idx = *node_ids.get(&"c".to_string()).unwrap();
    let d_idx = *node_ids.get(&"d".to_string()).unwrap();

    let v_ab = result.node_voltages[*a_idx] - result.node_voltages[*b_idx];
    let v_cd = result.node_voltages[*c_idx] - result.node_voltages[*d_idx];

    println!("   All resistors = 100R, supply = 10V");
    println!("   V_ab (expected 0V for balance): {:.4}V", v_ab);
    println!("   V_cd (expected -5V): {:.4}V\n", v_cd);
}

fn example_rc_charging() {
    println!("3. RC Charging Circuit");
    println!("   V_in --[R=1k]--+--[C=1uF]-- GND");
    println!("                   |");
    println!("                   V_out");
    println!();

    let mut circuit = Circuit::new("RC Charging");
    circuit.add_node("vin");
    circuit.add_node("vout");
    circuit.add_resistor("R1", "vin", "vout", 1000.0);
    circuit.add_capacitor("C1", "vout", "gnd", 1e-6);
    circuit.add_voltage_source("V1", "vin", "gnd", 5.0);

    let result = analyze_transient(&circuit, 0.0, 0.005, 0.00005);
    let _tau = 0.001;

    println!("   Time constant = R*C = 1ms");
    println!("   Time    | V_out");
    println!("   --------|-------");
    for (i, &t) in result.time_points.iter().enumerate() {
        if i % 20 == 0 {
            let v = result.node_voltages[i][2];
            println!("   {:.4}s  | {:.4}V", t, v);
        }
    }
    println!();
}

fn example_rlc_circuit() {
    println!("4. RLC Bandpass Filter (AC analysis)");
    println!();

    let mut circuit = Circuit::new("RLC Bandpass");
    circuit.add_node("input");
    circuit.add_node("output");
    circuit.add_resistor("R1", "input", "output", 100.0);
    circuit.add_inductor("L1", "input", "output", 0.01);
    circuit.add_capacitor("C1", "output", "gnd", 0.001);

    let mut vs = ruspice::Component::voltage_source(
        "V1",
        circuit.get_node("input").unwrap(),
        circuit.get_node("gnd").unwrap(),
        1.0,
    );
    vs.ac_amplitude = 1.0;
    circuit.add_component(vs);

    let fc = 1.0 / (2.0 * std::f64::consts::PI * (100.0_f64 * 0.001).sqrt());
    println!("   R=100R, L=10mH, C=1mF");
    println!("   Resonant frequency: {:.0} Hz\n", fc);

    let result = analyze_ac(&circuit, 100.0, 10000.0, 20);

    println!("   Freq(Hz)  | Magnitude | Phase(deg)");
    println!("   ----------|------------|-------------");
    for (i, freq) in result.frequencies.iter().enumerate() {
        let mag = result.magnitudes[i][2];
        let phase = result.phases[i][2].to_degrees();
        println!("   {:>8.1}  | {:>10.4} | {:>11.2}", freq, mag, phase);
    }
}