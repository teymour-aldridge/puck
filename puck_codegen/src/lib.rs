use darling::FromMeta;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, AttributeArgs, DeriveInput};

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

#[proc_macro_attribute]
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
            let path = handler.at;
            let function = format_ident!("{}", handler.function);
            quote! {
                if path == #path {
                    let response = #function(request);
                    let mut encoder = ::puck::encoder::Encoder::new(response);
                    encoder.write_tcp_stream(stream).unwrap();
                    #[allow(all)]
                    return;
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
