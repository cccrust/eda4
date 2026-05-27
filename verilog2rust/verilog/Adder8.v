module Adder8(a, b, cin, sum, cout);
    input [7:0] a, b;
    input cin;
    output [7:0] sum;
    output cout;
    wire c4;

    Adder4 low(.a(a[3:0]), .b(b[3:0]), .cin(cin), .sum(sum[3:0]), .cout(c4));
    Adder4 high(.a(a[7:4]), .b(b[7:4]), .cin(c4), .sum(sum[7:4]), .cout(cout));
endmodule
