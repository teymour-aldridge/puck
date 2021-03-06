use darling::FromMeta;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, Ident};

use proc_macro::TokenStream;

use crate::channels::{Channel, Channels};

#[derive(darling::FromMeta)]
struct Route {
    #[darling(multiple)]
    handle: Vec<Handler>,
    #[darling(multiple)]
    channels: Vec<Channel>,
}

#[derive(darling::FromMeta, Clone)]
pub struct Handler {
    pub at: String,
    pub function: String,
    #[darling(default, multiple)]
    pub receive: Vec<Ident>,
    #[darling(default, multiple)]
    pub send: Vec<Ident>,
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

    let channels = Channels::new(args.channels);

    let derive = parse_macro_input!(input as DeriveInput);
    let ident = derive.ident.clone();

    let tys = channels.emit_tys();

    let call = channels.emit_call();

    let call_clone = channels.emit_call_clone();

    let routes = channels.emit_routes(args.handle);

    let res: TokenStream = From::from(quote! {
        #derive
        fn __inner_request_handler((stream, #call): (::puck::lunatic::net::TcpStream, #tys)) {
            let request = ::puck::Request::parse(&stream)
                    .expect("could not parse request")
                    .expect("empty request");
                let path = request.url.path();
                let split = path.split('/').collect::<Vec<_>>();
                #routes
                else {
                    let response = ::puck::err_404(request);
                    let mut encoder = ::puck::encoder::Encoder::new(response);
                    encoder.write_tcp_stream(stream).unwrap();
                }
        }

        impl ::puck::Handler for #ident {
            fn handle(addr: &'static str) -> ::puck::anyhow::Result<()> {
                let conn = ::puck::lunatic::net::TcpListener::bind(addr)?;
                #channels
                while let Ok(stream) = conn.accept() {
                    ::puck::lunatic::Process::spawn_with((stream, #call_clone), __inner_request_handler)
                        .detach();
                }
                Ok(())
            }
        }
    });
    res
}
