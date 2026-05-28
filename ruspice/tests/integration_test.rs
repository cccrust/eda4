use ruspice::{Circuit, analyze_dc, analyze_ac, analyze_transient};
use approx::assert_relative_eq;

#[test]
fn test_simple_resistor_circuit() {
    let mut circuit = Circuit::new("Simple Resistor");
    circuit.add_node("v1");
    circuit.add_resistor("R1", "v1", "gnd", 1000.0);
    circuit.add_voltage_source("V1", "v1", "gnd", 5.0);

    let result = analyze_dc(&circuit);
    assert_relative_eq!(result.node_voltages[1], 5.0, epsilon = 0.001);
}

#[test]
fn test_resistor_divider_voltage() {
    let mut circuit = Circuit::new("Resistor Divider");
    circuit.add_node("vin");
    circuit.add_node("vout");
    circuit.add_resistor("R1", "vin", "vout", 1000.0);
    circuit.add_resistor("R2", "vout", "gnd", 1000.0);
    circuit.add_voltage_source("V1", "vin", "gnd", 10.0);

    let result = analyze_dc(&circuit);
    let vout = result.node_voltages[2];
    assert_relative_eq!(vout, 5.0, epsilon = 0.01);
}

#[test]
fn test_resistor_divider_with_load() {
    let mut circuit = Circuit::new("Loaded Divider");
    circuit.add_node("vin");
    circuit.add_node("vout");
    circuit.add_node("load");
    circuit.add_resistor("R1", "vin", "vout", 1000.0);
    circuit.add_resistor("R2", "vout", "gnd", 1000.0);
    circuit.add_resistor("RL", "vout", "load", 2000.0);
    circuit.add_voltage_source("V1", "vin", "gnd", 10.0);

    let result = analyze_dc(&circuit);
    let vout = result.node_voltages[2];
    assert!(vout < 5.0 && vout > 4.0);
}

#[test]
fn test_series_resistors() {
    let mut circuit = Circuit::new("Series Resistors");
    circuit.add_node("vin");
    circuit.add_node("mid");
    circuit.add_resistor("R1", "vin", "mid", 500.0);
    circuit.add_resistor("R2", "mid", "gnd", 500.0);
    circuit.add_voltage_source("V1", "vin", "gnd", 10.0);

    let result = analyze_dc(&circuit);
    assert_relative_eq!(result.node_voltages[1], 10.0, epsilon = 0.001);
    assert_relative_eq!(result.node_voltages[2], 5.0, epsilon = 0.001);
}

#[test]
fn test_parallel_resistors() {
    let mut circuit = Circuit::new("Parallel Resistors");
    circuit.add_node("vin");
    circuit.add_node("vout");
    circuit.add_resistor("R1", "vin", "vout", 1000.0);
    circuit.add_resistor("R2", "vin", "vout", 1000.0);
    circuit.add_voltage_source("V1", "vin", "gnd", 10.0);

    let result = analyze_dc(&circuit);
    let vout = result.node_voltages[2];
    assert_relative_eq!(vout, 10.0, epsilon = 0.001);
}

#[test]
fn test_rc_circuit_charging() {
    let mut circuit = Circuit::new("RC Charging");
    circuit.add_node("vin");
    circuit.add_node("vout");
    circuit.add_resistor("R1", "vin", "vout", 1000.0);
    circuit.add_capacitor("C1", "vout", "gnd", 1e-6);
    circuit.add_voltage_source("V1", "vin", "gnd", 5.0);

    let result = analyze_transient(&circuit, 0.0, 0.005, 0.0001);

    let first_v = result.node_voltages[0][2];
    let last_v = result.node_voltages.last().unwrap()[2];

    assert!(first_v < 1.0);
    assert!(last_v > 4.0);
}

#[test]
fn test_rc_time_constant() {
    let mut circuit = Circuit::new("RC Time Constant");
    circuit.add_node("vin");
    circuit.add_node("vout");
    circuit.add_resistor("R1", "vin", "vout", 1000.0);
    circuit.add_capacitor("C1", "vout", "gnd", 1e-6);
    circuit.add_voltage_source("V1", "vin", "gnd", 5.0);

    let result = analyze_transient(&circuit, 0.0, 0.003, 0.0001);

    let tau_index = (0.001 / 0.0001) as usize;
    if tau_index < result.node_voltages.len() {
        let v_at_tau = result.node_voltages[tau_index][2];
        assert!(v_at_tau > 2.0 && v_at_tau < 4.5);
    }
}

#[test]
fn test_ac_lowpass_response() {
    let mut circuit = Circuit::new("RC Lowpass");
    circuit.add_node("input");
    circuit.add_node("output");
    circuit.add_resistor("R1", "input", "output", 1600.0);
    circuit.add_capacitor("C1", "output", "gnd", 0.1e-6);

    let mut vs = ruspice::Component::voltage_source(
        "V1",
        circuit.get_node("input").unwrap(),
        circuit.get_node("gnd").unwrap(),
        1.0,
    );
    vs.ac_amplitude = 1.0;
    circuit.add_component(vs);

    let result = analyze_ac(&circuit, 100.0, 10000.0, 10);

    assert_eq!(result.frequencies.len(), 10);
    assert_eq!(result.magnitudes.len(), 10);
    assert_eq!(result.phases.len(), 10);
}

#[test]
fn test_dc_voltage_through_capacitor() {
    let mut circuit = Circuit::new("Capacitor DC");
    circuit.add_node("vin");
    circuit.add_node("vout");
    circuit.add_capacitor("C1", "vin", "vout", 1e-6);
    circuit.add_resistor("R1", "vout", "gnd", 1000.0);
    circuit.add_voltage_source("V1", "vin", "gnd", 5.0);

    let result = analyze_dc(&circuit);
    assert!(result.node_voltages[2] < 5.0);
}

#[test]
fn test_circuit_ground_always_zero() {
    let mut circuit = Circuit::new("Ground Test");
    circuit.add_node("test");
    circuit.add_voltage_source("V1", "test", "gnd", 7.5);

    let result = analyze_dc(&circuit);
    assert_relative_eq!(result.node_voltages[0], 0.0, epsilon = 1e-10);
}

#[test]
fn test_multiple_voltage_sources() {
    let mut circuit = Circuit::new("Multi Source");
    circuit.add_node("n1");
    circuit.add_node("n2");
    circuit.add_resistor("R1", "n1", "gnd", 1000.0);
    circuit.add_resistor("R2", "n2", "gnd", 2000.0);
    circuit.add_voltage_source("V1", "n1", "gnd", 5.0);
    circuit.add_voltage_source("V2", "n2", "gnd", 3.0);

    let result = analyze_dc(&circuit);
    assert_relative_eq!(result.node_voltages[1], 5.0, epsilon = 0.01);
    assert_relative_eq!(result.node_voltages[2], 3.0, epsilon = 0.01);
}

#[test]
fn test_inductor_stores_energy() {
    let mut circuit = Circuit::new("Inductor Test");
    circuit.add_node("vin");
    circuit.add_node("vout");
    circuit.add_inductor("L1", "vin", "vout", 0.01);
    circuit.add_resistor("R1", "vout", "gnd", 100.0);
    circuit.add_voltage_source("V1", "vin", "gnd", 10.0);

    let result = analyze_dc(&circuit);
    assert!(result.node_voltages[1] > 0.0);
}