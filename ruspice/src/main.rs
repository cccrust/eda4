use ruspice::{Circuit, analyze_dc, analyze_ac, analyze_transient};
use ruspice::visualization;
use std::env;
use std::fs;

fn print_banner() {
    println!("ruspice v0.1.0 - Analog Circuit Simulator");
    println!("==========================================\n");
}

fn create_rc_circuit() -> Circuit {
    let mut circuit = Circuit::new("RC Charging");
    circuit.add_node("vin");
    circuit.add_node("vout");
    circuit.add_resistor("R1", "vin", "vout", 1000.0);
    circuit.add_capacitor("C1", "vout", "gnd", 1e-6);
    circuit.add_voltage_source("V1", "vin", "gnd", 5.0);
    circuit
}

fn create_resistor_divider() -> Circuit {
    let mut circuit = Circuit::new("Resistor Divider");
    circuit.add_node("input");
    circuit.add_node("output");
    circuit.add_resistor("R1", "input", "output", 1000.0);
    circuit.add_resistor("R2", "output", "gnd", 1000.0);
    circuit.add_voltage_source("V1", "input", "gnd", 5.0);
    circuit
}

fn create_lowpass_circuit() -> Circuit {
    let mut circuit = Circuit::new("RC Low-Pass");
    circuit.add_node("input");
    circuit.add_node("output");
    circuit.add_resistor("R1", "input", "output", 1600.0);
    circuit.add_capacitor("C1", "output", "gnd", 0.1e-6);
    let mut vs = ruspice::Component::voltage_source("V1", circuit.get_node("input").unwrap(), circuit.get_node("gnd").unwrap(), 1.0);
    vs.ac_amplitude = 1.0;
    vs.ac_phase = 0.0;
    circuit.add_component(vs);
    circuit
}

fn demo_resistor_divider() {
    println!("=== Resistor Divider Demo ===");
    let circuit = create_resistor_divider();

    let result = analyze_dc(&circuit);
    println!("Input voltage: 5.0V");
    println!("Output voltage: {:.3}V", result.node_voltages[2]);
    println!("Expected: 2.5V\n");
}

fn demo_rc_transient() {
    println!("=== RC Transient Demo ===");
    let circuit = create_rc_circuit();

    let result = analyze_transient(&circuit, 0.0, 0.005, 0.0001);
    println!("Time constant: 1ms");
    println!("Steps: {}", result.time_points.len());
    println!("Initial V(out): {:.3}V", result.node_voltages[0][2]);
    println!("Final V(out): {:.3}V", result.node_voltages.last().unwrap()[2]);
    println!("Expected final: ~5.0V\n");
}

fn demo_ac_analysis() {
    println!("=== AC Analysis Demo ===");
    let circuit = create_lowpass_circuit();

    let result = analyze_ac(&circuit, 100.0, 1e6, 10);
    println!("Frequency sweep: 100Hz to 1MHz (10 points)");
    println!("Sample magnitudes at output node:");
    for (i, freq) in result.frequencies.iter().enumerate() {
        if i < 3 || i > 7 {
            println!("  {:.2e} Hz: {:.4}", freq, result.magnitudes[i][2]);
        }
    }
    println!("...\n");
}

fn demo_rc_filter() {
    println!("=== RC Filter Frequency Response ===");
    let mut circuit = Circuit::new("RC Filter");
    circuit.add_node("in");
    circuit.add_node("out");
    circuit.add_resistor("R1", "in", "out", 1000.0);
    circuit.add_capacitor("C1", "out", "gnd", 0.16e-6);
    let mut vs = ruspice::Component::voltage_source("V1", circuit.get_node("in").unwrap(), circuit.get_node("gnd").unwrap(), 1.0);
    vs.ac_amplitude = 1.0;
    circuit.add_component(vs);

    let result = analyze_ac(&circuit, 10.0, 1e5, 20);
    println!("RC = 160us, fc = {:.0} Hz", 1.0 / (2.0 * std::f64::consts::PI * 1000.0 * 0.16e-6));
    for (i, freq) in result.frequencies.iter().enumerate() {
        let mag_db = 20.0 * result.magnitudes[i][2].log10();
        println!("{:>10.1} Hz: {:>8.2} dB", freq, mag_db);
    }
}

fn cmd_circuit(circuit: &Circuit) {
    println!("{}", visualization::ascii_circuit(circuit));
}

fn cmd_plot_dc(circuit: &Circuit) {
    let result = analyze_dc(circuit);
    println!("{}", visualization::ascii_voltage_current_plot(&result, circuit));
}

fn cmd_plot_transient(circuit: &Circuit, node_name: &str) {
    let node_idx = *circuit.nodes.get(node_name).unwrap_or(&0);
    let result = analyze_transient(circuit, 0.0, 0.005, 0.0001);
    println!("{}", visualization::ascii_transient_plot(&result, node_idx));
}

fn cmd_plot_ac(circuit: &Circuit, node_name: &str) {
    let node_idx = *circuit.nodes.get(node_name).unwrap_or(&0);
    let result = analyze_ac(circuit, 100.0, 1e5, 50);
    println!("{}", visualization::ascii_ac_plot(&result, node_idx));
}

fn cmd_save_svg(circuit: &Circuit, filename: &str) {
    let svg = visualization::svg_circuit(circuit);
    if let Err(e) = fs::write(filename, svg) {
        eprintln!("Error saving SVG: {}", e);
    } else {
        println!("Circuit diagram saved to {}", filename);
    }
}

fn cmd_save_plot(circuit: &Circuit, filename: &str, node_name: &str) {
    let node_idx = *circuit.nodes.get(node_name).unwrap_or(&0);
    let result = analyze_transient(circuit, 0.0, 0.005, 0.0001);
    let svg = visualization::plot_to_svg(&result, node_idx, "Transient Response", "Time (s)", "Voltage (V)");
    if let Err(e) = fs::write(filename, svg) {
        eprintln!("Error saving plot: {}", e);
    } else {
        println!("Plot saved to {}", filename);
    }
}

fn main() {
    print_banner();

    let args: Vec<String> = env::args().collect();

    let circuit = create_rc_circuit();

    if args.len() > 1 {
        match args[1].as_str() {
            "dc" => {
                demo_resistor_divider();
            }
            "transient" => {
                demo_rc_transient();
            }
            "ac" => {
                demo_ac_analysis();
            }
            "all" => {
                demo_resistor_divider();
                demo_rc_transient();
                demo_ac_analysis();
                demo_rc_filter();
            }
            "circuit" => {
                cmd_circuit(&circuit);
            }
            "plot-dc" => {
                let divider = create_resistor_divider();
                cmd_plot_dc(&divider);
            }
            "plot-transient" => {
                let node = args.get(2).map(|s| s.as_str()).unwrap_or("vout");
                cmd_plot_transient(&circuit, node);
            }
            "plot-ac" => {
                let node = args.get(2).map(|s| s.as_str()).unwrap_or("output");
                cmd_plot_ac(&circuit, node);
            }
            "svg" => {
                let filename = args.get(2).map(|s| s.as_str()).unwrap_or("circuit.svg");
                cmd_save_svg(&circuit, filename);
            }
            "save-plot" => {
                let filename = args.get(2).map(|s| s.as_str()).unwrap_or("transient.svg");
                let node = args.get(3).map(|s| s.as_str()).unwrap_or("vout");
                cmd_save_plot(&circuit, filename, node);
            }
            _ => {
                println!("Usage: ruspice [command]");
                println!("Commands:");
                println!("  dc              - Run DC analysis demo");
                println!("  transient       - Run transient analysis demo");
                println!("  ac              - Run AC analysis demo");
                println!("  all             - Run all demos");
                println!("  circuit         - Display circuit ASCII diagram");
                println!("  plot-dc         - Plot DC node voltages");
                println!("  plot-transient  - Plot transient response (default: vout)");
                println!("  plot-ac [node]  - Plot AC frequency response");
                println!("  svg [file]      - Save circuit as SVG (default: circuit.svg)");
                println!("  save-plot [f] [n] - Save transient plot to SVG file");
                println!();
                println!("Running all demos by default...\n");
                demo_resistor_divider();
                demo_rc_transient();
                demo_ac_analysis();
                demo_rc_filter();
            }
        }
    } else {
        demo_resistor_divider();
        demo_rc_transient();
        demo_ac_analysis();
        demo_rc_filter();
    }

    println!("Done!");
}