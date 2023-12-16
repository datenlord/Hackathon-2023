#![allow(clippy::all)]
#![deny(
    unused_imports,
    unused_variables,
    // unused_mut,
    clippy::unnecessary_mut_passed,
    unused_results
)]

use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

fn path_is_option(path: &syn::Path) -> bool {
    path.segments.len() == 1 && path.segments[0].ident == "Option"
}

#[proc_macro_derive(LogicalModule)]
pub fn logical_module_macro_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    // ident 当前枚举名称
    let DeriveInput { ident, .. } = input;
    // 实现 comment 方法
    let output = quote! {

        impl #ident{
            pub fn new(args: LogicalModuleNewArgs) -> Self {
                let ret = Self::inner_new(args);
                // tracing::info!("new module {}", ret.name());
                ret
            }

            pub fn name() -> &'static str {
                stringify!(#ident)
            }
        }

    };
    output.into()
}
