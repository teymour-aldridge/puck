use darling::FromMeta;
use quote::{format_ident, quote};
use syn::{parse_macro_input, AttributeArgs, DeriveInput};

use proc_macro::TokenStream;

#[derive(darling::FromMeta)]
struct Route {
    #[darling(multiple)]
    handle: Vec<Handler>,
}

#[derive(darling::FromMeta)]
struct Handler {
    at: String,
    function: String,
}

pub enum Segment {
    Static(String),
    Int,
    String,
}

pub fn handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let args = match Route::from_list(&args) {
        Ok(t) => t,
        Err(e) => return e.write_errors().into(),
    };

    let state_machine = args
        .handle
        .into_iter()
        .map(|handler| {
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

            let function = format_ident!("{}", handler.function.clone());
            let len = segments.len();

            let collected = segments
                .into_iter()
                .enumerate()
                .map(|(pos, token_stream)| {
                    quote! {{
                        let path = split[#pos];
                        #token_stream
                    }}
                })
                .fold(quote! {}, |a, b| quote! {#a && #b});

            quote! {
                if split.len() == #len {
                    if ** #collected {
                        let response = #function(request);
                        let mut encoder = ::puck::encoder::Encoder::new(response);
                        encoder.write_tcp_stream(stream).unwrap();
                        #[allow(all)]
                        return;
                    }
                }
            }
        })
        .fold(quote! {}, |a, b| quote! {#a #b});

    let derive = parse_macro_input!(input as DeriveInput);
    let ident = derive.ident.clone();

    From::from(quote! {
        #derive

        impl ::puck::Handler for #ident {
            fn handle(stream: ::puck::lunatic::net::TcpStream) {
                let request = ::puck::Request::parse(&stream)
                    .expect("could not parse request")
                    .expect("empty request");
                let path = request.url.path();
                let split = path.split('/').collect::<Vec<_>>();
                #state_machine
                else {
                    let response = ::puck::err_404(request);
                    let mut encoder = ::puck::encoder::Encoder::new(response);
                    encoder.write_tcp_stream(stream).unwrap();
                }
            }
        }
    })
}
