extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(TemporaryComponent)]
pub fn temporary_component_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let gen = quote! {
        impl TemporaryComponent for #name {
            fn advance_timer(&mut self, duration : Duration) {
                self.timer.tick(duration);
            }

            fn is_finished(&self) -> bool {
                self.timer.finished()
            }
        }
    };

    gen.into()
}