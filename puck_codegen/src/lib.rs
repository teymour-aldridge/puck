#[macro_use]
extern crate syn;

use proc_macro::TokenStream;

mod channels;
mod handler;
mod sm;

#[proc_macro_attribute]
pub fn handler(args: TokenStream, input: TokenStream) -> TokenStream {
    handler::handler(args, input)
}
