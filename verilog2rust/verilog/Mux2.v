module Mux2(a, b, sel, y);
    input a, b, sel;
    output y;
    wire s, not_sel, t1, t2;

    not u1(not_sel, sel);
    and u2(t1, a, not_sel);
    and u3(t2, b, sel);
    or u4(y, t1, t2);
endmodule
