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
    case 'bitstream_decode_result': {
      const outEl = document.getElementById('decode-output');
      if (resp.error) {
        outEl.textContent = `Error: ${resp.error}`;
        outEl.className = 'output error';
      } else {
        outEl.textContent = `Device: ${resp.device}\nCRC Valid: ${resp.crc_valid}\nFormat: ${resp.tiles.format || 'v2f-bitdecode-v2'}\n`;
        outEl.textContent += `Decode Level: ${resp.tiles.decode_level || 'wiring'}\n`;
        outEl.textContent += `Total Tiles: ${resp.tiles.num_tiles || resp.tiles.tiles ? Object.keys(resp.tiles.tiles || {}).length : '?'}\n`;
        outEl.className = 'output success';
      }
      break;
    }
    case 'error': {
      const errorMsg = resp.error || 'Unknown error';
      ['verilog-output', 'pnr-output', 'spice-output', 'decode-output'].forEach(id => {
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

function runBitstreamDecode(_useHttp) {
  const bin = document.getElementById('decode-code').value.trim();
  if (!bin) { alert('Please paste a base64 bitstream'); return; }
  const outEl = document.getElementById('decode-output');
  outEl.textContent = 'Decoding bitstream...';
  outEl.className = 'output info';

  const req = { type: 'bitstream_decode', bin_base64: bin };
  if (wsReady) {
    sendWs(req);
  } else {
    sendHttp('POST', '/api/bitstream/decode', { bin_base64: bin }).then(handleResponse);
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

const BITSTREAM_EXAMPLES = {
  blinky_hx1k: null,
  halfadd_hx1k: null,
  mux2_hx1k: null,
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

let pendingDecodeBin = null;

document.getElementById('decode-examples').addEventListener('change', function() {
  const val = this.value;
  if (!val) return;
  this.value = '';

  if (val === 'run_pnr_blinky') {
    const code = PNR_EXAMPLES.blinky;
    const outEl = document.getElementById('pnr-output');
    outEl.textContent = 'Running PnR for decode...';
    outEl.className = 'output info';
    const req = { type: 'verilog_pnr', code, device: 'hx1k', top: 'top' };
    if (wsReady) {
      const origHandler = handleResponse;
      window._pendingDecodeHandler = (resp) => {
        if (resp.type === 'verilog_pnr_result') {
          pendingDecodeBin = resp.bin_base64;
          document.getElementById('decode-code').value = resp.bin_base64;
          const decEl = document.getElementById('decode-output');
          decEl.textContent = `PnR done. ${resp.bin_base64.length} chars loaded. Click "Decode Bitstream" to decode.`;
          decEl.className = 'output info';
          document.querySelector('[data-tab="bitstream"]').click();
        }
        handleResponse = origHandler;
      };
      handleResponse = (resp) => {
        if (resp.type === 'verilog_pnr_result' || resp.type === 'error') {
          window._pendingDecodeHandler(resp);
        } else {
          origHandler(resp);
        }
      };
      sendWs(req);
    } else {
      sendHttp('POST', '/api/verilog/pnr', { code, device: 'hx1k', top: 'top' }).then((resp) => {
        if (resp.type === 'verilog_pnr_result') {
          pendingDecodeBin = resp.bin_base64;
          document.getElementById('decode-code').value = resp.bin_base64;
          const decEl = document.getElementById('decode-output');
          decEl.textContent = `PnR done. ${resp.bin_base64.length} chars loaded. Click "Decode Bitstream" to decode.`;
          decEl.className = 'output info';
          document.querySelector('[data-tab="bitstream"]').click();
        } else {
          handleResponse(resp);
        }
      });
    }
    return;
  }

  if (val === 'run_pnr_halfadd') {
    const code = PNR_EXAMPLES.halfadd;
    const outEl = document.getElementById('pnr-output');
    outEl.textContent = 'Running PnR for decode...';
    outEl.className = 'output info';
    const req = { type: 'verilog_pnr', code, device: 'hx1k', top: 'top' };
    if (wsReady) {
      const origHandler = handleResponse;
      handleResponse = (resp) => {
        if (resp.type === 'verilog_pnr_result' || resp.type === 'error') {
          if (resp.type === 'verilog_pnr_result') {
            pendingDecodeBin = resp.bin_base64;
            document.getElementById('decode-code').value = resp.bin_base64;
            const decEl = document.getElementById('decode-output');
            decEl.textContent = `PnR done. ${resp.bin_base64.length} chars loaded. Click "Decode Bitstream" to decode.`;
            decEl.className = 'output info';
            document.querySelector('[data-tab="bitstream"]').click();
          }
          handleResponse = origHandler;
        } else {
          origHandler(resp);
        }
      };
      sendWs(req);
    } else {
      sendHttp('POST', '/api/verilog/pnr', { code, device: 'hx1k', top: 'top' }).then((resp) => {
        if (resp.type === 'verilog_pnr_result') {
          pendingDecodeBin = resp.bin_base64;
          document.getElementById('decode-code').value = resp.bin_base64;
          const decEl = document.getElementById('decode-output');
          decEl.textContent = `PnR done. ${resp.bin_base64.length} chars loaded. Click "Decode Bitstream" to decode.`;
          decEl.className = 'output info';
          document.querySelector('[data-tab="bitstream"]').click();
        } else {
          handleResponse(resp);
        }
      });
    }
    return;
  }

  const bin = BITSTREAM_EXAMPLES[val];
  if (bin) {
    document.getElementById('decode-code').value = bin;
    const decEl = document.getElementById('decode-output');
    decEl.textContent = `Example "${val}" loaded (${bin.length} chars). Click "Decode Bitstream" to decode.`;
    decEl.className = 'output info';
  }
});

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
document.getElementById('btn-decode-run').addEventListener('click', () => runBitstreamDecode(false));
document.getElementById('btn-decode-clear').addEventListener('click', () => clearOutput('decode-output'));

// Start WebSocket connection
connectWs();