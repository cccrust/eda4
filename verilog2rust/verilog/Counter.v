module Counter(clk, rst, en, q);
    input clk, rst, en;
    output reg [7:0] q;

    always @(posedge clk or posedge rst) begin
        if (rst)
            q <= 8'b00000000;
        else if (en)
            q <= q + 1;
    end
endmodule
