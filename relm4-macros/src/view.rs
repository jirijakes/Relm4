use internal::{
    token_streams::{TokenStreams, TraitImplDetails},
    widgets::ViewWidgets,
};
use proc_macro::TokenStream;
use proc_macro2::Span as Span2;
use quote::quote;
use syn::{parse_macro_input, Ident};

pub(super) fn generate_tokens(input: TokenStream) -> TokenStream {
    let view_widgets: ViewWidgets = parse_macro_input!(input);

    let TokenStreams {
        error,
        init,
        assign,
        connect,
        ..
    } = view_widgets.generate_streams(
        &TraitImplDetails {
            vis: None,
            model_name: Ident::new("_", Span2::call_site()),
            sender_name: Ident::new("sender", Span2::call_site()),
            root_name: None,
        },
        true,
    );

    let output = quote! {
        #init
        #connect
        #assign
        {
            #error
        }
    };
    output.into()
}
