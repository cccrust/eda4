module Decoder2x4(en, a, y);
    input en;
    input [1:0] a;
    output [3:0] y;
    reg [3:0] y;

    always @(*) begin
        if (!en)
            y = 4'b0000;
        else
            case (a)
                2'b00: y = 4'b0001;
                2'b01: y = 4'b0010;
                2'b10: y = 4'b0100;
                2'b11: y = 4'b1000;
            endcase
    end
endmodule
