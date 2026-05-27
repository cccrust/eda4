module Register(d, q, clk, load);
    input [3:0] d;
    output reg [3:0] q;
    input clk, load;

    always @(posedge clk) begin
        if (load)
            q <= d;
    end
endmodule
