use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::Ident;

use crate::widgets::{PropertyName, ReturnedWidget, Widget, WidgetTemplateAttr};

impl ReturnedWidget {
    fn return_assign_tokens(&self) -> TokenStream2 {
        let name = &self.name;

        if let Some(ty) = &self.ty {
            quote! {
                let #name : #ty
            }
        } else {
            quote! {
                let #name
            }
        }
    }
}

impl Widget {
    pub(crate) fn start_assign_stream(&self, stream: &mut TokenStream2) {
        let w_name = &self.name;
        self.properties.assign_stream(stream, w_name, false);
    }

    pub(super) fn assign_stream(
        &self,
        stream: &mut TokenStream2,
        p_name: &PropertyName,
        w_name: &Ident,
        is_conditional: bool,
    ) {
        // Template children are already assigned by the template.
        if self.template_attr != WidgetTemplateAttr::TemplateChild {
            let assign_fn = p_name.assign_fn_stream(w_name);
            let self_assign_args = p_name.assign_args_stream(w_name);
            let assign = self.widget_assignment();
            let span = p_name.span();

            let args = self.args.as_ref().map(|args| {
                quote_spanned! {
                   args.span() => ,#args
                }
            });

            stream.extend(if let Some(ret_widget) = &self.returned_widget {
                let return_assign_stream = ret_widget.return_assign_tokens();
                let unwrap = ret_widget.is_optional.then(|| quote! { .unwrap() });
                quote_spanned! {
                    span => #return_assign_stream = #assign_fn(#self_assign_args #assign #args) #unwrap;
                }
            } else {
                quote_spanned! {
                    span => #assign_fn(#self_assign_args #assign #args);
                }
            });
        }

        // Recursively generate code for properties
        let w_name = &self.name;
        self.properties
            .assign_stream(stream, w_name, is_conditional);

        if let Some(returned_widget) = &self.returned_widget {
            let w_name = &returned_widget.name;
            returned_widget
                .properties
                .assign_stream(stream, w_name, is_conditional);
        }
    }
}
