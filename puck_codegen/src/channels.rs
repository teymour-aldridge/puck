use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::handler::Handler;
use crate::sm::route_matcher;

#[derive(darling::FromMeta)]
pub struct Channel {
    name: String,
    ty: String,
}

pub struct Channels {
    items: Vec<Channel>,
}

impl Channels {
    pub fn new(items: Vec<Channel>) -> Self {
        Self { items }
    }
    pub fn emit_routes(&self, handlers: Vec<Handler>) -> Result<TokenStream, syn::Error> {
        Ok(handlers
            .into_iter()
            .map(|handler| self.call(&handler))
            .collect::<Result<Vec<TokenStream>, syn::Error>>()?
            .into_iter()
            .fold(quote! {}, |a, b| quote! {#a #b}))
    }

    fn call(&self, handler: &Handler) -> Result<TokenStream, syn::Error> {
        let collected = route_matcher(handler);

        let len = handler.at.split('/').collect::<Vec<_>>().len();
        let function = format_ident!("{}", handler.function);

        let extra_args = handler
            .receive
            .iter()
            .filter_map(|receive| {
                if self
                    .items
                    .iter()
                    .find(|channel| channel.name.to_string() == receive.to_string())
                    .is_some()
                {
                    let receive = format_ident!("receive_{}", receive);
                    Some(quote! {
                        #receive,
                    })
                } else {
                    None
                }
            })
            .fold(quote! {}, |a, b| quote! {#a, #b});

        let extra_args = handler
            .send
            .iter()
            .filter_map(|receive| {
                if self
                    .items
                    .iter()
                    .find(|channel| channel.name.to_string() == receive.to_string())
                    .is_some()
                {
                    let receive = format_ident!("send_{}", receive);
                    Some(quote! {
                        #receive
                    })
                } else {
                    None
                }
            })
            .fold(extra_args, |a, b| quote! {#a, #b});

        Ok(quote! {
            if split.len() == #len {
                if ** #collected {
                    let response = #function(request #extra_args);
                    let mut encoder = ::puck::encoder::Encoder::new(response);
                    encoder.write_tcp_stream(stream).unwrap();
                    #[allow(all)]
                    return;
                }
            }
        })
    }

    pub fn emit_tys(&self) -> TokenStream {
        self.items
            .iter()
            .map(|item| {
                let ty = format_ident!("{}", &item.ty);
                quote! {
                    ::puck::lunatic::channel::Sender<#ty>,
                    ::puck::lunatic::channel::Receiver<#ty>
                }
            })
            .fold(quote! {}, |a, b| quote! {#a #b})
    }

    pub fn emit_call_clone(&self) -> TokenStream {
        self.items
            .iter()
            .map(|item| {
                let send = format_ident!("send_{}", item.name);
                let receive = format_ident!("receive_{}", item.name);
                quote! {
                    #send.clone(),
                    #receive.clone()
                }
            })
            .fold(quote! {}, |a, b| quote! {#a #b})
    }

    pub fn emit_call(&self) -> TokenStream {
        self.items
            .iter()
            .map(|item| {
                let send = format_ident!("send_{}", item.name);
                let receive = format_ident!("receive_{}", item.name);
                quote! {
                    #send,
                    #receive
                }
            })
            .fold(quote! {}, |a, b| quote! {#a #b})
    }
}

impl ToTokens for Channels {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(
            self.items
                .iter()
                .map(|channel| {
                    let channel_name = &channel.name;
                    let ty = format_ident!("{}", channel.ty);
                    let send_name = format_ident!("send_{}", channel_name);
                    let receive_name = format_ident!("receive_{}", channel_name);
                    quote! {
                        let (#send_name, #receive_name):
                            (::puck::lunatic::channel::Sender<#ty>,
                             ::puck::lunatic::channel::Receiver<#ty>) =
                             ::puck::lunatic::channel::unbounded();
                    }
                })
                .fold(quote! {}, |a, b| quote! {#a #b}),
        );
    }
}
