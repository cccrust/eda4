use crate::protocol::{AcResult, DcResult, Request, Response, SpiceAnalysisType, TransientResult};
use ruspice::visualization;

pub fn handle(req: Request) -> Response {
    match req {
        Request::SpiceAnalyze {
            code,
            analysis,
            ac_freq_start,
            ac_freq_end,
            ac_points,
            tran_start,
            tran_end,
            tran_step,
        } => handle_spice(code, analysis, ac_freq_start, ac_freq_end, ac_points, tran_start, tran_end, tran_step),
        _ => Response::Error { error: "Unknown request type for spice service".into() },
    }
}

fn handle_spice(
    code: String,
    analysis: SpiceAnalysisType,
    ac_freq_start: Option<f64>,
    ac_freq_end: Option<f64>,
    ac_points: Option<usize>,
    tran_start: Option<f64>,
    tran_end: Option<f64>,
    tran_step: Option<f64>,
) -> Response {
    let circuit = match ruspice::parse_spice(&code) {
        Ok(c) => c,
        Err(e) => return Response::Error { error: format!("SPICE parse error: {}", e) },
    };

    let ascii_circuit = visualization::ascii_circuit(&circuit);
    let ac_start = ac_freq_start.unwrap_or(100.0);
    let ac_end = ac_freq_end.unwrap_or(1e6);
    let ac_pts = ac_points.unwrap_or(50);
    let t_start = tran_start.unwrap_or(0.0);
    let t_end = tran_end.unwrap_or(0.005);
    let t_step = tran_step.unwrap_or(0.00005);

    let dc = match &analysis {
        SpiceAnalysisType::All | SpiceAnalysisType::Dc => {
            let result = ruspice::analyze_dc(&circuit);
            let ascii_plot = visualization::ascii_voltage_current_plot(&result, &circuit);
            Some(DcResult {
                node_voltages: result.node_voltages,
                ascii_plot,
            })
        }
        _ => None,
    };

    let ac = match &analysis {
        SpiceAnalysisType::All | SpiceAnalysisType::Ac => {
            let result = ruspice::analyze_ac(&circuit, ac_start, ac_end, ac_pts);
            let max_node = circuit.nodes.len().saturating_sub(1);
            let ascii_plot = visualization::ascii_ac_plot(&result, max_node.min(2));
            Some(AcResult {
                frequencies: result.frequencies,
                magnitudes: result.magnitudes,
                phases: result.phases,
                ascii_plot,
            })
        }
        _ => None,
    };

    let transient = match &analysis {
        SpiceAnalysisType::All | SpiceAnalysisType::Transient => {
            let result = ruspice::analyze_transient(&circuit, t_start, t_end, t_step);
            let max_node = circuit.nodes.len().saturating_sub(1);
            let ascii_plot = visualization::ascii_transient_plot(&result, max_node.min(2));
            Some(TransientResult {
                time_points: result.time_points,
                node_voltages: result.node_voltages,
                ascii_plot,
            })
        }
        _ => None,
    };

    let analysis_type = match &analysis {
        SpiceAnalysisType::All => "all",
        SpiceAnalysisType::Dc => "dc",
        SpiceAnalysisType::Ac => "ac",
        SpiceAnalysisType::Transient => "transient",
    };

    Response::SpiceResult {
        analysis_type: analysis_type.to_string(),
        dc,
        ac,
        transient,
        ascii_circuit,
    }
}