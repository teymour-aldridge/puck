use proc_macro2::TokenStream;
use quote::quote;

use crate::handler::Handler;
use crate::handler::Segment;

/// Generate code to call the relevant route, if it matches the path in question.
pub fn route_matcher(handler: &Handler) -> TokenStream {
    let path: String = handler.at.clone();

    let segments = path
        .split('/')
        .map(|split| {
            if split.starts_with('<') {
                if split.starts_with("<int") {
                    Segment::Int
                } else if split.starts_with("<string") {
                    Segment::String
                } else {
                    unreachable!()
                }
            } else {
                Segment::Static(split.to_string())
            }
        })
        .map(|segment| match segment {
            Segment::Static(segment) => {
                quote! {
                    #segment == path
                }
            }
            Segment::Int => {
                quote! {
                    path.parse::<i32>().is_ok()
                }
            }
            Segment::String => {
                quote! {
                    true
                }
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    segments
        .into_iter()
        .enumerate()
        .map(|(pos, token_stream)| {
            quote! {{
                let path = split[#pos];
                #token_stream
            }}
        })
        .fold(quote! {}, |a, b| quote! {#a && #b})
}
