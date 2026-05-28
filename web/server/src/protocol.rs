use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Request {
    #[serde(rename = "verilog_sim")]
    VerilogSim {
        code: String,
        top: Option<String>,
    },
    #[serde(rename = "verilog_pnr")]
    VerilogPnr {
        code: String,
        device: Option<String>,
        top: Option<String>,
    },
    #[serde(rename = "spice_analyze")]
    SpiceAnalyze {
        code: String,
        analysis: SpiceAnalysisType,
        #[serde(default)]
        ac_freq_start: Option<f64>,
        #[serde(default)]
        ac_freq_end: Option<f64>,
        #[serde(default)]
        ac_points: Option<usize>,
        #[serde(default)]
        tran_start: Option<f64>,
        #[serde(default)]
        tran_end: Option<f64>,
        #[serde(default)]
        tran_step: Option<f64>,
    },
    #[serde(rename = "bitstream_decode")]
    BitstreamDecode {
        bin_base64: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SpiceAnalysisType {
    #[default]
    All,
   Dc,
    Ac,
    Transient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    #[serde(rename = "verilog_sim_result")]
    VerilogSimResult {
        stdout: String,
        stderr: String,
        generated_rust: String,
    },
    #[serde(rename = "verilog_pnr_result")]
    VerilogPnrResult {
        json: String,
        asc: String,
        bin_base64: String,
        device: String,
    },
    #[serde(rename = "spice_result")]
    SpiceResult {
        analysis_type: String,
        dc: Option<DcResult>,
        ac: Option<AcResult>,
        transient: Option<TransientResult>,
        ascii_circuit: String,
    },
    #[serde(rename = "bitstream_decode_result")]
    BitstreamDecodeResult {
        device: String,
        crc_valid: bool,
        tiles: serde_json::Value,
    },
    #[serde(rename = "error")]
    Error { error: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcResult {
    pub node_voltages: Vec<f64>,
    pub ascii_plot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcResult {
    pub frequencies: Vec<f64>,
    pub magnitudes: Vec<Vec<f64>>,
    pub phases: Vec<Vec<f64>>,
    pub ascii_plot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransientResult {
    pub time_points: Vec<f64>,
    pub node_voltages: Vec<Vec<f64>>,
    pub ascii_plot: String,
}