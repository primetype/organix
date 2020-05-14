extern crate proc_macro;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod ast;
mod gen;
mod symbol;

#[proc_macro_derive(Organix, attributes(runtime))]
#[proc_macro_error]
pub fn derive_organix(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ast = ast::Input::from_syn(&input).unwrap();
    let gen = gen::gen(ast);
    gen.into()
}

#[proc_macro_derive(IntercomMsg)]
pub fn derive_intercom_msg(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let gen = quote! {
        impl organix::service::IntercomMsg for #name {}
    };
    gen.into()
}
