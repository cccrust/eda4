module cpu(clock);
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
endmodule
