use proc_macro::TokenStream;

mod handler;

#[proc_macro_attribute]
pub fn handler(args: TokenStream, input: TokenStream) -> TokenStream {
    handler::handler(args, input)
}
