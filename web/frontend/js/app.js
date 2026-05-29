const WS_URL = `${location.protocol === 'https:' ? 'wss:' : 'ws:'}//${location.host}/ws`;

let ws = null;
let wsReady = false;
let wsQueue = [];

const statusEl = document.getElementById('wsStatus');

function setStatus(state) {
  statusEl.className = `ws-status ${state}`;
  statusEl.textContent = state;
}

function connectWs() {
  setStatus('connecting');
  try {
    ws = new WebSocket(WS_URL);
  } catch (e) {
    setStatus('disconnected');
    return;
  }

  ws.onopen = () => {
    setStatus('connected');
    wsReady = true;
    while (wsQueue.length > 0) {
      const req = wsQueue.shift();
      ws.send(JSON.stringify(req));
    }
  };

  ws.onmessage = (evt) => {
    try {
      const resp = JSON.parse(evt.data);
      handleResponse(resp);
    } catch (e) {
      console.error('WS parse error:', e);
    }
  };

  ws.onerror = () => setStatus('disconnected');

  ws.onclose = () => {
    setStatus('disconnected');
    wsReady = false;
    setTimeout(connectWs, 3000);
  };
}

function sendWs(req) {
  if (wsReady && ws) {
    ws.send(JSON.stringify(req));
  } else {
    wsQueue.push(req);
  }
}

function sendHttp(method, path, body) {
  return fetch(path, {
    method,
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  }).then(r => r.json());
}

function handleResponse(resp) {
  switch (resp.type) {
    case 'verilog_sim_result': {
      const outEl = document.getElementById('verilog-output');
      const genEl = document.getElementById('verilog-generated');
      if (resp.error) {
        outEl.textContent = `Error: ${resp.error}`;
        outEl.className = 'output error';
      } else {
        const hasOut = resp.stdout && resp.stdout.trim();
        const hasErr = resp.stderr && resp.stderr.trim();
        if (hasOut) {
          outEl.textContent = resp.stdout;
          outEl.className = 'output success';
        } else if (hasErr) {
          outEl.textContent = resp.stderr;
          outEl.className = 'output error';
        } else {
          outEl.textContent = '(no output — design may not have $display or initial blocks)';
          outEl.className = 'output';
        }
        if (resp.generated_rust) {
          genEl.textContent = resp.generated_rust;
        }
      }
      break;
    }
    case 'verilog_pnr_result': {
      const outEl = document.getElementById('pnr-output');
      const jsonEl = document.getElementById('pnr-json');
      const ascEl = document.getElementById('pnr-asc');
      const metaEl = document.getElementById('pnr-meta');
      if (resp.error) {
        outEl.textContent = `Error: ${resp.error}`;
        outEl.className = 'output error';
      } else {
        const binBytes = atob(resp.bin_base64).length;
        outEl.textContent = `✓ Synthesis + PnR complete\nDevice: ${resp.device}\nBitstream: ${resp.bin_base64.length} base64 chars (~${binBytes} bytes)`;
        outEl.className = 'output success';
        try {
          jsonEl.textContent = JSON.stringify(JSON.parse(resp.json), null, 2).substring(0, 2000);
        } catch { jsonEl.textContent = resp.json.substring(0, 1000); }
        ascEl.textContent = resp.asc.substring(0, 1000);
        metaEl.textContent = `Device: ${resp.device} | BIN: ~${binBytes} bytes`;
        // Load visualization data
        loadVizData(resp.json, resp.asc);
      }
      break;
    }
    case 'spice_result': {
      const outEl = document.getElementById('spice-output');
      if (resp.error) {
        outEl.textContent = `Error: ${resp.error}`;
        outEl.className = 'output error';
      } else {
        let txt = `Circuit: (shown below)\n\n`;
        if (resp.ascii_circuit) txt += `=== Circuit ===\n${resp.ascii_circuit}\n`;
        if (resp.dc) {
          txt += `\n=== DC Analysis ===\n${resp.dc.ascii_plot}\n`;
        }
        if (resp.ac) {
          txt += `\n=== AC Analysis ===\n${resp.ac.ascii_plot}\n`;
        }
        if (resp.transient) {
          txt += `\n=== Transient Analysis ===\n${resp.transient.ascii_plot}\n`;
        }
        outEl.textContent = txt;
        outEl.className = 'output success';
      }
      break;
    }
    case 'error': {
      const errorMsg = resp.error || 'Unknown error';
      ['verilog-output', 'pnr-output', 'spice-output'].forEach(id => {
        const el = document.getElementById(id);
        if (el && !el.classList.contains('success')) {
          el.textContent = `Error: ${errorMsg}`;
          el.className = 'output error';
        }
      });
      break;
    }
  }
}

function runVerilogSim(_useHttp) {
  const code = document.getElementById('verilog-code').value.trim();
  if (!code) { alert('Please enter Verilog code'); return; }
  const outEl = document.getElementById('verilog-output');
  outEl.textContent = 'Running simulation...';
  outEl.className = 'output info';
  document.getElementById('verilog-generated').textContent = '';

  const req = { type: 'verilog_sim', code, top: null };
  if (wsReady) {
    sendWs(req);
  } else {
    sendHttp('POST', '/api/verilog/simulate', { code, top: null }).then(handleResponse);
  }
}

function runVerilogPnr(_useHttp) {
  const code = document.getElementById('pnr-code').value.trim();
  if (!code) { alert('Please enter Verilog code'); return; }
  const device = document.getElementById('pnr-device').value;
  const outEl = document.getElementById('pnr-output');
  outEl.textContent = 'Running synthesis + PnR...';
  outEl.className = 'output info';

  const req = { type: 'verilog_pnr', code, device, top: null };
  if (wsReady) {
    sendWs(req);
  } else {
    sendHttp('POST', '/api/verilog/pnr', { code, device, top: null }).then(handleResponse);
  }
}

function runSpiceAnalyze(_useHttp) {
  const code = document.getElementById('spice-code').value.trim();
  if (!code) { alert('Please enter SPICE netlist'); return; }
  const analysis = document.getElementById('spice-analysis').value;
  const outEl = document.getElementById('spice-output');
  outEl.textContent = 'Running SPICE analysis...';
  outEl.className = 'output info';

  const req = { type: 'spice_analyze', code, analysis, ac_freq_start: null, ac_freq_end: null, ac_points: null, tran_start: null, tran_end: null, tran_step: null };
  if (wsReady) {
    sendWs(req);
  } else {
    sendHttp('POST', '/api/spice/analyze', { code, analysis, ac_freq_start: null, ac_freq_end: null, ac_points: null, tran_start: null, tran_end: null, tran_step: null }).then(handleResponse);
  }
}

function clearOutput(id) {
  const el = document.getElementById(id);
  if (el) el.textContent = '';
}

const VERILOG_EXAMPLES = {
  halfadd: `module HalfAdder(a, b, sum, cout);
  input a, b;
  output sum, cout;
  xor u1(sum, a, b);
  and u2(cout, a, b);
endmodule

module halfadd_tb;
  reg a, b;
  wire sum, cout;
  initial begin
    $display("=== HalfAdder Test ===");
    $display("  a  b | sum cout");
    a=0; b=0; #1 $display("  %b  %b |  %b   %b", a, b, sum, cout);
    a=1; b=0; #1 $display("  %b  %b |  %b   %b", a, b, sum, cout);
    a=0; b=1; #1 $display("  %b  %b |  %b   %b", a, b, sum, cout);
    a=1; b=1; #1 $display("  %b  %b |  %b   %b", a, b, sum, cout);
    $display("===================");
    $finish;
  end
endmodule`,
  fulladd: `module FullAdder(a, b, cin, sum, cout);
  input a, b, cin;
  output sum, cout;
  wire s, c1, c2;
  xor u1(s, a, b);
  xor u2(sum, s, cin);
  and u3(c1, a, b);
  and u4(c2, s, cin);
  or u5(cout, c1, c2);
endmodule

module fulladd_tb;
  reg a, b, cin;
  wire sum, cout;
  initial begin
    $display("=== FullAdder Test ===");
    $display("  a  b cin | sum cout");
    a=0; b=0; cin=0; #1 $display("  %b  %b  %b  |  %b   %b", a, b, cin, sum, cout);
    a=1; b=0; cin=0; #1 $display("  %b  %b  %b  |  %b   %b", a, b, cin, sum, cout);
    a=0; b=1; cin=0; #1 $display("  %b  %b  %b  |  %b   %b", a, b, cin, sum, cout);
    a=1; b=1; cin=1; #1 $display("  %b  %b  %b  |  %b   %b", a, b, cin, sum, cout);
    $display("====================");
    $finish;
  end
endmodule`,
  adder4: `module FullAdder(a, b, cin, sum, cout);
  input a, b, cin;
  output sum, cout;
  wire s, c1, c2;
  xor u1(s, a, b);
  xor u2(sum, s, cin);
  and u3(c1, a, b);
  and u4(c2, s, cin);
  or u5(cout, c1, c2);
endmodule

module Adder4(a, b, cin, sum, cout);
  input [3:0] a, b;
  input cin;
  output [3:0] sum;
  output cout;
  wire [3:0] c;
  FullAdder fa0(a[0], b[0], cin, sum[0], c[0]);
  FullAdder fa1(a[1], b[1], c[0], sum[1], c[1]);
  FullAdder fa2(a[2], b[2], c[1], sum[2], c[2]);
  FullAdder fa3(a[3], b[3], c[2], sum[3], cout);
endmodule

module adder4_tb;
  reg [3:0] a, b;
  reg cin;
  wire [3:0] sum;
  wire cout;
  Adder4 uut(a, b, cin, sum, cout);
  initial begin
    $display("=== Adder4 (4-bit) Test ===");
    $display("  a     b     cin | sum    cout");
    a=4'h0; b=4'h0; cin=0; #1 $display("  %h    %h    %b   | %h    %b", a, b, cin, sum, cout);
    a=4'h1; b=4'h2; cin=0; #1 $display("  %h    %h    %b   | %h    %b", a, b, cin, sum, cout);
    a=4'hF; b=4'h1; cin=0; #1 $display("  %h    %h    %b   | %h    %b", a, b, cin, sum, cout);
    a=4'hF; b=4'h1; cin=1; #1 $display("  %h    %h    %b   | %h    %b", a, b, cin, sum, cout);
    a=4'hA; b=4'h5; cin=0; #1 $display("  %h    %h    %b   | %h    %b", a, b, cin, sum, cout);
    $display("==========================");
    $finish;
  end
endmodule`,
  alu: `module ALU(a, b, op, result, zero);
  input [7:0] a, b;
  input [2:0] op;
  output reg [7:0] result;
  output reg zero;
  always @(*) begin
    case (op)
      0: result = a + b;
      1: result = a - b;
      2: result = a & b;
      3: result = a | b;
      4: result = a ^ b;
      5: result = a << 1;
      6: result = a >> 1;
      default: result = 8'b0;
    endcase
    zero = (result == 0);
  end
endmodule

module alu_tb;
  reg [7:0] a, b;
  reg [2:0] op;
  wire [7:0] result;
  wire zero;
  ALU uut(a, b, op, result, zero);
  initial begin
    $display("=== ALU (8-bit) Test ===");
    $display("  a=%d  b=%d  op=%d  => result=%d  zero=%b", a, b, op, result, zero);
    a=10; b=5;  op=0; #1 $display("  a=%d  b=%d  op=%d  => result=%d  zero=%b", a, b, op, result, zero);
    a=10; b=5;  op=1; #1 $display("  a=%d  b=%d  op=%d  => result=%d  zero=%b", a, b, op, result, zero);
    a=10; b=5;  op=2; #1 $display("  a=%d  b=%d  op=%d  => result=%d  zero=%b", a, b, op, result, zero);
    a=10; b=5;  op=3; #1 $display("  a=%d  b=%d  op=%d  => result=%d  zero=%b", a, b, op, result, zero);
    a=10; b=5;  op=4; #1 $display("  a=%d  b=%d  op=%d  => result=%d  zero=%b", a, b, op, result, zero);
    a=10; b=5;  op=5; #1 $display("  a=%d  b=%d  op=%d  => result=%d  zero=%b", a, b, op, result, zero);
    a=10; b=5;  op=6; #1 $display("  a=%d  b=%d  op=%d  => result=%d  zero=%b", a, b, op, result, zero);
    a=0;  b=0;  op=0; #1 $display("  a=%d  b=%d  op=%d  => result=%d  zero=%b", a, b, op, result, zero);
    $display("=========================");
    $finish;
  end
endmodule`,
  mux2: `module Mux2(a, b, sel, y);
  input a, b, sel;
  output y;
  wire not_sel, t1, t2;
  not u1(not_sel, sel);
  and u2(t1, a, not_sel);
  and u3(t2, b, sel);
  or u4(y, t1, t2);
endmodule

module mux2_tb;
  reg a, b, sel;
  wire y;
  Mux2 uut(a, b, sel, y);
  initial begin
    $display("=== Mux2 (2-to-1) Test ===");
    $display("  sel  a  b | y");
    sel=0; a=0; b=1; #1 $display("  %b    %b  %b | %b", sel, a, b, y);
    sel=0; a=1; b=0; #1 $display("  %b    %b  %b | %b", sel, a, b, y);
    sel=1; a=0; b=1; #1 $display("  %b    %b  %b | %b", sel, a, b, y);
    sel=1; a=1; b=0; #1 $display("  %b    %b  %b | %b", sel, a, b, y);
    $display("=========================");
    $finish;
  end
endmodule`,
  mux4: `module Mux4(a, b, c, d, sel, y);
  input a, b, c, d;
  input [1:0] sel;
  output y;
  wire [1:0] not_sel;
  wire t1, t2, t3, t4;
  not u0(not_sel[0], sel[0]);
  not u1(not_sel[1], sel[1]);
  and u2(t1, a, not_sel[1], not_sel[0]);
  and u3(t2, b, not_sel[1], sel[0]);
  and u4(t3, c, sel[1], not_sel[0]);
  and u5(t4, d, sel[1], sel[0]);
  or u6(y, t1, t2, t3, t4);
endmodule

module mux4_tb;
  reg a, b, c, d;
  reg [1:0] sel;
  wire y;
  Mux4 uut(a, b, c, d, sel, y);
  initial begin
    $display("=== Mux4 (4-to-1) Test ===");
    $display("  sel | y");
    a=1; b=0; c=0; d=0;
    sel=0; #1 $display("  %b%b | %b", sel[1], sel[0], y);
    sel=1; #1 $display("  %b%b | %b", sel[1], sel[0], y);
    sel=2; #1 $display("  %b%b | %b", sel[1], sel[0], y);
    sel=3; #1 $display("  %b%b | %b", sel[1], sel[0], y);
    a=0; b=1; c=0; d=0;
    sel=0; #1 $display("  %b%b | %b", sel[1], sel[0], y);
    sel=1; #1 $display("  %b%b | %b", sel[1], sel[0], y);
    sel=2; #1 $display("  %b%b | %b", sel[1], sel[0], y);
    sel=3; #1 $display("  %b%b | %b", sel[1], sel[0], y);
    $display("==========================");
    $finish;
  end
endmodule`,
  dff: `module DFF(d, clk, q);
  input d, clk;
  output reg q;
  always @(posedge clk) begin
    q <= d;
  end
endmodule

module dff_tb;
  reg d, clk;
  wire q;
  DFF uut(d, clk, q);
  initial begin
    $display("=== D Flip-Flop Test ===");
    $display("  clk  d | q");
    clk=0; d=0;
    $display("  %b    %b | %b", clk, d, q);
    clk=1; #1 $display("  %b    %b | %b", clk, d, q);
    clk=0; d=1; #1 $display("  %b    %b | %b", clk, d, q);
    clk=1; #1 $display("  %b    %b | %b", clk, d, q);
    clk=0; d=0; #1 $display("  %b    %b | %b", clk, d, q);
    $display("========================");
    $finish;
  end
endmodule`,
  counter: `module Counter(clk, rst, count);
  input clk, rst;
  output [3:0] count;
  reg [3:0] count;
  always @(posedge clk) begin
    if (rst)
      count <= 0;
    else
      count <= count + 1;
  end
endmodule

module counter_tb;
  reg clk, rst;
  wire [3:0] count;
  Counter uut(clk, rst, count);
  initial begin
    $display("=== Counter (4-bit) Test ===");
    $display("  clk rst | count");
    clk=0; rst=1;
    $display("  %b   %b  | %h", clk, rst, count);
    clk=1; #1 rst=0;
    $display("  %b   %b  | %h", clk, rst, count);
    clk=0; #1 $display("  %b   %b  | %h", clk, rst, count);
    clk=1; #1 $display("  %b   %b  | %h", clk, rst, count);
    clk=0; #1 $display("  %b   %b  | %h", clk, rst, count);
    clk=1; #1 $display("  %b   %b  | %h", clk, rst, count);
    clk=0; #1 $display("  %b   %b  | %h", clk, rst, count);
    clk=1; #1 $display("  %b   %b  | %h", clk, rst, count);
    clk=0; #1 $display("  %b   %b  | %h", clk, rst, count);
    $display("============================");
    $finish;
  end
endmodule`,
  decoder: `module Decoder2x4(enable, in, out);
  input enable;
  input [1:0] in;
  output [3:0] out;
  wire [1:0] not_in;
  not u0(not_in[0], in[0]);
  not u1(not_in[1], in[1]);
  and u2(out[0], enable, not_in[1], not_in[0]);
  and u3(out[1], enable, not_in[1], in[0]);
  and u4(out[2], enable, in[1], not_in[0]);
  and u5(out[3], enable, in[1], in[0]);
endmodule

module decoder_tb;
  reg enable;
  reg [1:0] in;
  wire [3:0] out;
  Decoder2x4 uut(enable, in, out);
  initial begin
    $display("=== Decoder 2x4 Test ===");
    $display("  en in | out[3:0]");
    enable=1;
    in=0; #1 $display("   %b  %b | %b%b%b%b", enable, in, out[3], out[2], out[1], out[0]);
    in=1; #1 $display("   %b  %b | %b%b%b%b", enable, in, out[3], out[2], out[1], out[0]);
    in=2; #1 $display("   %b  %b | %b%b%b%b", enable, in, out[3], out[2], out[1], out[0]);
    in=3; #1 $display("   %b  %b | %b%b%b%b", enable, in, out[3], out[2], out[1], out[0]);
    enable=0; in=0; #1 $display("   %b  %b | %b%b%b%b", enable, in, out[3], out[2], out[1], out[0]);
    $display("=========================");
    $finish;
  end
endmodule`,
  adder8: `module FullAdder(a, b, cin, sum, cout);
  input a, b, cin;
  output sum, cout;
  wire s, c1, c2;
  xor u1(s, a, b);
  xor u2(sum, s, cin);
  and u3(c1, a, b);
  and u4(c2, s, cin);
  or u5(cout, c1, c2);
endmodule

module Adder4(a, b, cin, sum, cout);
  input [3:0] a, b;
  input cin;
  output [3:0] sum;
  output cout;
  wire [3:0] c;
  FullAdder fa0(a[0], b[0], cin, sum[0], c[0]);
  FullAdder fa1(a[1], b[1], c[0], sum[1], c[1]);
  FullAdder fa2(a[2], b[2], c[1], sum[2], c[2]);
  FullAdder fa3(a[3], b[3], c[2], sum[3], cout);
endmodule

module Adder8(a, b, cin, sum, cout);
  input [7:0] a, b;
  input cin;
  output [7:0] sum;
  output cout;
  wire c4;
  Adder4 low(.a(a[3:0]), .b(b[3:0]), .cin(cin), .sum(sum[3:0]), .cout(c4));
  Adder4 high(.a(a[7:4]), .b(b[7:4]), .cin(c4), .sum(sum[7:4]), .cout(cout));
endmodule

module adder8_tb;
  reg [7:0] a, b;
  reg cin;
  wire [7:0] sum;
  wire cout;
  Adder8 uut(a, b, cin, sum, cout);
  initial begin
    $display("=== Adder8 (8-bit) Test ===");
    $display("  a       b       cin | sum       cout");
    a=8'h00; b=8'h01; cin=0; #1 $display("  %h     %h     %b   | %h     %b", a, b, cin, sum, cout);
    a=8'hFF; b=8'h01; cin=0; #1 $display("  %h     %h     %b   | %h     %b", a, b, cin, sum, cout);
    a=8'h0F; b=8'hF0; cin=0; #1 $display("  %h     %h     %b   | %h     %b", a, b, cin, sum, cout);
    a=8'h55; b=8'h2A; cin=1; #1 $display("  %h     %h     %b   | %h     %b", a, b, cin, sum, cout);
    a=8'hAA; b=8'h55; cin=0; #1 $display("  %h     %h     %b   | %h     %b", a, b, cin, sum, cout);
    $display("==========================");
    $finish;
  end
endmodule`,
  register: `module Register(clk, d, q);
  input clk, d;
  output reg q;
  always @(posedge clk) q <= d;
endmodule

module register_tb;
  reg clk, d;
  wire q;
  Register uut(clk, d, q);
  initial begin
    $display("=== Register Test ===");
    $display("  clk  d | q");
    clk=0; d=0;
    $display("  %b    %b | %b", clk, d, q);
    clk=1; #1 $display("  %b    %b | %b", clk, d, q);
    clk=0; d=1; #1 $display("  %b    %b | %b", clk, d, q);
    clk=1; #1 $display("  %b    %b | %b", clk, d, q);
    clk=0; d=0; #1 $display("  %b    %b | %b", clk, d, q);
    $display("====================");
    $finish;
  end
endmodule`,
  fsm: `module FSM(clk, rst, in, out);
  input clk, rst, in;
  output reg [1:0] out;
  reg [1:0] state;
  parameter S0=2'b00, S1=2'b01, S2=2'b10;
  always @(posedge clk) begin
    if (rst) state <= S0;
    else case (state)
      S0: state <= in ? S1 : S0;
      S1: state <= in ? S2 : S0;
      S2: state <= S0;
    endcase
  end
  always @(*) case(state) S0: out=2'b01; S1: out=2'b10; S2: out=2'b11; endcase
endmodule

module fsm_tb;
  reg clk, rst, in;
  wire [1:0] out;
  FSM uut(clk, rst, in, out);
  initial begin
    $display("=== FSM (Moore, 3-state) Test ===");
    $display("  clk  rst  in | state  out");
    clk=0; rst=1; in=0;
    #1 $display("  %b    %b    %b  | S0     %b%b", clk, rst, in, out[1], out[0]);
    rst=0; #1
    in=1; clk=1; #1 clk=0;
    #1 $display("  %b    %b    %b  | S1     %b%b", clk, rst, in, out[1], out[0]);
    clk=1; #1 clk=0;
    in=1; #1 $display("  %b    %b    %b  | S2     %b%b", clk, rst, in, out[1], out[0]);
    clk=1; #1 clk=0;
    in=0; #1 $display("  %b    %b    %b  | S0     %b%b", clk, rst, in, out[1], out[0]);
    clk=1; #1 clk=0;
    in=1; #1 $display("  %b    %b    %b  | S1     %b%b", clk, rst, in, out[1], out[0]);
    $display("==================================");
    $finish;
  end
endmodule`,
  mcu0m: `module cpu(clock);
  input clock;
  parameter LD=0, ADD=1, JMP=2, ST=3, CMP=4, JEQ=5;
  reg [15:0] A;
  reg [15:0] IR;
  reg [15:0] SW;
  reg [15:0] PC;
  reg [15:0] pc0;
  reg [7:0]  m [0:32];
  reg [15:0] t;
  integer i;

  initial begin
    PC = 0;
    SW = 0;
    t = 0;
    m[0] = 8'h00; m[1] = 8'h16;
    m[2] = 8'h40; m[3] = 8'h1A;
    m[4] = 8'h50; m[5] = 8'h12;
    m[6] = 8'h10; m[7] = 8'h18;
    m[8] = 8'h30; m[9] = 8'h16;
    m[10] = 8'h00; m[11] = 8'h14;
    m[12] = 8'h10; m[13] = 8'h16;
    m[14] = 8'h30; m[15] = 8'h14;
    m[16] = 8'h20; m[17] = 8'h00;
    m[18] = 8'h20; m[19] = 8'h12;
    m[20] = 8'h00; m[21] = 8'h00;
    m[22] = 8'h00; m[23] = 8'h00;
    m[24] = 8'h00; m[25] = 8'h01;
    m[26] = 8'h00; m[27] = 8'h0A;

    $display("Memory dump:");
    $display("%8x: %8x", 0, {m[0], m[1]});
    $display("%8x: %8x", 2, {m[2], m[3]});
    $display("%8x: %8x", 4, {m[4], m[5]});
    $display("%8x: %8x", 6, {m[6], m[7]});
    $display("%8x: %8x", 8, {m[8], m[9]});
    $display("%8x: %8x", 10, {m[10], m[11]});
    $display("%8x: %8x", 12, {m[12], m[13]});
    $display("%8x: %8x", 14, {m[14], m[15]});
    $display("%8x: %8x", 16, {m[16], m[17]});
    $display("%8x: %8x", 18, {m[18], m[19]});
    $display("%8x: %8x", 20, {m[20], m[21]});
    $display("%8x: %8x", 22, {m[22], m[23]});
    $display("%8x: %8x", 24, {m[24], m[25]});
    $display("%8x: %8x", 26, {m[26], m[27]});
  end

  always @(posedge clock) begin
    IR = {m[PC], m[PC+1]};
    pc0 = PC;
    PC = PC + 2;
    if (IR[15:12] == LD) A = {m[IR[11:0]], m[IR[11:0]+1]};
    else if (IR[15:12] == ST) begin {m[IR[11:0]], m[IR[11:0]+1]} = A; end
    else if (IR[15:12] == CMP) begin SW[15] = (A < {m[IR[11:0]], m[IR[11:0]+1]}); SW[14] = (A == {m[IR[11:0]], m[IR[11:0]+1]}); end
    else if (IR[15:12] == ADD) A = A + {m[IR[11:0]], m[IR[11:0]+1]};
    else if (IR[15:12] == JMP) PC = IR[11:0];
    else if (IR[15:12] == JEQ) if (SW[14]) PC = IR[11:0];
    $display("%4dns PC=%x IR=%x, SW=%x, A=%d", t, pc0, IR, SW, A);
    t = t + 10;
  end
endmodule

module main;
  reg clock;
  cpu cpux(clock);
  initial clock = 0;
  always #10 clock = ~clock;
  initial #2000 $finish;
endmodule`,
};

const SPICE_EXAMPLES = {
  resdiv: `* Resistor Divider
V1 Vin gnd DC 10
R1 Vin Vout 1k
R2 Vout gnd 1k
.DC V1 0 10 0.5
.END`,
  rc: `* RC Circuit - AC Analysis
V1 Vin gnd AC 1
R1 Vin Vout 1k
C1 Vout gnd 1u
.AC LIN 50 100 10k
.END`,
  rcdiff: `* RC Transient - Differentiation
V1 Vin gnd PULSE(0 5 0 1u 1u 0.5m 1m)
R1 Vin Vout 1k
C1 Vout gnd 1u
.TRAN 10u 5m
.END`,
  diode: `* Diode Circuit
V1 Vin gnd DC 5
D1 Vin Vout 1N4148
R1 Vout gnd 1k
.DC V1 0 5 0.1
.END`,
  transistor: `* Transistor Amplifier
Vcc Vcc gnd DC 5
Vin base gnd AC 0.01
R1 Vcc base 10k
R2 base gnd 10k
Rc Vcc collector 1k
Re emitter gnd 500
Q1 collector base emitter 2N2222
.AC DEC 50 1 1Meg
.END`,
  ringosc: `* Ring Oscillator (3 inverters)
V1 Vdd gnd DC 5
M1 out1 net1 net1 Vdd CMOSN W=1u L=1u
M2 net1 net1 gnd gnd CMOSN W=1u L=1u
M3 out2 out1 out1 Vdd CMOSN W=1u L=1u
M4 net2 out2 out2 gnd CMOSN W=1u L=1u
M5 out3 net2 net2 Vdd CMOSN W=1u L=1u
M6 net3 out3 net3 gnd CMOSN W=1u L=1u
.TRAN 1n 1u
.END`,
  wheatstone: `* Wheatstone Bridge
V1 V+ gnd DC 10
R1 V+ n1 1k
R2 n1 n2 1k
R3 n2 gnd 1k
R4 V+ n2 2k
.DC V1 0 10 0.1
.END`,
  opamp: `* Op-Amp Inverting Amplifier
V+ V+ gnd DC 15
V- V- gnd DC -15
Vin in+ gnd AC 1
R1 in+ out 10k
R2 in- gnd 10k
E1 out gnd POLY(1) in- V+ V- 100k 0
.AC DEC 50 1 10k
.END`,
};

const PNR_EXAMPLES = {
  blinky: `module top(input clk, output led);
  reg [25:0] counter;
  always @(posedge clk) counter <= counter + 1;
  assign led = counter[25];
endmodule`,
  halfadd: `module top(input a, b, output sum, cout);
  xor u1(sum, a, b);
  and u2(cout, a, b);
endmodule`,
  fulladd: `module top(input a, b, cin, output sum, cout);
  wire s, c1, c2;
  xor u1(s, a, b);
  xor u2(sum, s, cin);
  and u3(c1, a, b);
  and u4(c2, s, cin);
  or u5(cout, c1, c2);
endmodule`,
  adder4: `module FullAdder(input a, b, cin, output sum, cout);
  wire s, c1, c2;
  xor u1(s, a, b);
  xor u2(sum, s, cin);
  and u3(c1, a, b);
  and u4(c2, s, cin);
  or u5(cout, c1, c2);
endmodule

module top(input [3:0] a, b, input cin, output [3:0] sum, output cout);
  wire [3:0] c;
  FullAdder fa0(a[0], b[0], cin, sum[0], c[0]);
  FullAdder fa1(a[1], b[1], c[0], sum[1], c[1]);
  FullAdder fa2(a[2], b[2], c[1], sum[2], c[2]);
  FullAdder fa3(a[3], b[3], c[2], sum[3], cout);
endmodule`,
  alu: `module ALU(input [3:0] a, b, input [1:0] op, output reg [3:0] result, output reg zero);
  always @(*) begin
    case (op)
      0: result = a + b;
      1: result = a - b;
      2: result = a & b;
      3: result = a | b;
    endcase
    zero = (result == 0);
  end
endmodule

module top(input [3:0] a, b, input [1:0] op, output [3:0] result, output zero);
  ALU alu(a, b, op, result, zero);
endmodule`,
  mux2: `module top(input a, b, sel, output y);
  wire not_sel, t1, t2;
  not u1(not_sel, sel);
  and u2(t1, a, not_sel);
  and u3(t2, b, sel);
  or u4(y, t1, t2);
endmodule`,
  counter8: `module top(input clk, rst, en, output [7:0] q);
  reg [7:0] count;
  always @(posedge clk) begin
    if (rst) count <= 0;
    else if (en) count <= count + 1;
  end
  assign q = count;
endmodule`,
  decoder: `module top(input enable, input [1:0] in, output [3:0] out);
  wire [1:0] not_in;
  not u0(not_in[0], in[0]);
  not u1(not_in[1], in[1]);
  and u2(out[0], enable, not_in[1], not_in[0]);
  and u3(out[1], enable, not_in[1], in[0]);
  and u4(out[2], enable, in[1], not_in[0]);
  and u5(out[3], enable, in[1], in[0]);
endmodule`,
};


document.getElementById('verilog-examples').addEventListener('change', function() {
  if (this.value) {
    const code = VERILOG_EXAMPLES[this.value];
    if (code) {
      document.getElementById('verilog-code').value = code;
      this.value = '';
    }
  }
});

document.getElementById('spice-examples').addEventListener('change', function() {
  if (this.value) {
    const code = SPICE_EXAMPLES[this.value];
    if (code) {
      document.getElementById('spice-code').value = code;
      this.value = '';
    }
  }
});

document.getElementById('pnr-examples').addEventListener('change', function() {
  if (this.value) {
    const code = PNR_EXAMPLES[this.value];
    if (code) {
      document.getElementById('pnr-code').value = code;
      this.value = '';
    }
  }
});

// ---- FPGA Visualization (Circuit / Layout / Routing) ----

// roundRect polyfill for older browsers
if (!CanvasRenderingContext2D.prototype.roundRect) {
  CanvasRenderingContext2D.prototype.roundRect = function(x, y, w, h, r) {
    if (typeof r === 'number') r = [r, r, r, r];
    const [tl, tr, br, bl] = r.map(v => Math.min(v, Math.min(w, h) / 2));
    this.moveTo(x + tl, y);
    this.lineTo(x + w - tr, y);
    this.quadraticCurveTo(x + w, y, x + w, y + tr);
    this.lineTo(x + w, y + h - br);
    this.quadraticCurveTo(x + w, y + h, x + w - br, y + h);
    this.lineTo(x + bl, y + h);
    this.quadraticCurveTo(x, y + h, x, y + h - bl);
    this.lineTo(x, y + tl);
    this.quadraticCurveTo(x, y, x + tl, y);
    this.closePath();
    return this;
  };
}

let vizState = {
  netlist: null,
  layout: null,
  view: 'circuit',
  panX: 0, panY: 0,
  zoom: 1,
};
let vizDrag = false, vizDx = 0, vizDy = 0;

function loadVizData(jsonStr, ascStr) {
  try {
    vizState.netlist = parseNetlist(jsonStr);
  } catch (e) { vizState.netlist = null; }
  try {
    vizState.layout = parseLayout(ascStr);
  } catch (e) { vizState.layout = null; }
  vizState.panX = 0; vizState.panY = 0; vizState.zoom = 1;
  renderViz();
}

function parseNetlist(text) {
  const root = JSON.parse(text);
  const modules = root.modules;
  if (!modules) throw new Error('no modules');
  const key = Object.keys(modules)[0];
  const mod = modules[key];
  const cells = {};
  if (mod.cells) {
    for (const [name, c] of Object.entries(mod.cells)) {
      const ports = {};
      if (c.connections) {
        for (const [pn, bits] of Object.entries(c.connections)) {
          ports[pn] = Array.isArray(bits) ? bits.map(b => Number(b)) : [];
        }
      }
      cells[name] = { cell_type: c.type || '?', ports };
    }
  }
  const netnames = {};
  if (mod.netnames) {
    for (const [name, n] of Object.entries(mod.netnames)) {
      netnames[name] = (n.bits || []).map(b => Number(b));
    }
  }
  const ports = {};
  if (mod.ports) {
    for (const [name, p] of Object.entries(mod.ports)) {
      ports[name] = { direction: p.direction || '?', bits: (p.bits || []).map(b => Number(b)) };
    }
  }
  return { cells, netnames, ports };
}

function parseLayout(text) {
  let device = '';
  const tiles = [];
  const wiring = [];
  let pending = null;
  for (const line of text.split('\n')) {
    const t = line.trim();
    if (!t || t.startsWith('#')) continue;
    if (t.startsWith('.device ')) {
      device = t.slice(8).trim();
    } else if (t.startsWith('.logic_tile ')) {
      const parts = t.slice(12).trim().split(/\s+/);
      if (parts.length >= 2) {
        const col = parseInt(parts[0]), row = parseInt(parts[1]);
        if (parts.length >= 6) {
          tiles.push({ x: col, y: row, cell_name: parts[5].replace(/"/g, '') });
        } else {
          pending = { col, row };
        }
      }
    } else if (t.startsWith('.sym') && pending) {
      const parts = t.split(/\s+/);
      if (parts.length >= 6) {
        tiles.push({ x: pending.col, y: pending.row, cell_name: parts[5].replace(/"/g, '') });
      }
      pending = null;
    } else if (t.startsWith('.wiring ')) {
      const parts = t.slice(8).trim().split(/\s+/);
      if (parts.length >= 4) {
        const fc = parseInt(parts[0]), fr = parseInt(parts[1]);
        const tc = parseInt(parts[2]), tr = (fc === tc) ? fr + 1 : fr;
        const track = parseInt(parts[3]);
        wiring.push({ from_col: fc, from_row: fr, to_col: tc, to_row: tr, track });
      }
    } else {
      pending = null;
    }
  }
  let cols, rows;
  const du = device.toUpperCase();
  if (du.includes('HX1K') || du.includes('LP1K')) { cols = 16; rows = 28; }
  else if (du.includes('HX4K')) { cols = 20; rows = 38; }
  else if (du.includes('HX8K')) { cols = 28; rows = 68; }
  else if (du.includes('UP5K')) { cols = 22; rows = 38; }
  else {
    const mc = tiles.reduce((m, t) => Math.max(m, t.x), 0);
    const mr = tiles.reduce((m, t) => Math.max(m, t.y), 0);
    cols = mc + 1; rows = mr + 1;
  }
  return { device, tiles, wiring, cols, rows };
}

// --- Drawing helpers ---

function cellColor(type) {
  const map = {
    '$_INPUT_': [70, 70, 120],
    '$_OUTPUT_': [120, 60, 60],
    '$_ONE_': [80, 90, 60],
    '$_ADD_': [60, 90, 120],
    '$_DFF_P_': [60, 110, 70],
    '$_AND_': [100, 70, 100],
  };
  const c = map[type] || [50, 50, 50];
  return `rgb(${c[0]},${c[1]},${c[2]})`;
}

function toScreen(x, y, w, h) {
  const cx = w / 2 + vizState.panX + vizDx;
  const cy = h / 2 + vizState.panY + vizDy;
  return [(x * vizState.zoom + cx), (y * vizState.zoom + cy)];
}

// --- Circuit view ---

function drawCircuitView(ctx, w, h) {
  const net = vizState.netlist;
  if (!net) {
    ctx.fillStyle = '#555';
    ctx.textAlign = 'center';
    ctx.font = '16px sans-serif';
    ctx.fillText('No netlist data. Run PnR first.', w/2, h/2);
    return;
  }
  const layerOrder = ['$_INPUT_', '$_OUTPUT_', '$_ONE_', '$_ADD_', '$_DFF_P_', '$_AND_'];
  const layers = {};
  for (const l of layerOrder) layers[l] = [];
  for (const [name, cell] of Object.entries(net.cells)) {
    const key = cell.cell_type;
    (layers[key] || (layers[key] = [])).push([name, cell]);
  }

  const cellW = 120, cellH = 50, gapX = 80, gapY = 30;
  const positions = {};
  let col = 0;
  for (const layerName of layerOrder) {
    const cells = layers[layerName];
    if (!cells || !cells.length) continue;
    const n = cells.length;
    const totalH = n * cellH + (n - 1) * gapY;
    let rowOff = -totalH / 2;
    for (const [name] of cells) {
      positions[name] = { x: col, y: rowOff + cellH / 2 };
      rowOff += cellH + gapY;
    }
    col += cellW + gapX;
  }

  if (!Object.keys(positions).length) {
    ctx.fillStyle = '#555';
    ctx.textAlign = 'center';
    ctx.font = '14px sans-serif';
    ctx.fillText('No cells to display', w/2, h/2);
    return;
  }

  // Net -> cells mapping
  const netToCells = {};
  for (const [name, cell] of Object.entries(net.cells)) {
    for (const [_, bits] of Object.entries(cell.ports)) {
      for (const b of bits) {
        if (!netToCells[b]) netToCells[b] = [];
        netToCells[b].push(name);
      }
    }
  }

  // Draw edges
  ctx.lineWidth = 1.5;
  ctx.strokeStyle = 'rgba(137, 180, 250, 0.4)';
  for (const [_, conns] of Object.entries(netToCells)) {
    if (conns.length < 2) continue;
    for (let i = 0; i < conns.length; i++) {
      for (let j = i + 1; j < conns.length; j++) {
        const pa = positions[conns[i]], pb = positions[conns[j]];
        if (!pa || !pb) continue;
        const [sx, sy] = toScreen(pa.x, pa.y, w, h);
        const [ex, ey] = toScreen(pb.x, pb.y, w, h);
        const mx = (sx + ex) / 2, my = (sy + ey) / 2;
        const cp = { x: mx, y: my + 30 * vizState.zoom };
        ctx.beginPath();
        ctx.moveTo(sx, sy);
        for (let t = 1; t <= 20; t++) {
          const r = t / 20;
          const mt = 1 - r;
          const px = mt * mt * sx + 2 * mt * r * cp.x + r * r * ex;
          const py = mt * mt * sy + 2 * mt * r * cp.y + r * r * ey;
          ctx.lineTo(px, py);
        }
        ctx.stroke();
      }
    }
  }

  // Draw cells
  for (const [name, cell] of Object.entries(net.cells)) {
    const pos = positions[name];
    if (!pos) continue;
    const [sx, sy] = toScreen(pos.x, pos.y, w, h);
    const cw = cellW * vizState.zoom, ch = cellH * vizState.zoom;
    const r = cw / 2, b = ch / 2;
    ctx.fillStyle = cellColor(cell.cell_type);
    ctx.strokeStyle = '#fff';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.roundRect(sx - r, sy - b, cw, ch, 4);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = '#fff';
    ctx.font = `${Math.max(9 * vizState.zoom, 4)}px sans-serif`;
    ctx.textAlign = 'left';
    ctx.textBaseline = 'middle';
    ctx.fillText(name, sx - r + 6, sy);
    ctx.fillStyle = '#aaa';
    ctx.textAlign = 'right';
    ctx.fillText(cell.cell_type, sx + r - 6, sy);
  }
}

// --- Layout view ---

function drawLayoutView(ctx, w, h) {
  const lay = vizState.layout;
  if (!lay) {
    ctx.fillStyle = '#555';
    ctx.textAlign = 'center';
    ctx.font = '16px sans-serif';
    ctx.fillText('No layout data. Run PnR first.', w/2, h/2);
    return;
  }
  const tileW = 14, tileH = 14, gap = 2;
  const stepX = tileW + gap, stepY = tileH + gap;
  const ox = -(lay.cols * stepX) / 2, oy = -(lay.rows * stepY) / 2;
  const tileMap = {};
  for (const t of lay.tiles) tileMap[`${t.x},${t.y}`] = t;

  for (let row = 0; row < lay.rows; row++) {
    for (let col = 0; col < lay.cols; col++) {
      const [sx, sy] = toScreen(ox + col * stepX, oy + row * stepY, w, h);
      const used = tileMap[`${col},${row}`] !== undefined;
      ctx.fillStyle = used ? 'rgb(80, 150, 80)' : 'rgba(40, 40, 50, 0.8)';
      ctx.fillRect(sx, sy, tileW * vizState.zoom, tileH * vizState.zoom);
      ctx.strokeStyle = 'rgba(60,60,60,0.5)';
      ctx.lineWidth = 0.5;
      ctx.strokeRect(sx, sy, tileW * vizState.zoom, tileH * vizState.zoom);
    }
  }

  // IO port labels
  ctx.fillStyle = '#ff0';
  ctx.font = `${Math.max(10 * vizState.zoom, 4)}px sans-serif`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'bottom';
  for (const t of lay.tiles) {
    if (t.cell_name.startsWith('port_')) {
      const [sx, sy] = toScreen(ox + t.x * stepX, oy + t.y * stepY, w, h);
      ctx.fillText(t.cell_name, sx + (tileW * vizState.zoom) / 2, sy);
    }
  }

}

// --- Routing view ---

function drawRoutingView(ctx, w, h) {
  const net = vizState.netlist;
  const lay = vizState.layout;
  if (!net || !lay) {
    ctx.fillStyle = '#555';
    ctx.textAlign = 'center';
    ctx.font = '16px sans-serif';
    ctx.fillText('Need both netlist and layout. Run PnR first.', w/2, h/2);
    return;
  }
  const tileW = 14, tileH = 14, gap = 2;
  const stepX = tileW + gap, stepY = tileH + gap;
  const ox = -(lay.cols * stepX) / 2, oy = -(lay.rows * stepY) / 2;

  const tileCenter = (tx, ty) => [
    ox + tx * stepX + stepX / 2,
    oy + ty * stepY + stepY / 2,
  ];

  // Background grid
  for (let row = 0; row < lay.rows; row++) {
    for (let col = 0; col < lay.cols; col++) {
      const [sx, sy] = toScreen(ox + col * stepX, oy + row * stepY, w, h);
      ctx.fillStyle = 'rgba(30,30,40,0.4)';
      ctx.fillRect(sx, sy, tileW * vizState.zoom, tileH * vizState.zoom);
      ctx.strokeStyle = 'rgba(40,40,40,0.3)';
      ctx.lineWidth = 0.3;
      ctx.strokeRect(sx, sy, tileW * vizState.zoom, tileH * vizState.zoom);
    }
  }

  // Wiring segments
  const trackColors = [
    'rgb(255,100,100)', 'rgb(100,200,255)', 'rgb(100,255,100)',
    'rgb(255,255,100)', 'rgb(255,150,50)',  'rgb(200,100,255)',
    'rgb(255,100,200)', 'rgb(100,255,200)',
  ];
  for (const seg of lay.wiring) {
    const sc = tileCenter(seg.from_col, seg.from_row);
    const ec = tileCenter(seg.to_col, seg.to_row);
    const [fx, fy] = toScreen(sc[0], sc[1], w, h);
    const [tx2, ty2] = toScreen(ec[0], ec[1], w, h);
    ctx.strokeStyle = trackColors[seg.track % trackColors.length];
    ctx.lineWidth = 2.5 * vizState.zoom;
    ctx.beginPath();
    ctx.moveTo(fx, fy);
    ctx.lineTo(tx2, ty2);
    ctx.stroke();
  }

  // Cell -> tile mapping
  const cellTile = {};
  for (const t of lay.tiles) cellTile[t.cell_name] = [t.x, t.y];

  // Net -> cells
  const netToCells = {};
  for (const [name, cell] of Object.entries(net.cells)) {
    for (const [_, bits] of Object.entries(cell.ports)) {
      for (const b of bits) {
        if (!netToCells[b]) netToCells[b] = [];
        netToCells[b].push(name);
      }
    }
  }
  for (const [name, port] of Object.entries(net.ports)) {
    for (const b of port.bits) {
      if (!netToCells[b]) netToCells[b] = [];
      netToCells[b].push('port_' + name);
    }
  }

  // Logical connectivity overlay
  const netColors = [
    'rgba(137,180,250,0.25)', 'rgba(166,227,161,0.25)',
    'rgba(249,226,175,0.25)', 'rgba(243,139,168,0.25)',
    'rgba(203,166,247,0.25)',
  ];
  let ci = 0;
  ctx.lineWidth = 0.8 * vizState.zoom;
  for (const [_, cells] of Object.entries(netToCells)) {
    if (cells.length < 2) continue;
    const pts = [];
    for (const cn of cells) {
      const t = cellTile[cn];
      if (t) pts.push(tileCenter(t[0], t[1]));
    }
    if (pts.length < 2) continue;
    ctx.strokeStyle = netColors[ci % netColors.length];
    ci++;
    for (let i = 0; i < pts.length; i++) {
      for (let j = i + 1; j < pts.length; j++) {
        const [sx, sy] = toScreen(pts[i][0], pts[i][1], w, h);
        const [ex, ey] = toScreen(pts[j][0], pts[j][1], w, h);
        ctx.beginPath();
        ctx.moveTo(sx, sy);
        ctx.lineTo(ex, ey);
        ctx.stroke();
      }
    }
  }

  // Used tile highlights
  for (const t of lay.tiles) {
    const [cx, cy] = tileCenter(t.x, t.y);
    const [sx, sy] = toScreen(cx, cy, w, h);
    const tw = tileW * vizState.zoom * 0.8, th = tileH * vizState.zoom * 0.8;
    ctx.fillStyle = 'rgba(60,120,60,0.3)';
    ctx.strokeStyle = 'rgba(100,180,100,0.4)';
    ctx.lineWidth = 0.5;
    ctx.beginPath();
    ctx.roundRect(sx - tw/2, sy - th/2, tw, th, 2);
    ctx.fill();
    ctx.stroke();
  }
}

function renderViz() {
  const canvas = document.getElementById('viz-canvas');
  const wrap = document.getElementById('viz-wrap');
  if (!canvas || !wrap) return;
  const dpr = window.devicePixelRatio || 1;
  const rect = wrap.getBoundingClientRect();
  canvas.width = rect.width * dpr;
  canvas.height = rect.height * dpr;
  canvas.style.width = rect.width + 'px';
  canvas.style.height = rect.height + 'px';
  const ctx = canvas.getContext('2d');
  ctx.scale(dpr, dpr);
  const w = rect.width, h = rect.height;

  ctx.fillStyle = '#0d0d18';
  ctx.fillRect(0, 0, w, h);

  if (vizState.view === 'circuit') drawCircuitView(ctx, w, h);
  else if (vizState.view === 'layout') drawLayoutView(ctx, w, h);
  else if (vizState.view === 'routing') drawRoutingView(ctx, w, h);

  // Update info
  const net = vizState.netlist;
  const lay = vizState.layout;
  document.getElementById('viz-info-cells').textContent =
    net ? `Cells: ${Object.keys(net.cells).length}` : '';
  document.getElementById('viz-info-tiles').textContent =
    lay ? `${lay.device} ${lay.cols}x${lay.rows} tiles: ${lay.tiles.length}/${lay.cols*lay.rows}` : '';
  document.getElementById('viz-info-pos').textContent =
    `Zoom: ${(vizState.zoom * 100).toFixed(0)}%`;
}

// --- Viz canvas setup ---

function setupVizCanvas() {
  const canvas = document.getElementById('viz-canvas');
  const wrap = document.getElementById('viz-wrap');
  if (!canvas || !wrap) return;

  let isDragging = false, lastX = 0, lastY = 0;

  canvas.addEventListener('mousedown', (e) => {
    isDragging = true;
    lastX = e.clientX;
    lastY = e.clientY;
  });

  window.addEventListener('mousemove', (e) => {
    if (!isDragging) return;
    const dx = e.clientX - lastX;
    const dy = e.clientY - lastY;
    lastX = e.clientX;
    lastY = e.clientY;
    vizState.panX += dx;
    vizState.panY += dy;
    renderViz();
  });

  window.addEventListener('mouseup', () => { isDragging = false; });

  wrap.addEventListener('wheel', (e) => {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.1 : 0.9;
    vizState.zoom = Math.max(0.05, Math.min(10, vizState.zoom * factor));
    renderViz();
  }, { passive: false });

  // Resize
  const ro = new ResizeObserver(() => renderViz());
  ro.observe(wrap);
}

// --- Viz view switching ---

document.querySelectorAll('.viz-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.viz-btn').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    vizState.view = btn.dataset.view;
    renderViz();
  });
});

document.getElementById('btn-viz-reset')?.addEventListener('click', () => {
  vizState.panX = 0; vizState.panY = 0; vizState.zoom = 1;
  renderViz();
});

// Init canvas when PnR tab is shown
const pnrObserver = new MutationObserver(() => {
  const panel = document.getElementById('panel-verilog-pnr');
  if (panel && panel.style.display !== 'none') {
    setTimeout(setupVizCanvas, 50);
  }
});
const pnrPanel = document.getElementById('panel-verilog-pnr');
if (pnrPanel) {
  pnrObserver.observe(pnrPanel, { attributes: true, attributeFilter: ['style'] });
}
// Setup on load too
setTimeout(setupVizCanvas, 500);

// Tab switching
document.querySelectorAll('.tab').forEach(tab => {
  tab.addEventListener('click', () => {
    document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
    tab.classList.add('active');
    const tabId = tab.dataset.tab;
    document.querySelectorAll('.panel').forEach(p => p.style.display = 'none');
    const panel = document.getElementById(`panel-${tabId}`);
    if (panel) panel.style.display = 'block';
  });
});

// Button wiring
document.getElementById('btn-verilog-run').addEventListener('click', () => runVerilogSim(false));
document.getElementById('btn-verilog-clear').addEventListener('click', () => {
  clearOutput('verilog-output');
  clearOutput('verilog-generated');
});
document.getElementById('btn-pnr-run').addEventListener('click', () => runVerilogPnr(false));
document.getElementById('btn-pnr-clear').addEventListener('click', () => {
  clearOutput('pnr-output');
  clearOutput('pnr-json');
  clearOutput('pnr-asc');
  clearOutput('pnr-meta');
});
document.getElementById('btn-spice-run').addEventListener('click', () => runSpiceAnalyze(false));
document.getElementById('btn-spice-clear').addEventListener('click', () => clearOutput('spice-output'));
// Start WebSocket connection
connectWs();