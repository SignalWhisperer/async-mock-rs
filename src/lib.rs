use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse_macro_input;

#[allow(dead_code)]
fn print_tokens(tokens: &dyn ToTokens) {
    println!("{}", tokens.to_token_stream());
}

#[allow(dead_code)]
fn print_tokens_dbg(tokens: &dyn ToTokens) {
    println!("{:?}", tokens.to_token_stream());
}

fn contains_impl(token: &syn::Type) -> bool {
    match token {
        syn::Type::ImplTrait(_) => true,
        syn::Type::Group(group) => contains_impl(group.elem.as_ref()),
        syn::Type::Paren(paren) => contains_impl(paren.elem.as_ref()),
        syn::Type::Reference(reference) => contains_impl(reference.elem.as_ref()),
        _ => false,
    }
}

fn convert_impl_to_dyn(token: &syn::Type) -> syn::Type {
    match &token {
        syn::Type::ImplTrait(impl_trait) => syn::Type::TraitObject(syn::TypeTraitObject {
            dyn_token: Some(syn::token::Dyn::default()),
            bounds: impl_trait.bounds.clone(),
        }),
        syn::Type::Group(group) => syn::Type::Group(syn::TypeGroup {
            group_token: group.group_token,
            elem: Box::new(convert_impl_to_dyn(group.elem.as_ref())),
        }),
        syn::Type::Paren(paren) => syn::Type::Paren(syn::TypeParen {
            paren_token: paren.paren_token,
            elem: Box::new(convert_impl_to_dyn(paren.elem.as_ref())),
        }),
        syn::Type::Reference(reference) => syn::Type::Reference(syn::TypeReference {
            and_token: reference.and_token,
            lifetime: reference.lifetime.clone(),
            mutability: reference.mutability,
            elem: Box::new(convert_impl_to_dyn(reference.elem.as_ref())),
        }),
        _ => token.clone(),
    }
}

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
            let mut fn_arg_types_dyn = Vec::new();
            let mut fn_arg_names = Vec::new();
            let mut has_impl_ref = false;

            for arg in f.sig.inputs.iter() {
                if let syn::FnArg::Typed(pat) = arg {
                    if let syn::Pat::Ident(ident) = pat.pat.as_ref() {
                        fn_arg_names.push(ident.ident.clone());
                    }

                    has_impl_ref |= contains_impl(pat.ty.as_ref());
                    fn_arg_types.push(pat.ty.clone());
                    fn_arg_types_dyn.push(convert_impl_to_dyn(pat.ty.as_ref()));
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

            let fn_storage_type = if has_impl_ref {
                quote! { Box<dyn Fn(#(#fn_arg_types_dyn),*) #fn_rt + Send + Sync> }
            } else {
                quote! { fn(#(#fn_arg_types_dyn),*) #fn_rt }
            };

            objects.push(quote! {
                #expectation_name: #expectation_struct_name
            });

            let returning_fn_name = if has_impl_ref {
                format_ident!("returning_dyn")
            } else {
                format_ident!("returning")
            };

            expectations.push(quote! {
                #[cfg(test)]
                #[derive(Default)]
                pub struct #expectation_struct_name {
                    inner: std::sync::Mutex<#expectation_struct_name_inner>,
                }

                #[cfg(test)]
                #[derive(Default)]
                pub struct #expectation_struct_name_inner {
                    expecting: u32,
                    called: u32,
                    returning: Option<#fn_storage_type>,
                }

                #[cfg(test)]
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

                    pub fn #returning_fn_name(
                        &mut self,
                        func: #fn_storage_type,
                    ) -> &mut Self {
                        self.inner.lock().unwrap().returning = Some(func);
                        self
                    }
                }
            });

            let get_mutex_expectation = quote! {
                let expectation = self.#expectation_name.inner.lock();
                assert!(expectation.is_ok(), "Poisoned inner mocking state for `{}`.", stringify!(#function_name));
                let mut expectation = expectation.unwrap();
            };

            let func_call_with_drop = if has_impl_ref {
                quote! {
                    let func = expectation.returning.as_ref();

                    if let Some(func) = func {
                        func(#(#fn_arg_names),*)
                    } else {
                        drop(expectation);
                        panic!("Missing returning function for `{}`", stringify!(#function_name));
                    }
                }
            } else {
                quote! {
                    let func = expectation.returning;

                    if let Some(func) = &func {
                        func(#(#fn_arg_names),*)
                    } else {
                        drop(expectation);
                        panic!("Missing returning function for `{}`", stringify!(#function_name));
                    }
                }
            };

            impls.push(quote! {
                #function_signature {
                    #get_mutex_expectation

                    expectation.called += 1;

                    #func_call_with_drop
                }
            });

            expectation_validation.push(quote! {
                {
                    #get_mutex_expectation

                    let expecting = expectation.expecting;
                    let called = expectation.called;

                    drop(expectation);

                    if !std::thread::panicking() {
                        assert_eq!(
                            expecting,
                            called,
                            "Failed expectation for `{}`. Called {} times but expecting {}.",
                            stringify!(#function_name),
                            called,
                            expecting
                        );
                    }
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

    let code = quote! {
        #input

        #[cfg(test)]
        #[derive(Default)]
        #[allow(dead_code)]
        pub struct #mock_name {
            #(#objects),*
        }

        #[cfg(test)]
        impl Drop for #mock_name {
            fn drop(&mut self) {
                #(#expectation_validation)*
            }
        }

        #(#expectations)*

        #[cfg(test)]
        #[allow(dead_code)]
        impl #mock_name {
            #(#functions) *

            pub fn new() -> Self {
                Self::default()
            }
        }

        #[cfg(test)]
        #[async_trait::async_trait] // TODO: Only add this if it was used on the trait
        impl #trait_name for #mock_name {
            #(#impls) *
        }
    };

    // print_tokens(&code);

    code.into()
}
