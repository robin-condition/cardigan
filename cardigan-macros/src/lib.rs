use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::{ToTokens, format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, token::{And, Comma}, FnArg, ItemFn, ItemImpl, ItemStruct, PatType, Receiver, Token
};

#[proc_macro_attribute]
pub fn memoized(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut input_item: ItemFn = parse_macro_input!(input);

    let name = input_item.sig.ident.to_string();
    let comp_id = format_ident!("{}", name);
    let input_count = input_item.sig.inputs.len();

    let ret_type = match &input_item.sig.output {
        syn::ReturnType::Default => parse_quote!(()),
        syn::ReturnType::Type(_, b) => *b.clone(),
    };

    let internal_name_ident = format_ident!("__internal_{}", name);

    input_item.sig.ident = internal_name_ident.clone();

    eprintln!("{}", "Help I'm trapped in a code factory");

    let inp_names: Vec<_> = input_item.sig.inputs.iter().map(|f| match f {
        FnArg::Typed(p) => match &*p.pat {
            syn::Pat::Ident(id) => id.ident.clone(),
            _ => panic!(),
        },
        _ => panic!()
    }).collect();

    let transformed_inputs: Vec<FnArg> = input_item
        .sig
        .inputs
        .iter()
        .map(|f| match f {
            FnArg::Typed(p) => {
                let typ = *p.ty.clone();
                let id = match &*p.pat {
                    syn::Pat::Ident(id) => id.ident.clone(),
                    _ => panic!(),
                };

                parse_quote!(
                    #id : &::cardigan_incremental::Versioned<#typ>
                )
            }
            _ => panic!(),
        })
        .collect();

    let mut wrapper_fn_args: Punctuated<FnArg, Comma> = Punctuated::new();
    wrapper_fn_args.push(FnArg::Receiver(parse_quote!(&mut self)));
    for inp in transformed_inputs {
        wrapper_fn_args.push(inp);
    }

    let versioned_comp_struct: ItemStruct = parse_quote! {
        #[derive(Default)]
        pub struct #comp_id {
            input_versions: ::cardigan_incremental::VersionedComputationInfo<#input_count>,
            output_value: ::cardigan_incremental::Versioned<#ret_type>
        }
    };

    let implementation: ItemImpl = parse_quote!(
        impl #comp_id {

            async fn internal_recomp(#wrapper_fn_args) -> ::std::option::Option<#ret_type> {
                Some((#internal_name_ident (#((* #inp_names .get_value())?),*)).await)
            }


            pub async fn compute(#wrapper_fn_args) -> &::cardigan_incremental::Versioned<#ret_type> {
                let versions = [#(* #inp_names .version()),*];
                if !self.input_versions.must_recompute(&versions) {
                    return &self.output_value;
                }

                let recomped = self.internal_recomp(#(#inp_names),*).await;

                self.output_value.set_to_next(recomped);
                self.input_versions.update(&versions);
                return &self.output_value;
            }
        }
    );

    /*let output: proc_macro2::TokenStream = {
        /* transform input */
    };

    proc_macro::TokenStream::from(output)
    */
    return quote! {
        #versioned_comp_struct

        #input_item

        #implementation
    }.into()
}
