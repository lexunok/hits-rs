use proc_macro::TokenStream;
use quote::quote;
use syn::{
    FnArg, Ident, ItemFn, Pat, Type, parse_macro_input, punctuated::Punctuated, token::Comma,
};

fn find_claims_arg(input: &mut ItemFn) -> syn::Result<(Ident, Box<Type>)> {
    let claims_arg = input.sig.inputs.iter_mut().find_map(|arg| {
        if let FnArg::Typed(pat_type) = arg {
            if let Type::Path(type_path) = &*pat_type.ty {
                if type_path
                    .path
                    .segments
                    .last()
                    .map_or(false, |segment| segment.ident == "Claims")
                {
                    return Some(pat_type);
                }
            }
        }
        None
    });

    if let Some(pat_type) = claims_arg {
        if let Pat::Ident(pat_ident) = &*pat_type.pat {
            Ok((pat_ident.ident.clone(), pat_type.ty.clone()))
        } else {
            Err(syn::Error::new_spanned(
                pat_type,
                "Expected ident pattern for Claims argument",
            ))
        }
    } else {
        Err(syn::Error::new_spanned(
            input.sig.ident.clone(),
            "Handler function must have a `claims: Claims` argument",
        ))
    }
}

#[proc_macro_attribute]
pub fn has_role(attr: TokenStream, item: TokenStream) -> TokenStream {
    let required_role = parse_macro_input!(attr as Ident); // e.g., Admin
    let mut input = parse_macro_input!(item as ItemFn);

    let (claims_ident, _) = match find_claims_arg(&mut input) {
        Ok(args) => args,
        Err(e) => return e.to_compile_error().into(),
    };

    let check_code = quote! {
        if !#claims_ident.roles.contains(&entity::role::Role::#required_role) {
            return Err(crate::error::AppError::Forbidden);
        }
    };

    let check_code_as_tokenstream: TokenStream = check_code.into();
    input.block.stmts.insert(
        0,
        parse_macro_input!(check_code_as_tokenstream as syn::Stmt),
    );

    quote! { #input }.into()
}

#[proc_macro_attribute]
pub fn has_any_role(attr: TokenStream, item: TokenStream) -> TokenStream {
    let required_roles = parse_macro_input!(attr with Punctuated::<Ident, Comma>::parse_terminated);
    let mut input = parse_macro_input!(item as ItemFn);

    let (claims_ident, _) = match find_claims_arg(&mut input) {
        Ok(args) => args,
        Err(e) => return e.to_compile_error().into(),
    };

    let roles_check = required_roles.iter().map(|role| {
        quote! { #claims_ident.roles.contains(&entity::role::Role::#role) }
    });

    let check_code = quote! {
        let has_any_role = #(#roles_check)||*;
        if !has_any_role {
            return Err(crate::error::AppError::Forbidden);
        }
    };

    let check_code_as_tokenstream: TokenStream = check_code.into();
    input.block.stmts.insert(
        0,
        parse_macro_input!(check_code_as_tokenstream as syn::Stmt),
    );

    quote! { #input }.into()
}

#[proc_macro_attribute]
pub fn has_all_roles(attr: TokenStream, item: TokenStream) -> TokenStream {
    let required_roles = parse_macro_input!(attr with Punctuated::<Ident, Comma>::parse_terminated);
    let mut input = parse_macro_input!(item as ItemFn);

    let (claims_ident, _) = match find_claims_arg(&mut input) {
        Ok(args) => args,
        Err(e) => return e.to_compile_error().into(),
    };

    let roles_check = required_roles.iter().map(|role| {
        quote! { #claims_ident.roles.contains(&entity::role::Role::#role) }
    });

    let check_code = quote! {
        let has_all_roles = #(#roles_check)&&*;
        if !has_all_roles {
            return Err(crate::error::AppError::Forbidden);
        }
    };

    let check_code_as_tokenstream: TokenStream = check_code.into();
    input.block.stmts.insert(
        0,
        parse_macro_input!(check_code_as_tokenstream as syn::Stmt),
    );

    quote! { #input }.into()
}


#[proc_macro_derive(IntoDataResponse)]
pub fn into_data_response_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    let name = &ast.ident;
    let generated_impl = quote! {
        impl axum::response::IntoResponse for #name {
            fn into_response(self) -> axum::response::Response {
                axum::Json(self).into_response()
            }
        }
    };
    generated_impl.into()
}
