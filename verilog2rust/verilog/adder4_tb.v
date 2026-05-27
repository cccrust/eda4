`include "FullAdder.v"
`include "Adder4.v"

module Adder4_tb;
    reg [3:0] a, b;
    reg cin;
    wire [3:0] sum;
    wire cout;

    Adder4 dut(a, b, cin, sum, cout);

    initial begin
        $display("=== Adder4 Testbench ===");
        a = 4'd3; b = 4'd5; cin = 1'b0;
        #10;
        $display("3 + 5 + 0 = %d (carry=%d)", sum, cout);
        a = 4'd9; b = 4'd7; cin = 1'b1;
        #10;
        $display("9 + 7 + 1 = %d (carry=%d)", sum, cout);
        a = 4'd0; b = 4'd15; cin = 1'b0;
        #10;
        $display("0 + 15 + 0 = %d (carry=%d)", sum, cout);
        a = 4'd15; b = 4'd15; cin = 1'b1;
        #10;
        $display("15 + 15 + 1 = %d (carry=%d)", sum, cout);
        $finish;
    end
endmodule
