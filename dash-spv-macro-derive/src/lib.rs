extern crate proc_macro;

use proc_macro::{TokenStream, TokenTree};
use syn::{parse_macro_input, AttributeArgs, Data, DeriveInput, Meta, NestedMeta, Type, PathArguments, GenericArgument, TypePtr, TypeArray, Ident, TypePath, Field};
use quote::{format_ident, quote, ToTokens};
use syn::__private::TokenStream2;

fn generate_field_conversion_code(field_name: &Option<Ident>) -> TokenStream2 {
    let field_name_str = field_name.clone().unwrap().to_string();
    let field_name_bytes = format_ident!("{}_bytes", field_name_str);
    let field_name_len = format_ident!("{}_length", field_name_str);
    quote! {
        #field_name_len: obj.#field_name.map_or(0, |v| v.len()),
        #field_name_bytes: obj.#field_name.map_or(std::ptr::null_mut(), |v| dash_spv_ffi::boxed_vec(v)),
    }
}

fn from_path(f: &Field, type_path: &TypePath, type_ptr: Option<&TypePtr>) -> Box<dyn ToTokens> {
    let field_name = &f.ident;
    if let Some(segment) = type_path.path.segments.first() {
        if segment.ident == "u8" {
            return Box::new(quote! { #field_name: FFIConversion::ffi_to(obj.#field_name) });
        } else if segment.ident == "Option" {
            println!("// from_path:::Option {:?}", type_path);
            if let PathArguments::AngleBracketed(args) = &segment.arguments {
                // Check if the inner type is Vec<u8>
                match args.args.first() {
                    Some(GenericArgument::Type(Type::Path(inner_path))) => match inner_path.path.segments.first() {
                        Some(inner_segment) if inner_segment.ident == "Vec" => {
                            // println!("from_path:::Option<Vec<T>> {:?}", type_path);
                            generate_field_conversion_code(field_name);
                        },
                        _ => {
                            panic!("from_path: Unknown field {:?} {:?}", f.ident, inner_path)
                        }
                    },
                    Some(GenericArgument::Type(Type::Ptr(inner_ptr))) => {
                        panic!("from_path: Unknown field {:?} {:?}", f.ident, inner_ptr)
                    },
                    _ => panic!("from_path: Unknown field {:?} {:?}", f.ident, args)
                }
            }
        } else if type_ptr.is_some() {
            // return Box::new(quote! { #field_name: (!ffi.#field_name.is_null()).then_some(FFIConversion::from_ffi(&*ffi.#field_name)) });
            // return Box::new(quote! { #field_name: FFIConversion::ffi_from(&*ffi.#field_name) });
            return Box::new(quote! { #field_name: FFIConversion::ffi_from((*ffi).#field_name) });
        } else {
            return Box::new(quote! { #field_name: (*ffi).#field_name });
        }
    }
    Box::new(quote! { #field_name: (*ffi).#field_name })
}

fn from_ptr(f: &Field, type_ptr: &TypePtr) -> Box<dyn ToTokens> {
    let field_name = &f.ident;
    //println!("// from_ptr::: {:?} {:?}", field_name, type_ptr);
    match &*type_ptr.elem {
        Type::Ptr(type_ptr) => from_ptr(f, type_ptr),
        Type::Path(type_path) => from_path(f, type_path, Some(type_ptr)),
        // _ => Box::new(quote! { #field_name: FFIConversion::ffi_from(&*ffi.#field_name) }),
        _ => Box::new(quote! { #field_name: FFIConversion::ffi_from((*ffi).#field_name) }),
    }
}




fn to_path(f: &Field, type_path: &TypePath, type_ptr: Option<&TypePtr>) -> Box<dyn ToTokens> {
    let field_name = &f.ident;
    if let Some(segment) = type_path.path.segments.first() {
        if segment.ident == "Option" {
            if let PathArguments::AngleBracketed(args) = &segment.arguments {
                match args.args.first() {
                    Some(GenericArgument::Type(Type::Path(inner_path))) => match inner_path.path.segments.first() {
                        Some(inner_segment) if inner_segment.ident == "Vec" => {
                            generate_field_conversion_code(field_name);
                        },
                        _ => {}
                    },
                    Some(GenericArgument::Type(Type::Ptr(_inner_ptr))) => {

                    },
                    _ => {}
                }
            }
        }
    }
    if type_ptr.is_some() {
        Box::new(quote! { #field_name: FFIConversion::ffi_to(obj.#field_name) })
    } else {
        Box::new(quote! { #field_name: obj.#field_name })
    }
}

fn to_vec_ptr(f: &Field, type_ptr: &TypePtr, type_arr: &TypeArray) -> Box<dyn ToTokens> {
    let field_name = &f.ident;
    return Box::new(quote! { #field_name: dash_spv_ffi::boxed_vec(obj.#field_name.iter().map(|o| dash_spv_ffi::boxed(FFIConversion::ffi_to(o))).collect()) });

}

fn to_ptr(f: &Field, type_ptr: &TypePtr) -> Box<dyn ToTokens> {
    //println!("// to_ptr::: {:?} {:?}", f.ident, type_ptr);
    match &*type_ptr.elem {
        Type::Array(TypeArray { elem, .. }) => match &**elem {
            Type::Path(type_path) => to_path(f, type_path, Some(type_ptr)),
            _ => panic!("to_pointer: Unknown field (arr->) {:?} {:?}", f.ident, elem),
        },
        Type::Ptr(TypePtr { elem, .. }) => match &**elem {
            Type::Path(type_path) => to_path(f, type_path, Some(type_ptr)),
            // Type::Ptr(type_ptr) => to_vec_ptr(f, type_ptr),
            Type::Array(type_arr) => to_vec_ptr(f, type_ptr, type_arr),
            _ => panic!("to_pointer: Unknown field (ptr->) {:?} {:?}", f.ident, elem),
        },
        Type::Path(type_path) => to_path(f, type_path, Some(type_ptr)),
        _ => panic!("to_pointer: Unknown field (path->) {:?} {:?}", f.ident, type_ptr.elem),
    }
}

fn to_arr(f: &Field, _type_array: &TypeArray) -> Box<dyn ToTokens> {
    let field_name = &f.ident;

    if let Type::Path(type_path) = &f.ty {
        if type_path.path.segments.last().unwrap().ident == "Option" {
            return Box::new(quote! { #field_name: if obj.#field_name.is_none() { std::ptr::null_mut() } else { dash_spv_ffi::boxed(FFIConversion::ffi_to(obj.#field_name.unwrap())) } });
        }
    }

    return Box::new(quote! { #field_name: FFIConversion::ffi_to(obj.#field_name) });
}

#[proc_macro_attribute]
pub fn ffi_conversion(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let attrs = parse_macro_input!(attr as AttributeArgs);
    let ffi_name = ast.ident.clone();
    let target_name = match attrs.first() {
        // Some(NestedMeta::Lit(literal)) => literal,
        // Some(TokenTree::Literal(literal)) => literal.to_string(),
        Some(NestedMeta::Meta(Meta::Path(path))) => path.segments.first().unwrap().ident.clone(),
        _ => panic!("Expected a single path as an argument to ffi_conversion"),
    };
    let (conversions_to_ffi, conversions_from_ffi) = match ast.data {
        Data::Struct(ref data_struct) => {
            (data_struct.fields.iter().map(|f| {
                match &f.ty {
                    Type::Ptr(type_ptr) => to_ptr(f, type_ptr),
                    Type::Array(type_array) => to_arr(f, type_array),
                    Type::Path(type_path) => to_path(f, type_path, None),
                    _ => panic!("to_struct: Unknown field {:?}", f.ident),
                }
            }).collect::<Vec<_>>(),
             data_struct.fields.iter().map(|f| {
                 let field_name = &f.ident;
                 match &f.ty {
                     Type::Ptr(type_ptr) => from_ptr(f, type_ptr),
                     Type::Path(type_path) => from_path(f, type_path, None),
                     _ => Box::new(quote! { #field_name: (*ffi).#field_name }),
                 }
             }).collect::<Vec<_>>())
        }
        _ => {
            syn::Error::new_spanned(&ffi_name, "FFIConversion can only be derived for structs")
                .to_compile_error();
            (vec![], vec![])
        }
    };
    let function_tokens = quote! {};
    let expanded = quote! {
        #ast
        #function_tokens
        impl FFIConversion<#target_name> for #ffi_name {
            unsafe fn ffi_from(ffi: *mut #ffi_name) -> #target_name {
                #target_name { #(#conversions_from_ffi,)* }
            }
            unsafe fn ffi_to(obj: #target_name) -> *mut #ffi_name {
                dash_spv_ffi::boxed(#ffi_name { #(#conversions_to_ffi,)* })
            }
        }
        impl FFIConversion<Option<#target_name>> for #ffi_name {
            unsafe fn ffi_from(ffi: *mut #ffi_name) -> Option<#target_name> {
                (!ffi.is_null()).then_some(#target_name { #(#conversions_from_ffi,)* })
            }
            unsafe fn ffi_to(obj: Option<#target_name>) -> *mut #ffi_name {
                obj.map_or(std::ptr::null_mut(), |obj| dash_spv_ffi::boxed(#ffi_name { #(#conversions_to_ffi,)* }))
            }
        }
    };
    println!("{}", expanded.to_string());
    TokenStream::from(expanded)
}