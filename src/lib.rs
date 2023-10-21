use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn async_mock(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemTrait);
    let trait_name = input.ident.clone();
    let mock_name = format_ident!("Mock{trait_name}");
    let mut objects = Vec::new();
    let mut expectations = Vec::new();
    let mut expectation_validation = Vec::new();
    let mut functions = Vec::new();
    let mut impls = Vec::new();
    let mut counter = 0;

    for item in input.items.iter() {
        if let syn::TraitItem::Fn(f) = item {
            let mut fn_arg_types = Vec::new();
            let mut fn_arg_names = Vec::new();
            for arg in f.sig.inputs.iter() {
                match arg {
                    syn::FnArg::Receiver(_) => continue,
                    syn::FnArg::Typed(pat) => {
                        fn_arg_types.push(pat.ty.clone());
                        if let syn::Pat::Ident(ident) = pat.pat.as_ref() {
                            fn_arg_names.push(ident.ident.clone());
                        }
                    }
                }
            }

            let function_name = format_ident!("{}", f.sig.ident);
            let expect_name = format_ident!("expect_{function_name}");
            let expectation_name = format_ident!("{function_name}_expectation");
            let expectation_struct_name = format_ident!("__{mock_name}Expectation{counter}");
            let expectation_struct_name_inner =
                format_ident!("__{mock_name}ExpectationInner{counter}");
            let fn_rt = f.sig.output.clone();
            let function_signature = f.sig.clone();

            objects.push(quote! {
                #expectation_name: #expectation_struct_name
            });

            expectations.push(quote! {
                #[derive(Default)]
                pub struct #expectation_struct_name {
                    inner: std::sync::Mutex<#expectation_struct_name_inner>,
                }

                #[derive(Default)]
                pub struct #expectation_struct_name_inner {
                    expecting: u32,
                    called: u32,
                    returning: Option<fn(#(#fn_arg_types),*) #fn_rt>,
                }

                impl #expectation_struct_name {
                    pub fn once(&mut self) -> &mut Self {
                        self.inner.lock().unwrap().expecting = 1;
                        self
                    }

                    pub fn never(&mut self) -> &mut Self {
                        self.inner.lock().unwrap().expecting = 0;
                        self
                    }

                    pub fn times(&mut self, count: u32) -> &mut Self {
                        self.inner.lock().unwrap().expecting = count;
                        self
                    }

                    pub fn returning(
                        &mut self,
                        func: fn(#(#fn_arg_types),*) #fn_rt,
                    ) -> &mut Self {
                        self.inner.lock().unwrap().returning = Some(func);
                        self
                    }
                }
            });

            impls.push(quote! {
                    #function_signature {
                        let mut expectation = self.#expectation_name.inner.lock().unwrap();
                        let func = expectation.returning.expect(format!("Missing expectation for `{}`", stringify!(#function_name)).as_str());
                        expectation.called += 1;

                        func(#(#fn_arg_names),*)
                    }
                });

            expectation_validation.push(quote! {
                {
                    let expectation = self.#expectation_name.inner.lock().unwrap();
                    assert_eq!(
                        expectation.expecting,
                        expectation.called,
                        "Failed expectation for {}. Called {} times but expecting {}.",
                        stringify!(#function_name),
                        expectation.called,
                        expectation.expecting
                    );
                }
            });

            functions.push(quote! {
                pub fn #expect_name(&mut self) -> &mut #expectation_struct_name {
                    &mut self.#expectation_name
                }
            });

            counter += 1;
        };
    }

    quote! {
        #input

        #[cfg(test)]
        pub(crate) mod mocks {
            use super::*;

            #[derive(Default)]
            #[allow(dead_code)]
            pub struct #mock_name {
                #(#objects),*
            }

            impl Drop for #mock_name {
                fn drop(&mut self) {
                    #(#expectation_validation)*
                }
            }

            #(#expectations)*

            #[allow(dead_code)]
            impl #mock_name {
                #(#functions) *
            }

            #[async_trait::async_trait]
            impl #trait_name for #mock_name {
                #(#impls) *
            }
        }
    }
    .into()
}
