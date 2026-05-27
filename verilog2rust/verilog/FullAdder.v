module FullAdder(a, b, cin, sum, cout);
    input a, b, cin;
    output sum, cout;
    wire s, c1, c2;

    xor u1(s, a, b);
    xor u2(sum, s, cin);
    and u3(c1, a, b);
    and u4(c2, s, cin);
    or u5(cout, c1, c2);
endmodule
