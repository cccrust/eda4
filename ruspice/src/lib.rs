use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ComponentType {
    Resistor,
    Capacitor,
    Inductor,
    VoltageSource,
    CurrentSource,
    Diode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub component_type: ComponentType,
    pub node_pos: usize,
    pub node_neg: usize,
    pub value: f64,
    pub ac_amplitude: f64,
    pub ac_phase: f64,
}

impl Component {
    pub fn new(name: &str, component_type: ComponentType, node_pos: usize, node_neg: usize, value: f64) -> Self {
        Self {
            name: name.to_string(),
            component_type,
            node_pos,
            node_neg,
            value,
            ac_amplitude: 0.0,
            ac_phase: 0.0,
        }
    }

    pub fn resistor(name: &str, node_pos: usize, node_neg: usize, resistance: f64) -> Self {
        Self::new(name, ComponentType::Resistor, node_pos, node_neg, resistance)
    }

    pub fn capacitor(name: &str, node_pos: usize, node_neg: usize, capacitance: f64) -> Self {
        Self::new(name, ComponentType::Capacitor, node_pos, node_neg, capacitance)
    }

    pub fn inductor(name: &str, node_pos: usize, node_neg: usize, inductance: f64) -> Self {
        Self::new(name, ComponentType::Inductor, node_pos, node_neg, inductance)
    }

    pub fn voltage_source(name: &str, node_pos: usize, node_neg: usize, voltage: f64) -> Self {
        Self::new(name, ComponentType::VoltageSource, node_pos, node_neg, voltage)
    }

    pub fn current_source(name: &str, node_pos: usize, node_neg: usize, current: f64) -> Self {
        Self::new(name, ComponentType::CurrentSource, node_pos, node_neg, current)
    }

    pub fn with_ac(mut self, amplitude: f64, phase: f64) -> Self {
        self.ac_amplitude = amplitude;
        self.ac_phase = phase;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    pub name: String,
    pub voltage: f64,
}

impl Node {
    pub fn new(id: usize, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            voltage: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circuit {
    pub name: String,
    pub nodes: HashMap<String, usize>,
    pub ground: usize,
    pub components: Vec<Component>,
    pub num_nodes: usize,
}

impl Circuit {
    pub fn new(name: &str) -> Self {
        let mut circuit = Self {
            name: name.to_string(),
            nodes: HashMap::new(),
            ground: 0,
            components: Vec::new(),
            num_nodes: 0,
        };
        circuit.add_node("gnd");
        circuit
    }

    pub fn add_node(&mut self, name: &str) -> usize {
        if let Some(&id) = self.nodes.get(name) {
            return id;
        }
        let id = self.num_nodes;
        self.nodes.insert(name.to_string(), id);
        self.num_nodes += 1;
        id
    }

    pub fn get_node(&self, name: &str) -> Option<usize> {
        self.nodes.get(name).copied()
    }

    pub fn add_component(&mut self, component: Component) {
        self.components.push(component);
    }

    pub fn add_resistor(&mut self, name: &str, node_pos: &str, node_neg: &str, resistance: f64) {
        let np = self.add_node(node_pos);
        let nn = self.add_node(node_neg);
        self.add_component(Component::resistor(name, np, nn, resistance));
    }

    pub fn add_capacitor(&mut self, name: &str, node_pos: &str, node_neg: &str, capacitance: f64) {
        let np = self.add_node(node_pos);
        let nn = self.add_node(node_neg);
        self.add_component(Component::capacitor(name, np, nn, capacitance));
    }

    pub fn add_inductor(&mut self, name: &str, node_pos: &str, node_neg: &str, inductance: f64) {
        let np = self.add_node(node_pos);
        let nn = self.add_node(node_neg);
        self.add_component(Component::inductor(name, np, nn, inductance));
    }

    pub fn add_voltage_source(&mut self, name: &str, node_pos: &str, node_neg: &str, voltage: f64) {
        let np = self.add_node(node_pos);
        let nn = self.add_node(node_neg);
        self.add_component(Component::voltage_source(name, np, nn, voltage));
    }

    pub fn add_current_source(&mut self, name: &str, node_pos: &str, node_neg: &str, current: f64) {
        let np = self.add_node(node_pos);
        let nn = self.add_node(node_neg);
        self.add_component(Component::current_source(name, np, nn, current));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DCAnalysisResult {
    pub node_voltages: Vec<f64>,
    pub branch_currents: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ACAnalysisResult {
    pub frequencies: Vec<f64>,
    pub magnitudes: Vec<Vec<f64>>,
    pub phases: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransientResult {
    pub time_points: Vec<f64>,
    pub node_voltages: Vec<Vec<f64>>,
}

pub struct Solver {
    circuit: Circuit,
}

impl Solver {
    pub fn new(circuit: Circuit) -> Self {
        Self { circuit }
    }

    pub fn dc_analysis(&self) -> DCAnalysisResult {
        let n = self.circuit.num_nodes;
        let num_v_sources = self.circuit.components.iter()
            .filter(|c| matches!(c.component_type, ComponentType::VoltageSource))
            .count();

        let total_unknowns = n - 1 + num_v_sources;
        let mut conductance = DMatrix::zeros(total_unknowns, total_unknowns);
        let mut rhs = DVector::zeros(total_unknowns);

        for comp in &self.circuit.components {
            match comp.component_type {
                ComponentType::Resistor => {
                    let g = 1.0 / comp.value;
                    let n1 = if comp.node_pos == self.circuit.ground { 0 } else { comp.node_pos - 1 };
                    let n2 = if comp.node_neg == self.circuit.ground { 0 } else { comp.node_neg - 1 };

                    if comp.node_pos != self.circuit.ground {
                        conductance[(n1, n1)] += g;
                    }
                    if comp.node_neg != self.circuit.ground {
                        conductance[(n2, n2)] += g;
                    }
                    if comp.node_pos != self.circuit.ground && comp.node_neg != self.circuit.ground {
                        conductance[(n1, n2)] -= g;
                        conductance[(n2, n1)] -= g;
                    }
                }
                ComponentType::CurrentSource => {
                    let n1 = if comp.node_pos == self.circuit.ground { 0 } else { comp.node_pos - 1 };
                    let n2 = if comp.node_neg == self.circuit.ground { 0 } else { comp.node_neg - 1 };

                    if comp.node_pos != self.circuit.ground {
                        rhs[n1] -= comp.value;
                    }
                    if comp.node_neg != self.circuit.ground {
                        rhs[n2] += comp.value;
                    }
                }
                ComponentType::VoltageSource => {
                    // Will handle below with modified nodal analysis
                }
                _ => {}
            }
        }

        let mut vs_idx = n - 1;
        for comp in &self.circuit.components {
            if matches!(comp.component_type, ComponentType::VoltageSource) {
                let n1 = if comp.node_pos == self.circuit.ground { 0 } else { comp.node_pos - 1 };
                let n2 = if comp.node_neg == self.circuit.ground { 0 } else { comp.node_neg - 1 };

                if comp.node_pos != self.circuit.ground {
                    conductance[(n1, vs_idx)] = 1.0;
                    conductance[(vs_idx, n1)] = 1.0;
                }
                if comp.node_neg != self.circuit.ground {
                    conductance[(n2, vs_idx)] = -1.0;
                    conductance[(vs_idx, n2)] = -1.0;
                }
                rhs[vs_idx] = comp.value;
                vs_idx += 1;
            }
        }

        let voltages = match conductance.clone().try_inverse() {
            Some(inv) => inv * rhs,
            None => {
                let lu = conductance.lu();
                lu.solve(&rhs).unwrap_or_else(|| DVector::zeros(total_unknowns))
            }
        };

        let mut node_voltages = vec![0.0; n];
        node_voltages[0] = 0.0;
        for i in 1..n {
            node_voltages[i] = voltages[i - 1];
        }

        DCAnalysisResult {
            node_voltages,
            branch_currents: Vec::new(),
        }
    }

    pub fn ac_analysis(&self, start_freq: f64, end_freq: f64, num_points: usize) -> ACAnalysisResult {
        let frequencies: Vec<f64> = (0..num_points)
            .map(|i| {
                let log_start = start_freq.log10();
                let log_end = end_freq.log10();
                let log_freq = log_start + (log_end - log_start) * (i as f64 / (num_points - 1) as f64);
                10.0_f64.powf(log_freq)
            })
            .collect();

        let n = self.circuit.num_nodes;
        let mut magnitudes = Vec::new();
        let mut phases = Vec::new();

        for &freq in &frequencies {
            let omega = 2.0 * std::f64::consts::PI * freq;
            let num_v_sources = self.circuit.components.iter()
                .filter(|c| matches!(c.component_type, ComponentType::VoltageSource))
                .count();
            let total_unknowns = n - 1 + num_v_sources;

            let mut conductance_re = DMatrix::zeros(total_unknowns, total_unknowns);
            let mut conductance_im = DMatrix::zeros(total_unknowns, total_unknowns);
            let mut rhs_re: DVector<f64> = DVector::zeros(total_unknowns);
            let mut rhs_im: DVector<f64> = DVector::zeros(total_unknowns);

            for comp in &self.circuit.components {
                match comp.component_type {
                    ComponentType::Resistor => {
                        let g = 1.0 / comp.value;
                        let n1 = if comp.node_pos == self.circuit.ground { 0 } else { comp.node_pos - 1 };
                        let n2 = if comp.node_neg == self.circuit.ground { 0 } else { comp.node_neg - 1 };

                        if comp.node_pos != self.circuit.ground {
                            conductance_re[(n1, n1)] += g;
                        }
                        if comp.node_neg != self.circuit.ground {
                            conductance_re[(n2, n2)] += g;
                        }
                        if comp.node_pos != self.circuit.ground && comp.node_neg != self.circuit.ground {
                            conductance_re[(n1, n2)] -= g;
                            conductance_re[(n2, n1)] -= g;
                        }
                    }
                    ComponentType::Capacitor => {
                        let g = omega * comp.value;
                        let n1 = if comp.node_pos == self.circuit.ground { 0 } else { comp.node_pos - 1 };
                        let n2 = if comp.node_neg == self.circuit.ground { 0 } else { comp.node_neg - 1 };

                        if comp.node_pos != self.circuit.ground {
                            conductance_im[(n1, n1)] += g;
                        }
                        if comp.node_neg != self.circuit.ground {
                            conductance_im[(n2, n2)] += g;
                        }
                        if comp.node_pos != self.circuit.ground && comp.node_neg != self.circuit.ground {
                            conductance_im[(n1, n2)] -= g;
                            conductance_im[(n2, n1)] -= g;
                        }
                    }
                    ComponentType::Inductor => {
                        let g = 1.0 / (omega * comp.value);
                        let n1 = if comp.node_pos == self.circuit.ground { 0 } else { comp.node_pos - 1 };
                        let n2 = if comp.node_neg == self.circuit.ground { 0 } else { comp.node_neg - 1 };

                        if comp.node_pos != self.circuit.ground {
                            conductance_im[(n1, n1)] -= g;
                        }
                        if comp.node_neg != self.circuit.ground {
                            conductance_im[(n2, n2)] -= g;
                        }
                        if comp.node_pos != self.circuit.ground && comp.node_neg != self.circuit.ground {
                            conductance_im[(n1, n2)] += g;
                            conductance_im[(n2, n1)] += g;
                        }
                    }
                    ComponentType::VoltageSource => {
                        let amp = if comp.ac_amplitude > 0.0 { comp.ac_amplitude } else { comp.value };
                        let phase = comp.ac_phase;
                        let n1 = if comp.node_pos == self.circuit.ground { 0 } else { comp.node_pos - 1 };
                        let n2 = if comp.node_neg == self.circuit.ground { 0 } else { comp.node_neg - 1 };

                        if comp.node_pos != self.circuit.ground {
                            conductance_re[(n1, n1)] += 1.0;
                            rhs_re[n1] += amp * phase.to_radians().cos();
                            rhs_im[n1] += amp * phase.to_radians().sin();
                        }
                        if comp.node_neg != self.circuit.ground {
                            conductance_re[(n2, n2)] += 1.0;
                            rhs_re[n2] -= amp * phase.to_radians().cos();
                            rhs_im[n2] -= amp * phase.to_radians().sin();
                        }
                    }
                    ComponentType::CurrentSource => {
                        let amp = if comp.ac_amplitude > 0.0 { comp.ac_amplitude } else { comp.value };
                        let phase = comp.ac_phase;
                        let n1 = if comp.node_pos == self.circuit.ground { 0 } else { comp.node_pos - 1 };
                        let n2 = if comp.node_neg == self.circuit.ground { 0 } else { comp.node_neg - 1 };

                        if comp.node_pos != self.circuit.ground {
                            rhs_re[n1] -= amp * phase.to_radians().cos();
                            rhs_im[n1] -= amp * phase.to_radians().sin();
                        }
                        if comp.node_neg != self.circuit.ground {
                            rhs_re[n2] += amp * phase.to_radians().cos();
                            rhs_im[n2] += amp * phase.to_radians().sin();
                        }
                    }
                    _ => {}
                }
            }

            let mut mag = Vec::new();
            let mut ph = Vec::new();
            for i in 0..n {
                let v_re = if i == 0 { 0.0 } else { conductance_re[(i-1, 0)] };
                let v_im = if i == 0 { 0.0 } else { conductance_im[(i-1, 0)] };
                let v_mag = (v_re * v_re + v_im * v_im).sqrt();
                let v_phase = v_im.atan2(v_re);
                mag.push(v_mag);
                ph.push(v_phase);
            }
            magnitudes.push(mag);
            phases.push(ph);
        }

        ACAnalysisResult {
            frequencies,
            magnitudes,
            phases,
        }
    }

    pub fn transient_analysis(&self, start_time: f64, end_time: f64, time_step: f64) -> TransientResult {
        let num_steps = ((end_time - start_time) / time_step).ceil() as usize + 1;
        let time_points: Vec<f64> = (0..num_steps)
            .map(|i| start_time + i as f64 * time_step)
            .collect();

        let n = self.circuit.num_nodes;
        let num_v_sources = self.circuit.components.iter()
            .filter(|c| matches!(c.component_type, ComponentType::VoltageSource))
            .count();
        let total_unknowns = n - 1 + num_v_sources;

        let mut node_voltages = vec![vec![0.0; n]; num_steps];
        let mut prev_voltages = vec![0.0; n];

        for (step, &_t) in time_points.iter().enumerate() {
            let mut conductance = DMatrix::zeros(total_unknowns, total_unknowns);
            let mut charges = DVector::zeros(total_unknowns);
            let mut rhs = DVector::zeros(total_unknowns);

            for comp in &self.circuit.components {
                match comp.component_type {
                    ComponentType::Resistor => {
                        let g = 1.0 / comp.value;
                        let n1 = if comp.node_pos == self.circuit.ground { 0 } else { comp.node_pos - 1 };
                        let n2 = if comp.node_neg == self.circuit.ground { 0 } else { comp.node_neg - 1 };

                        if comp.node_pos != self.circuit.ground {
                            conductance[(n1, n1)] += g;
                        }
                        if comp.node_neg != self.circuit.ground {
                            conductance[(n2, n2)] += g;
                        }
                        if comp.node_pos != self.circuit.ground && comp.node_neg != self.circuit.ground {
                            conductance[(n1, n2)] -= g;
                            conductance[(n2, n1)] -= g;
                        }
                    }
                    ComponentType::Capacitor => {
                        let c = comp.value;
                        let g = c / time_step;
                        let n1 = if comp.node_pos == self.circuit.ground { 0 } else { comp.node_pos - 1 };
                        let n2 = if comp.node_neg == self.circuit.ground { 0 } else { comp.node_neg - 1 };
                        let v_prev = prev_voltages[comp.node_pos] - prev_voltages[comp.node_neg];

                        if comp.node_pos != self.circuit.ground {
                            conductance[(n1, n1)] += g;
                            charges[n1] += g * v_prev;
                        }
                        if comp.node_neg != self.circuit.ground {
                            conductance[(n2, n2)] += g;
                            charges[n2] -= g * v_prev;
                        }
                        if comp.node_pos != self.circuit.ground && comp.node_neg != self.circuit.ground {
                            conductance[(n1, n2)] -= g;
                            conductance[(n2, n1)] -= g;
                        }
                    }
                    ComponentType::CurrentSource => {
                        let i = comp.value;
                        let n1 = if comp.node_pos == self.circuit.ground { 0 } else { comp.node_pos - 1 };
                        let n2 = if comp.node_neg == self.circuit.ground { 0 } else { comp.node_neg - 1 };

                        if comp.node_pos != self.circuit.ground {
                            rhs[n1] -= i;
                        }
                        if comp.node_neg != self.circuit.ground {
                            rhs[n2] += i;
                        }
                    }
                    ComponentType::Inductor => {
                        let l = comp.value;
                        let g = time_step / l;
                        let n1 = if comp.node_pos == self.circuit.ground { 0 } else { comp.node_pos - 1 };
                        let n2 = if comp.node_neg == self.circuit.ground { 0 } else { comp.node_neg - 1 };
                        let v_prev = prev_voltages[comp.node_pos] - prev_voltages[comp.node_neg];

                        if comp.node_pos != self.circuit.ground {
                            conductance[(n1, n1)] += g;
                            charges[n1] += g * v_prev;
                        }
                        if comp.node_neg != self.circuit.ground {
                            conductance[(n2, n2)] += g;
                            charges[n2] -= g * v_prev;
                        }
                        if comp.node_pos != self.circuit.ground && comp.node_neg != self.circuit.ground {
                            conductance[(n1, n2)] -= g;
                            conductance[(n2, n1)] -= g;
                        }
                    }
                    _ => {}
                }
            }

            let mut vs_idx = n - 1;
            for comp in &self.circuit.components {
                if matches!(comp.component_type, ComponentType::VoltageSource) {
                    let n1 = if comp.node_pos == self.circuit.ground { 0 } else { comp.node_pos - 1 };
                    let n2 = if comp.node_neg == self.circuit.ground { 0 } else { comp.node_neg - 1 };

                    if comp.node_pos != self.circuit.ground {
                        conductance[(n1, vs_idx)] = 1.0;
                        conductance[(vs_idx, n1)] = 1.0;
                    }
                    if comp.node_neg != self.circuit.ground {
                        conductance[(n2, vs_idx)] = -1.0;
                        conductance[(vs_idx, n2)] = -1.0;
                    }
                    rhs[vs_idx] = comp.value;
                    vs_idx += 1;
                }
            }

            let vector = rhs + charges;

            let voltages = match conductance.clone().try_inverse() {
                Some(inv) => inv * vector,
                None => {
                    let lu = conductance.lu();
                    lu.solve(&vector).unwrap_or_else(|| DVector::zeros(total_unknowns))
                }
            };

            node_voltages[step][0] = 0.0;
            for i in 1..n {
                node_voltages[step][i] = voltages[i - 1];
            }
            prev_voltages.copy_from_slice(&node_voltages[step]);
        }

        TransientResult {
            time_points,
            node_voltages,
        }
    }
}

pub fn analyze_dc(circuit: &Circuit) -> DCAnalysisResult {
    let solver = Solver::new(circuit.clone());
    solver.dc_analysis()
}

pub fn analyze_ac(circuit: &Circuit, start_freq: f64, end_freq: f64, num_points: usize) -> ACAnalysisResult {
    let solver = Solver::new(circuit.clone());
    solver.ac_analysis(start_freq, end_freq, num_points)
}

pub fn analyze_transient(circuit: &Circuit, start_time: f64, end_time: f64, time_step: f64) -> TransientResult {
    let solver = Solver::new(circuit.clone());
    solver.transient_analysis(start_time, end_time, time_step)
}

pub fn parse_spice(netlist: &str) -> Result<Circuit, String> {
    let mut circuit = Circuit::new("SPICE Netlist");
    let mut title = String::new();

    for line in netlist.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('*') || line.starts_with('.') && !line.to_uppercase().starts_with(".AC") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let first = parts[0];

        if first == "V" || first == "v" {
            if parts.len() >= 4 {
                let name = parts[0];
                let node_pos = parts[1];
                let node_neg = parts[2];
                let dc_value: f64 = parts[3].parse().map_err(|_| format!("Invalid DC value for {}", name))?;
                circuit.add_node(node_pos);
                circuit.add_node(node_neg);
                circuit.add_voltage_source(name, node_pos, node_neg, dc_value);
                if parts.len() >= 7 && parts[4].to_uppercase() == "AC" {
                    let ac_amp: f64 = parts[5].parse().unwrap_or(1.0);
                    if let Some(comp) = circuit.components.last_mut() {
                        comp.ac_amplitude = ac_amp;
                    }
                }
            }
        } else if first == "R" || first == "r" {
            if parts.len() >= 4 {
                let name = parts[0];
                let node_pos = parts[1];
                let node_neg = parts[2];
                let value: f64 = parts[3].parse().map_err(|_| format!("Invalid resistance for {}", name))?;
                circuit.add_node(node_pos);
                circuit.add_node(node_neg);
                circuit.add_resistor(name, node_pos, node_neg, value);
            }
        } else if first == "C" || first == "c" {
            if parts.len() >= 4 {
                let name = parts[0];
                let node_pos = parts[1];
                let node_neg = parts[2];
                let value: f64 = parts[3].parse().map_err(|_| format!("Invalid capacitance for {}", name))?;
                circuit.add_node(node_pos);
                circuit.add_node(node_neg);
                circuit.add_capacitor(name, node_pos, node_neg, value);
            }
        } else if first == "L" || first == "l" {
            if parts.len() >= 4 {
                let name = parts[0];
                let node_pos = parts[1];
                let node_neg = parts[2];
                let value: f64 = parts[3].parse().map_err(|_| format!("Invalid inductance for {}", name))?;
                circuit.add_node(node_pos);
                circuit.add_node(node_neg);
                circuit.add_inductor(name, node_pos, node_neg, value);
            }
        } else if first == "I" || first == "i" {
            if parts.len() >= 4 {
                let name = parts[0];
                let node_pos = parts[1];
                let node_neg = parts[2];
                let value: f64 = parts[3].parse().map_err(|_| format!("Invalid current for {}", name))?;
                circuit.add_node(node_pos);
                circuit.add_node(node_neg);
                circuit.add_current_source(name, node_pos, node_neg, value);
            }
        } else if line.to_uppercase().starts_with(".TITLE") {
            if parts.len() >= 2 {
                title = parts[1..].join(" ");
                circuit.name = title.clone();
            }
        }
    }

    Ok(circuit)
}

pub struct AnalysisOptions {
    pub dc: bool,
    pub ac: bool,
    pub transient: bool,
    pub ac_start: f64,
    pub ac_end: f64,
    pub ac_points: usize,
    pub tran_start: f64,
    pub tran_end: f64,
    pub tran_step: f64,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            dc: true,
            ac: true,
            transient: true,
            ac_start: 100.0,
            ac_end: 1e6,
            ac_points: 50,
            tran_start: 0.0,
            tran_end: 0.005,
            tran_step: 0.00005,
        }
    }
}

pub fn run_analysis(circuit: &Circuit, opts: &AnalysisOptions) {
    println!("\n=== Circuit: {} ===\n", circuit.name);
    println!("Nodes: {:?}", circuit.nodes);
    println!("Components: {}", circuit.components.len());

    if opts.dc {
        println!("\n--- DC Analysis ---");
        let result = analyze_dc(circuit);
        for (i, &v) in result.node_voltages.iter().enumerate() {
            println!("  Node {}: {:.4}V", i, v);
        }
    }

    if opts.ac {
        println!("\n--- AC Analysis ---");
        let result = analyze_ac(circuit, opts.ac_start, opts.ac_end, opts.ac_points);
        let node_idx = circuit.nodes.values().max().copied().unwrap_or(0).min(2);
        println!("  Freq(Hz)     | Magnitude | Phase(deg)");
        for (i, &freq) in result.frequencies.iter().enumerate() {
            if i % 10 == 0 {
                let mag = result.magnitudes[i][node_idx];
                let phase = result.phases[i][node_idx].to_degrees();
                println!("  {:>10.2}  | {:>10.4} | {:>11.2}", freq, mag, phase);
            }
        }
    }

    if opts.transient {
        println!("\n--- Transient Analysis ---");
        let result = analyze_transient(circuit, opts.tran_start, opts.tran_end, opts.tran_step);
        let node_idx = circuit.nodes.values().max().copied().unwrap_or(0).min(2);
        println!("  Time(s)    | V_node{}", node_idx);
        for (i, &t) in result.time_points.iter().enumerate() {
            if i % 20 == 0 {
                println!("  {:.5}   | {:.4}V", t, result.node_voltages[i][node_idx]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resistor_divider() {
        let mut circuit = Circuit::new("Resistor Divider");
        circuit.add_node("in");
        circuit.add_node("out");
        circuit.add_resistor("R1", "in", "out", 1000.0);
        circuit.add_resistor("R2", "out", "gnd", 1000.0);
        circuit.add_voltage_source("V1", "in", "gnd", 5.0);

        let result = analyze_dc(&circuit);
        assert!((result.node_voltages[2] - 2.5).abs() < 0.001);
    }

    #[test]
    fn test_rc_time_constant() {
        let mut circuit = Circuit::new("RC Circuit");
        circuit.add_node("in");
        circuit.add_node("out");
        circuit.add_resistor("R1", "in", "out", 1000.0);
        circuit.add_capacitor("C1", "out", "gnd", 1e-6);
        circuit.add_voltage_source("V1", "in", "gnd", 5.0);

        let result = analyze_transient(&circuit, 0.0, 0.005, 0.0001);
        let first_v = result.node_voltages[0][2];
        let mid_v = result.node_voltages[25][2];
        let last_v = result.node_voltages.last().unwrap()[2];
        eprintln!("first={:.4}, mid={:.4}, last={:.4}", first_v, mid_v, last_v);
        assert!(first_v < 1.0, "first_v = {} should be < 1.0", first_v);
        assert!(mid_v > 4.0, "mid_v = {} should be > 4.0", mid_v);
        assert!(last_v > 4.5, "last_v = {} should be > 4.5", last_v);
    }

    #[test]
    fn test_ohm_law() {
        let mut circuit = Circuit::new("Ohm's Law");
        circuit.add_node("in");
        circuit.add_resistor("R1", "in", "gnd", 500.0);
        circuit.add_voltage_source("V1", "in", "gnd", 10.0);

        let result = analyze_dc(&circuit);
        assert!((result.node_voltages[1] - 10.0).abs() < 0.001);
    }
}

pub mod visualization {
    use crate::{Circuit, ComponentType, DCAnalysisResult, TransientResult, ACAnalysisResult};
    use std::collections::HashMap;

    pub fn ascii_circuit(circuit: &Circuit) -> String {
        let mut s = String::new();
        s.push_str(&format!("Circuit: {}\n", circuit.name));
        s.push_str(&format!("Nodes: {}\n", circuit.num_nodes));
        s.push_str("Components:\n");
        s.push_str("----------\n");

        for comp in &circuit.components {
            let type_str = match comp.component_type {
                ComponentType::Resistor => "R",
                ComponentType::Capacitor => "C",
                ComponentType::Inductor => "L",
                ComponentType::VoltageSource => "V",
                ComponentType::CurrentSource => "I",
                ComponentType::Diode => "D",
            };

            let node_names: Vec<String> = circuit.nodes.iter()
                .map(|(name, &id)| {
                    if id == comp.node_pos {
                        format!("({})", name)
                    } else if id == comp.node_neg {
                        format!("[{}]", name)
                    } else {
                        name.clone()
                    }
                })
                .collect();

            s.push_str(&format!("  {} {}: {} between {} and {} = {}\n",
                type_str, comp.name,
                match comp.component_type {
                    ComponentType::Resistor => "resistor",
                    ComponentType::Capacitor => "capacitor",
                    ComponentType::Inductor => "inductor",
                    ComponentType::VoltageSource => "voltage source",
                    ComponentType::CurrentSource => "current source",
                    ComponentType::Diode => "diode",
                },
                node_names.get(0).map(|n| n.as_str()).unwrap_or("?"),
                node_names.get(1).map(|n| n.as_str()).unwrap_or("?"),
                comp.value
            ));
        }
        s
    }

    pub fn ascii_voltage_current_plot(result: &DCAnalysisResult, circuit: &Circuit) -> String {
        let mut s = String::new();
        s.push_str("Node Voltages (DC Analysis):\n");
        s.push_str("============================\n");

        let node_names: Vec<&String> = circuit.nodes.iter()
            .map(|(name, _)| name)
            .collect();

        for (i, &v) in result.node_voltages.iter().enumerate() {
            let name = if i < node_names.len() { node_names[i].as_str() } else { "?" };
            let bar_len = ((v / 5.0).max(0.0).min(1.0) * 20.0) as usize;
            let bar: String = (0..bar_len).map(|_| '█').collect();
            let empty: String = (0..(20 - bar_len)).map(|_| '░').collect();
            s.push_str(&format!("  {:>6}: {} {} {:.3}V\n", name, bar, empty, v));
        }
        s
    }

    pub fn ascii_transient_plot(result: &TransientResult, node_idx: usize) -> String {
        let mut s = String::new();
        s.push_str("Transient Response:\n");
        s.push_str("===================\n");

        if result.node_voltages.is_empty() || node_idx >= result.node_voltages[0].len() {
            return "Invalid node index\n".to_string();
        }

        let values: Vec<f64> = result.node_voltages.iter()
            .map(|v| v[node_idx])
            .collect();

        let min_v = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_v = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = (max_v - min_v).max(0.001);

        let num_rows = 20;
        let num_cols = values.len().min(60);

        let step = values.len() / num_cols;
        let mut grid: Vec<Vec<char>> = vec![vec![' '; num_cols]; num_rows];

        for (col, i) in (0..values.len()).step_by(step).enumerate() {
            if col >= num_cols { break; }
            let v = values[i];
            let row = ((max_v - v) / range * (num_rows - 1) as f64) as usize;
            let row = row.min(num_rows - 1);
            grid[row][col] = '█';
        }

        for (i, row) in grid.iter().enumerate() {
            let v = max_v - (i as f64 / (num_rows - 1) as f64) * range;
            s.push_str(&format!("{:>8.2} │{}\n", v, row.iter().collect::<String>()));
        }

        let _time_range = result.time_points.last().unwrap() - result.time_points[0];
        s.push_str(&format!("         └{}──\n", "─".repeat(num_cols)));
        s.push_str(&format!("          0        t = {:.4}s\n", result.time_points[0], ));
        s.push_str(&format!("                    t = {:.4}s\n", result.time_points.last().unwrap()));

        s
    }

    pub fn ascii_ac_plot(result: &ACAnalysisResult, node_idx: usize) -> String {
        let mut s = String::new();
        s.push_str("AC Frequency Response:\n");
        s.push_str("======================\n");

        if result.magnitudes.is_empty() || node_idx >= result.magnitudes[0].len() {
            return "Invalid node index\n".to_string();
        }

        let num_rows = 20;
        let num_cols = result.frequencies.len().min(50);

        let mut grid: Vec<Vec<char>> = vec![vec![' '; num_cols]; num_rows];

        for col in 0..num_cols {
            let i = col * result.frequencies.len() / num_cols;
            let mag = result.magnitudes[i][node_idx];
            let mag_db = if mag > 0.0 { 20.0 * mag.log10() } else { -100.0 };
            let row = ((mag_db + 60.0) / 60.0 * (num_rows - 1) as f64).max(0.0).min((num_rows - 1) as f64) as usize;
            let row = num_rows - 1 - row;
            if row < num_rows {
                grid[row][col] = '█';
            }
        }

        for (i, row) in grid.iter().enumerate() {
            let db = 60.0 - (i as f64 / (num_rows - 1) as f64) * 60.0;
            s.push_str(&format!("{:>7.0}dB │{}\n", db as i32, row.iter().collect::<String>()));
        }

        s.push_str(&format!("         └{}──\n", "─".repeat(num_cols)));
        let log_start = result.frequencies[0].log10() as i32;
        let log_end = result.frequencies.last().unwrap().log10() as i32;
        s.push_str(&format!("          10^{}Hz          10^{}Hz\n", log_start, log_end));

        s
    }

    pub fn svg_circuit(circuit: &Circuit) -> String {
        let mut svg = String::new();
        svg.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 400">
<style>
    .wire { stroke: #333; stroke-width: 2; fill: none; }
    .component { stroke: #006; stroke-width: 2; fill: none; }
    .label { font-family: monospace; font-size: 12px; fill: #333; }
    .node { fill: #333; }
</style>
"#);

        svg.push_str(&format!(r#"<text x="10" y="20" class="label">Circuit: {}</text>
<text x="10" y="380" class="label">Nodes: {} | Components: {}</text>
"#, circuit.name, circuit.num_nodes, circuit.components.len()));

        let width = 600.0;
        let height = 350.0;
        let margin = 50.0;

        let mut positions: HashMap<usize, (f64, f64)> = HashMap::new();
        let nodes_vec: Vec<(&String, &usize)> = circuit.nodes.iter().collect();
        let n = nodes_vec.len();

        for (i, (_, &node_id)) in nodes_vec.iter().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64 - std::f64::consts::PI / 2.0;
            let x = width / 2.0 + (width / 2.0 - margin) * angle.cos();
            let y = height / 2.0 + (height / 2.0 - margin) * angle.sin();
            positions.insert(node_id, (x, y));
        }

        if let Some(&gnd_id) = circuit.nodes.get("gnd") {
            positions.insert(gnd_id, (width - margin, height - margin));
        }

        for comp in &circuit.components {
            let (x1, y1) = positions.get(&comp.node_pos).copied().unwrap_or((100.0, 100.0));
            let (x2, y2) = positions.get(&comp.node_neg).copied().unwrap_or((200.0, 200.0));

            svg.push_str(&format!(r#"  <line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" class="wire"/>
  <text x="{:.0}" y="{:.0}" class="label" text-anchor="middle">{} = {:.2}</text>
"#,
                x1, y1, x2, y2,
                (x1 + x2) / 2.0, (y1 + y2) / 2.0 - 10.0,
                comp.name, comp.value
            ));

            let mid_x = (x1 + x2) / 2.0;
            let mid_y = (y1 + y2) / 2.0;
            let _angle = ((y2 - y1).atan2(x2 - x1) * 180.0 / std::f64::consts::PI).abs();

            match comp.component_type {
                ComponentType::Resistor => {
                    svg.push_str(&format!(r#"  <path d="M {:.0},{:.0} L {:.0},{:.0}" class="component"/>
  <text x="{:.0}" y="{:.0}" class="label">{}Ω</text>
"#,
                        x1, y1, x2, y2,
                        mid_x, mid_y + 15.0,
                        comp.value
                    ));
                }
                ComponentType::Capacitor => {
                    svg.push_str(&format!(r#"  <line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" class="wire"/>
  <line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" class="component"/>
"#,
                        x1, y1, x2, y2,
                        mid_x - 8.0, mid_y - 8.0,
                        mid_x - 8.0, mid_y + 8.0
                    ));
                    svg.push_str(&format!(r#"  <line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" class="component"/>
  <text x="{:.0}" y="{:.0}" class="label">{}F</text>
"#,
                        mid_x + 8.0, mid_y - 8.0,
                        mid_x + 8.0, mid_y + 8.0,
                        mid_x, mid_y + 25.0,
                        comp.value
                    ));
                }
                ComponentType::Inductor => {
                    let dx = x2 - x1;
                    let dy = y2 - y1;
                    let len = (dx * dx + dy * dy).sqrt();
                    let cx = -dy / len * 8.0;
                    let cy = dx / len * 8.0;
                    svg.push_str(&format!(r#"  <path d="M {:.0},{:.0} Q {:.0},{:.0} {:.0},{:.0}" class="component"/>
  <text x="{:.0}" y="{:.0}" class="label">{}H</text>
"#,
                        x1, y1, mid_x + cx, mid_y + cy, x2, y2,
                        mid_x, mid_y + 15.0,
                        comp.value
                    ));
                }
                ComponentType::VoltageSource => {
                    svg.push_str(&format!(r#"  <circle cx="{:.0}" cy="{:.0}" r="10" class="component"/>
  <text x="{:.0}" y="{:.0}" class="label">+{}V</text>
"#,
                        mid_x, mid_y,
                        mid_x + 15.0, mid_y - 5.0,
                        comp.value
                    ));
                }
                ComponentType::CurrentSource => {
                    svg.push_str(&format!(r#"  <circle cx="{:.0}" cy="{:.0}" r="10" class="component"/>
  <text x="{:.0}" y="{:.0}" class="label">{}A</text>
"#,
                        mid_x, mid_y,
                        mid_x + 15.0, mid_y - 5.0,
                        comp.value
                    ));
                }
                ComponentType::Diode => {}
            }
        }

        svg.push_str("</svg>\n");
        svg
    }

    pub fn plot_to_svg(result: &TransientResult, node_idx: usize, title: &str, xlabel: &str, ylabel: &str) -> String {
        let width = 600.0;
        let height = 400.0;
        let margin_left = 60.0;
        let margin_right = 30.0;
        let margin_top = 40.0;
        let margin_bottom = 50.0;

        let plot_width = width - margin_left - margin_right;
        let plot_height = height - margin_top - margin_bottom;

        let times = &result.time_points;
        let values: Vec<f64> = result.node_voltages.iter()
            .map(|v| v[node_idx])
            .collect();

        let min_t = times.first().copied().unwrap_or(0.0);
        let max_t = times.last().copied().unwrap_or(1.0);
        let min_v = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_v = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range_t = (max_t - min_t).max(0.001);
        let range_v = (max_v - min_v).max(0.001);

        let mut svg = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}">
<style>
    .grid {{ stroke: #ddd; stroke-width: 1; }}
    .axis {{ stroke: #333; stroke-width: 2; }}
    .line {{ fill: none; stroke: #0066cc; stroke-width: 2; }}
    .title {{ font-family: sans-serif; font-size: 16px; fill: #333; }}
    .label {{ font-family: sans-serif; font-size: 12px; fill: #666; }}
</style>
<text x="{}" y="25" class="title">{}</text>
"#,
            width, height,
            width / 2.0, title
        );

        for i in 0..=4 {
            let y = margin_top + plot_height * i as f64 / 4.0;
            let v = max_v - (i as f64 / 4.0) * range_v;
            svg.push_str(&format!(r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" class="grid"/>
  <text x="{}" y="{}" class="label" text-anchor="end">{:.3}</text>
"#,
                margin_left, y, width - margin_right, y,
                margin_left - 5.0, y + 4.0, v
            ));
        }

        for i in 0..=4 {
            let x = margin_left + plot_width * i as f64 / 4.0;
            let t = min_t + (i as f64 / 4.0) * range_t;
            svg.push_str(&format!(r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" class="grid"/>
  <text x="{}" y="{}" class="label" text-anchor="middle">{:.4}</text>
"#,
                x, margin_top, x, height - margin_bottom,
                x, height - margin_bottom + 20.0, t
            ));
        }

        svg.push_str(&format!(r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" class="axis"/>
  <line x1="{}" y1="{}" x2="{}" y2="{}" class="axis"/>
  <text x="{}" y="{}" class="label" text-anchor="middle">{}</text>
  <text x="15" y="{}" class="label" text-anchor="middle" transform="rotate(-90, 15, {})">{}</text>
"#,
            margin_left, height - margin_bottom, width - margin_right, height - margin_bottom,
            margin_left, margin_top, margin_left, height - margin_bottom,
            width / 2.0, height - 5.0, xlabel,
            height / 2.0, height / 2.0, ylabel
        ));

        if !values.is_empty() {
            let points: Vec<String> = values.iter().enumerate().map(|(i, &v)| {
                let x = margin_left + (times[i] - min_t) / range_t * plot_width;
                let y = margin_top + (1.0 - (v - min_v) / range_v) * plot_height;
                format!("{:.1},{:.1}", x, y)
            }).collect();

svg.push_str("  <polyline points=\"");
            svg.push_str(&points.join(" "));
            svg.push_str("\" class=\"line\"/>\n");
        }

        svg.push_str("</svg>\n");
        svg
    }
}