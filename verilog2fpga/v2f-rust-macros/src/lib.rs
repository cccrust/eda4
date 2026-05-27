extern crate proc_macro;

use proc_macro::TokenStream;

/// `fpga!` 巨集：在 Rust 中直接撰寫硬體描述
///
/// # 語法範例
/// ```ignore
/// fpga! {
///     module Blinky(
///         input  clk: 1,
///         output led: 1
///     ) {
///         let counter = Reg::<26>(0);
///         counter.next = counter + 1;
///         led = counter[25];
///     }
/// }
/// ```
#[proc_macro]
pub fn fpga(input: TokenStream) -> TokenStream {
    let _input_str = input.to_string();
    // 第一版：輸出一個空的實作，實際 HDL 由 v2f-rust 的 builder API 處理
    // 未來版本會實際解析 token 並產生 Rust code + netlist
    let expanded = quote::quote! {
        // TODO: v2f-rust-macros 完整實作
    };
    expanded.into()
}
