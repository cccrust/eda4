`include "ALU.v"

module ALU_tb;
    reg [3:0] a, b;
    reg [1:0] op;
    wire [3:0] result;
    wire zero;

    ALU dut(a, b, op, result, zero);

    initial begin
        $display("=== ALU Testbench ===");
        $display(" op |   a    b  | result  zero");
        $display("----+----------+---------");

        // ADD
        a = 4'd3; b = 4'd5; op = 2'b00;
        #10;
        $display(" ADD | %2d  + %2d |    %2d    %d", a, b, result, zero);

        a = 4'd9; b = 4'd7; op = 2'b00;
        #10;
        $display(" ADD | %2d  + %2d |    %2d    %d", a, b, result, zero);

        a = 4'd0; b = 4'd0; op = 2'b00;
        #10;
        $display(" ADD | %2d  + %2d |    %2d    %d", a, b, result, zero);

        // SUB
        a = 4'd8; b = 4'd3; op = 2'b01;
        #10;
        $display(" SUB | %2d  - %2d |    %2d    %d", a, b, result, zero);

        a = 4'd3; b = 4'd8; op = 2'b01;
        #10;
        $display(" SUB | %2d  - %2d |    %2d    %d", a, b, result, zero);

        // AND
        a = 4'b1100; b = 4'b1010; op = 2'b10;
        #10;
        $display(" AND | %2d  & %2d |    %2d    %d", a, b, result, zero);

        // OR
        a = 4'b1100; b = 4'b0011; op = 2'b11;
        #10;
        $display(" OR  | %2d  | %2d |    %2d    %d", a, b, result, zero);

        a = 4'd0; b = 4'd0; op = 2'b11;
        #10;
        $display(" OR  | %2d  | %2d |    %2d    %d", a, b, result, zero);

        $finish;
    end
endmodule
