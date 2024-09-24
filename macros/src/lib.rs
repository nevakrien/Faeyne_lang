extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn define_ids(input: TokenStream) -> TokenStream {
    let ids: Vec<LitStr> = parse_macro_input!(input with syn::punctuated::Punctuated::<LitStr, syn::Token![,]>::parse_terminated).into_iter().collect();
    
    let const_defs = ids.iter().enumerate().map(|(idx, id)| {
        let id_name = id.value().replace(":", "").to_uppercase() + "_ID";
        let id_ident = syn::Ident::new(&id_name, id.span());
        quote! {
            pub const #id_ident: usize = #idx;
        }
    });

    let preload_statements = ids.iter().enumerate().map(|(_idx, id)| {
        let id_name = id.value().replace(":", "").to_uppercase() + "_ID";
        let id_ident = syn::Ident::new(&id_name, id.span());
        quote! {
            assert_eq!(table.get_id(#id), #id_ident);
        }
    });

    let get_id_arms = ids.iter().map(|id| {
        let id_name = id.value().replace(":", "").to_uppercase() + "_ID";
        let id_ident = syn::Ident::new(&id_name, id.span());
        quote! {
            (#id) => { #id_ident };
        }
    });

    let expanded = quote! {
        #(#const_defs)*

        pub fn preload_table(table: &mut StringTable) {
            #(#preload_statements)*
        }

        #[macro_export]
        macro_rules! get_id {
            #(#get_id_arms)*
            ($other:expr) => { $other };
        }
    };

    TokenStream::from(expanded)
}
