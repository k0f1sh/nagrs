use proc_macro;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Data, DataStruct, Fields};

#[proc_macro_derive(NagiosCmd)]
pub fn nagios_cmd_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_nagios_cmd_macro(&ast)
}

fn impl_nagios_cmd_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let upper_snake = &ast.ident.to_string().to_case(Case::UpperSnake);

    let fields = match &ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };
    let field_name = fields.iter().map(|field| &field.ident);

    let gen = quote! {
        impl NagiosCmd for #name {
            fn to_cmd_string(&self) -> String {
                let mut command_string: String = #upper_snake.to_string();
                #(
                    command_string.push_str(format!(";{}", self.#field_name).as_str());
                )*
                command_string
            }
        }
    };
    gen.into()
}
