extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, Data, DeriveInput, ItemFn, Meta, NestedMeta, Type, PathArguments, GenericArgument, TypePtr, TypeArray, Ident, TypePath, Field, DataStruct, Fields, FieldsUnnamed, FieldsNamed, DataEnum, Variant, Expr, Path};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::__private::TokenStream2;
use syn::spanned::Spanned;

enum ConversionType {
    Simple,
    Complex,
    Map,
    Vec
}

// impl ConversionType {
//     fn impl_from_vec(&self) -> TokenStream2 {
//         match self {
//             Self::Simple =>
//         }
//     }
// }

fn package() -> TokenStream2 {
    quote!(dash_spv_ffi)
}

fn interface() -> TokenStream2 {
    quote!(FFIConv)
}

fn ffi() -> TokenStream2 {
    quote!(ffi)
}

fn ffi_deref() -> TokenStream2 {
    let ffi = ffi();
    quote!(*#ffi)
}

fn obj() -> TokenStream2 {
    quote!(obj)
}

fn ffi_from() -> TokenStream2 {
    quote!(ffi_from)
}

fn ffi_from_opt() -> TokenStream2 {
    quote!(ffi_from_opt)
}

fn ffi_to() -> TokenStream2 {
    quote!(ffi_to)
}

fn ffi_to_opt() -> TokenStream2 {
    quote!(ffi_to_opt)
}

fn boxed() -> TokenStream2 {
    quote!(boxed)
}

fn boxed_vec() -> TokenStream2 {
    quote!(boxed_vec)

}

fn package_boxed() -> TokenStream2 {
    let package = package();
    let boxed = boxed();
    quote!(#package::#boxed)
}

fn package_boxed_expression(expr: TokenStream2) -> TokenStream2 {
    let package_boxed = package_boxed();
    quote!(#package_boxed(#expr))
}

fn package_boxed_vec() -> TokenStream2 {
    let package = package();
    let boxed_vec = boxed_vec();
    quote!(#package::#boxed_vec)
}

fn package_boxed_vec_expression(expr: TokenStream2) -> TokenStream2 {
    let package_boxed_vec = package_boxed_vec();
    quote!(#package_boxed_vec(#expr))
}

fn iter_map_collect(iter: TokenStream2, mapper: TokenStream2) -> TokenStream2 {
    quote!(#iter.map(#mapper).collect())
}

fn define_field(l_value: TokenStream2, r_value: TokenStream2) -> TokenStream2 {
    quote!(#l_value: #r_value)
}

fn define_lambda(l_value: TokenStream2, r_value: TokenStream2) -> TokenStream2 {
    quote!(#l_value => #r_value)
}

fn unwrap_or(field_path: TokenStream2, or: TokenStream2) -> TokenStream2 {
    quote!(#field_path.unwrap_or(#or))
}

fn ffi_deref_field_name(field_name: TokenStream2) -> TokenStream2 {
    let ffi_deref = ffi_deref();
    quote!((#ffi_deref).#field_name)
}

fn obj_field_name(field_name: TokenStream2) -> TokenStream2 {
    let obj = obj();
    quote!(#obj.#field_name)
}

fn create_struct(name: TokenStream2, fields: Vec<Box<dyn ToTokens>>) -> TokenStream2 {
    quote! {
        #[repr(C)]
        #[derive(Clone, Copy, Debug)]
        pub struct #name { #(#fields,)* }
    }
}


fn ffi_to_map_conversion(map_key_path: TokenStream2, key_index: TokenStream2, key_conversion: TokenStream2, value_conversion: TokenStream2) -> TokenStream2 {
    let keys_conversion = package_boxed_vec_expression(quote!(#map_key_path.keys().cloned().map(|#key_index| #key_conversion).collect()));
    let values_conversion = package_boxed_vec_expression(quote!(#map_key_path.values().cloned().map(|#key_index| #value_conversion).collect()));
    package_boxed_expression(quote! {{
        dash_spv_ffi::MapFFI {
            count: #map_key_path.len(),
            keys: #keys_conversion,
            values: #values_conversion,
        }
    }})
}

fn ffi_from_map_conversion(map_key_path: TokenStream2, key_index: TokenStream2, acc_type: TokenStream2, key_conversion: TokenStream2, value_conversion: TokenStream2) -> TokenStream2 {
    quote! {{
        let map = *#map_key_path;
        (0..map.count).into_iter().fold(#acc_type::new(), |mut acc, #key_index| {
            let key = #key_conversion;
            let value = #value_conversion;
            acc.insert(key, value);
            acc
        })
    }}
}

fn ffi_from_vec_conversion(vec_key_path: TokenStream2, key_index: TokenStream2, value_conversion: TokenStream2) -> TokenStream2 {
    quote! {{
        (0..*#vec_key_path.count).into_iter().map(|#key_index| #value_conversion).collect()
    }}
}

fn from_vec(path: &Path, field_name: &Ident) -> TokenStream2 {
    let last_segment = path.segments.last().unwrap();

    let ffi_field_name_quote = ffi_deref_field_name(quote!(#field_name));
    match &last_segment.arguments {
        PathArguments::AngleBracketed(args) => {
            let args = &args.args.iter().collect::<Vec<_>>();
            match &args[..] {
                [GenericArgument::Type(Type::Path(inner_path))] => {
                    let path = &inner_path.path;
                    let key_index = quote!(i);
                    let ffi_deref = ffi_deref();
                    let simple_conversion = |buffer: TokenStream2| quote!(#buffer.add(#key_index));
                    let value_simple_conversion = simple_conversion(quote!(*vec.values));
                    let field = quote!((#ffi_deref).#field_name);
                    let field_type = &path.segments.last().unwrap().ident;

                    println!("from_vec.2: {:?} {:?}", field_name, field);
                    match conversion_type_for_path(path) {
                        ConversionType::Simple => quote!({
                            let vec = *#field;
                            std::slice::from_raw_parts(vec.values as *const #field_type, vec.count).to_vec()
                        }),
                        ConversionType::Complex => {
                            let ffi_from_conversion = ffi_from_conversion(quote!(#field.values.add(i)));
                            let count = quote!(#field.count);
                            iter_map_collect(quote!((0..#count).into_iter()), quote!(|i| #ffi_from_conversion))
                        },
                        ConversionType::Map => panic!("from_vec (Map): Unknown field {:?} {:?}", field_name, args),
                        ConversionType::Vec => panic!("from_vec (Vec): Unknown field {:?} {:?}", field_name, args)
                    }
                },
                _ => panic!("from_path (vec): Unknown field {:?} {:?}", field_name, args)
            }
        },
        _ => ffi_from_conversion(ffi_field_name_quote)
    }
}

fn from_map(path: &Path, field_name: &Ident) -> TokenStream2 {
    let last_segment = path.segments.last().unwrap();
    let field_type = &last_segment.ident;
    match &last_segment.arguments {
        PathArguments::AngleBracketed(args) => {
            let args = &args.args.iter().collect::<Vec<_>>();
            match &args[..] {
                [GenericArgument::Type(Type::Path(inner_path_key)), GenericArgument::Type(Type::Path(inner_path_value))] => {
                    let key_index = quote!(i);
                    let ffi_deref = ffi_deref();
                    let simple_conversion = |buffer: TokenStream2| quote!(#buffer.add(#key_index));
                    let key_simple_conversion = simple_conversion(quote!(*map.keys));
                    let value_simple_conversion = simple_conversion(quote!(*map.values));

                    let key_conversion = match conversion_type_for_path(&inner_path_key.path) {
                        ConversionType::Simple => key_simple_conversion,
                        ConversionType::Complex => ffi_from_conversion(key_simple_conversion),
                        ConversionType::Map | ConversionType::Vec => panic!("Vec/Map not supported as Map key")
                    };
                    let value_conversion = match conversion_type_for_path(&inner_path_value.path) {
                        ConversionType::Simple => value_simple_conversion,
                        ConversionType::Complex => ffi_from_conversion(value_simple_conversion),
                        ConversionType::Vec => {
                            let path = &inner_path_value.path;
                            let last_segment = path.segments.last().unwrap();
                            let field_type = &last_segment.ident;
                            match &last_segment.arguments {
                                PathArguments::AngleBracketed(args) => {
                                    let args = &args.args.iter().collect::<Vec<_>>();
                                    match &args[..] {
                                        [GenericArgument::Type(Type::Path(inner_path_value))] => {
                                            let key_index = quote!(i);
                                            let simple_conversion = |buffer: TokenStream2| quote!(#buffer.add(#key_index));
                                            let value_simple_conversion = simple_conversion(quote!(*map.values));

                                            let value_conversion = match conversion_type_for_path(&inner_path_value.path) {
                                                ConversionType::Simple => value_simple_conversion,
                                                ConversionType::Complex => ffi_from_conversion(value_simple_conversion),
                                                _ => panic!("3 Nested Map/Vec not supported yet")
                                            };
                                            let ccc = simple_conversion(quote!(map.values));
                                            ffi_from_vec_conversion(quote!(((*#ccc))), key_index, value_conversion)
                                        },
                                        _ => panic!("from_path (map): Unknown field {:?} {:?}", field_name, args)
                                    }
                                },
                                _ => ffi_from_conversion(ffi_deref_field_name(quote!(#field_name)))
                            }
                        },
                        ConversionType::Map => {
                            let path = &inner_path_value.path;
                            let last_segment = path.segments.last().unwrap();
                            let field_type = &last_segment.ident;
                            match &last_segment.arguments {
                                PathArguments::AngleBracketed(args) => {
                                    let args = &args.args.iter().collect::<Vec<_>>();
                                    match &args[..] {
                                        [GenericArgument::Type(Type::Path(inner_path_key)), GenericArgument::Type(Type::Path(inner_path_value))] => {
                                            let key_index = quote!(i);
                                            let simple_conversion = |buffer: TokenStream2| quote!(#buffer.add(#key_index));
                                            let key_simple_conversion = simple_conversion(quote!(*map.keys));
                                            let value_simple_conversion = simple_conversion(quote!(*map.values));

                                            let key_conversion = match conversion_type_for_path(&inner_path_key.path) {
                                                ConversionType::Simple => key_simple_conversion,
                                                ConversionType::Complex  => ffi_from_conversion(key_simple_conversion),
                                                ConversionType::Vec | ConversionType::Map => panic!("Vec/Map not supported as Map key")
                                            };
                                            let value_conversion = match conversion_type_for_path(&inner_path_value.path) {
                                                ConversionType::Simple => value_simple_conversion,
                                                ConversionType::Complex => ffi_from_conversion(value_simple_conversion),
                                                _ => panic!("3 Nested Map/Vec not supported yet")
                                            };
                                            let ccc = simple_conversion(quote!(map.values));
                                            ffi_from_map_conversion(quote!(((*#ccc))), key_index, quote!(#field_type), key_conversion, value_conversion)
                                        },
                                        _ => panic!("from_path (map): Unknown field {:?} {:?}", field_name, args)
                                    }
                                },
                                _ => ffi_from_conversion(ffi_deref_field_name(quote!(#field_name)))
                            }
                        }
                    };
                    ffi_from_map_conversion(quote!((#ffi_deref).#field_name), key_index, quote!(#field_type), key_conversion, value_conversion)
                },
                _ => panic!("from_path (map): Unknown field {:?} {:?}", field_name, args)
            }
        },
        _ => ffi_from_conversion(ffi_deref_field_name(quote!(#field_name)))
    }
}

fn from_option(path: &Path, field_name: &Ident) -> TokenStream2 {
    let last_segment = path.segments.last().unwrap();
    let ffi_field_name_quote = ffi_deref_field_name(quote!(#field_name));
    match &last_segment.arguments {
        PathArguments::AngleBracketed(args) => {
            // Check if the inner type is Vec<u8>
            match args.args.first() {
                Some(GenericArgument::Type(Type::Path(inner_path))) => {
                    let path = &inner_path.path;
                    match path.segments.first() {
                        Some(inner_segment) => {
                            match inner_segment.ident.to_string().as_str() {
                                // std convertible
                                // TODO: what to use? 0 or ::MAX
                                "i8" | "u8" | "i16" | "u16" |
                                "i32" | "u32" | "i64" | "u64" |
                                "i128" | "u128" | "isize" | "usize" => quote!((#ffi_field_name_quote > 0).then_some(#ffi_field_name_quote)),
                                // TODO: mmm shit
                                "bool" => quote!((#ffi_field_name_quote > 0).then_some(#ffi_field_name_quote)),
                                "Vec" => {
                                    let conversion = from_vec(path, field_name);
                                    quote!((!#ffi_field_name_quote.is_null()).then_some(#conversion))
                                    // match obj.script {
                                    //     Some(vec) => dash_spv_ffi::boxed(dash_spv_ffi::VecFFI::new(vec)),
                                    //     None => std::ptr::null_mut()
                                    // }
                                },
                                _ => ffi_from_opt_conversion(ffi_field_name_quote)
                            }
                        },
                        _ => panic!("from_path: (type->path) Unknown field {:?} {:?}", field_name, inner_path)
                    }
                },
                _ => panic!("from_path: Unknown field {:?} {:?}", field_name, args)
            }
        },
        _ => panic!("from_path: Unknown field {:?}", field_name)
    }
}

fn from_path(f: &Field, type_path: &TypePath, _type_ptr: Option<&TypePtr>) -> TokenStream2 {
    let path = &type_path.path;
    let last_segment = path.segments.last().unwrap();
    let field_name = &f.ident.clone().unwrap();
    let field_value = ffi_deref_field_name(quote!(#field_name));
    match last_segment.ident.to_string().as_str() {
        "i8" | "u8" | "i16" | "u16" |
        "i32" | "u32" | "i64" | "u64" |
        "i128" | "u128" | "isize" | "usize" | "bool" => field_value,
        "VarInt" => quote!(#path(#field_value)),
        "Option" => from_option(path, field_name),
        "Vec" => from_vec(path, field_name),
        "BTreeMap" | "HashMap" => from_map(path, field_name),
        _ => ffi_from_conversion(field_value)
    }
}

fn from_ptr(f: &Field, type_ptr: &TypePtr) -> TokenStream2 {
    let field_name = &f.ident;
    match &*type_ptr.elem {
        Type::Ptr(type_ptr) => from_ptr(f, type_ptr),
        Type::Path(type_path) => from_path(f, type_path, Some(type_ptr)),
        _ => ffi_from_conversion(ffi_deref_field_name(quote!(#field_name)))
    }
}

fn define_vec(field_name: &Ident, arguments: &PathArguments) -> TokenStream2 {
    println!("define_vec.1: {:?} {:?}", field_name, arguments);
    let field_name_quote = quote!(#field_name);
    let obj_field_name_quote = obj_field_name(field_name_quote.clone());
    match arguments {
        PathArguments::AngleBracketed(args) => {
            let args = &args.args.iter().collect::<Vec<_>>();
            match &args[..] {
                [GenericArgument::Type(Type::Path(inner_path))] => {
                    let vec = quote!(vec);
                    let transformer = |path: &Path| {
                        match conversion_type_for_path(path) {
                            ConversionType::Simple => {
                                quote!(#vec.clone())
                            },
                            ConversionType::Complex => {
                                let conv = ffi_to_conversion(quote!(o));
                                iter_map_collect(quote!(#obj_field_name_quote.into_iter()),  quote!(|o| #conv))
                            },
                            ConversionType::Map => panic!("define_vec: Map nested in Vec not supported yet"),
                            ConversionType::Vec => panic!("define_vec: Vec nested in Vec not supported yet"),
                        }
                    };
                    let conversion = package_boxed_vec_expression(transformer(&inner_path.path));
                    println!("define_vec.2: {:?} {:?}", field_name, conversion);
                    define_field(field_name_quote, package_boxed_expression(quote! {{
                        let #vec = #obj_field_name_quote;
                        dash_spv_ffi::VecFFI { count: #vec.len(), values: #conversion }
                    }}))
                },
                // _ => define_field(field_name_quote,obj_field_name_quote)
                _ => panic!("define_vec: bad args {:?}", args)
            }
        },
        _ => panic!("define_vec: bad arguments {:?}", arguments)
        // _ => define_field(field_name_quote,obj_field_name_quote)
    }
}

fn to_vec(path: &Path, field_name: &Ident) -> TokenStream2 {
    println!("to_vec.1: {:?} {:?}", field_name, path);
    let obj = obj();
    let vec = quote!(vec);
    let transformer = match conversion_type_for_path(path) {
        ConversionType::Simple => {
            quote!(clone())
        },
        ConversionType::Complex => {
            let mapper = package_boxed_expression(ffi_to_conversion(quote!(o)));
            quote!(iter().map(|o| #mapper).collect())
        },
        ConversionType::Map => panic!("Map nested in Vec not supported yet"),
        ConversionType::Vec => panic!("Vec nested in Vec not supported yet"),
    };
    let conversion = package_boxed_vec_expression(quote!(#vec.#transformer));
    package_boxed_expression(quote! {{
        let #vec = #obj.#field_name;
        dash_spv_ffi::VecFFI { count: #vec.len(), values: #conversion }
    }})
}

fn define_map(field_name: &Ident, arguments: &PathArguments) -> TokenStream2 {
    let field_name_quote = quote!(#field_name);
    let obj_field_name_quote = obj_field_name(field_name_quote.clone());
    match arguments {
        PathArguments::AngleBracketed(args) => {
            let args = &args.args.iter().collect::<Vec<_>>();
            match &args[..] {
                [GenericArgument::Type(Type::Path(inner_path_key)), GenericArgument::Type(Type::Path(inner_path_value))] => {
                    let create_transformer = |path: &Path| {
                        let conversion = match conversion_type_for_path(path) {
                            ConversionType::Simple => quote!(o),
                            ConversionType::Complex => ffi_to_conversion(quote!(o)),
                            ConversionType::Vec => {
                                let last_segment = path.segments.last().unwrap();
                                match &last_segment.arguments {
                                    PathArguments::AngleBracketed(args) => {
                                        let args = &args.args.iter().collect::<Vec<_>>();
                                        match &args[..] {
                                            [GenericArgument::Type(Type::Path(inner_path))] => {
                                                let values_converter = match conversion_type_for_path(&inner_path.path) {
                                                    ConversionType::Simple => quote!(|o| o),
                                                    ConversionType::Complex => {
                                                        let mapper = ffi_to_conversion(quote!(o));
                                                        quote!(|o| #mapper)
                                                    },
                                                    ConversionType::Vec | ConversionType::Map => panic!("Don't support wrapping triple nested Map/Vec")
                                                };
                                                let values_conversion = package_boxed_vec_expression(quote!(o.values().cloned().map(#values_converter).collect()));
                                                package_boxed_expression(quote!({dash_spv_ffi::VecFFI { count: o.len(), values: #values_conversion }}))
                                            },
                                            _ => panic!("to_path (nested_map): Unknown field {:?} {:?}", field_name, args)
                                        }
                                    },
                                    _ => panic!("define_map (nested_vec): Unknown field {:?} {:?}", field_name, args)
                                }
                            },
                            ConversionType::Map => {
                                let last_segment = path.segments.last().unwrap();
                                match &last_segment.arguments {
                                    PathArguments::AngleBracketed(args) => {
                                        let args = &args.args.iter().collect::<Vec<_>>();
                                        match &args[..] {
                                            [GenericArgument::Type(Type::Path(inner_path_key)), GenericArgument::Type(Type::Path(inner_path_value))] => {
                                                let keys_converter = match conversion_type_for_path(&inner_path_key.path) {
                                                    ConversionType::Simple => quote!(|o| o),
                                                    ConversionType::Complex => {
                                                        let mapper = ffi_to_conversion(quote!(o));
                                                        quote!(|o| #mapper)
                                                    },
                                                    ConversionType::Vec | ConversionType::Map => panic!("Don't support wrapping triple nested Map/Vec")
                                                };
                                                let values_converter = match conversion_type_for_path(&inner_path_value.path) {
                                                    ConversionType::Simple => quote!(|o| o),
                                                    ConversionType::Complex => {
                                                        let mapper = ffi_to_conversion(quote!(o));
                                                        quote!(|o| #mapper)
                                                    },
                                                    ConversionType::Vec | ConversionType::Map => panic!("Don't support wrapping triple nested Map/Vec")
                                                };
                                                let keys_conversion = package_boxed_vec_expression(quote!(o.keys().cloned().map(#keys_converter).collect()));
                                                let values_conversion = package_boxed_vec_expression(quote!(o.values().cloned().map(#values_converter).collect()));
                                                package_boxed_expression(quote!({dash_spv_ffi::MapFFI { count: o.len(), keys: #keys_conversion, values: #values_conversion }}))
                                            },
                                            _ => panic!("to_path (nested_map): Unknown field {:?} {:?}", field_name, args)
                                        }
                                    }
                                    _ => panic!("to_path (nested_map): Unknown field {:?} {:?}", field_name, args)
                                }
                            }
                        };
                        quote!(|o| #conversion)
                    };
                    let keys_converter = create_transformer(&inner_path_key.path);
                    let values_converter = create_transformer(&inner_path_value.path);
                    let keys_conversion = package_boxed_vec_expression(quote!(#obj_field_name_quote.keys().cloned().map(#keys_converter).collect()));
                    let values_conversion = package_boxed_vec_expression(quote!(#obj_field_name_quote.values().cloned().map(#values_converter).collect()));

                    define_field(field_name_quote, package_boxed_expression(quote!({dash_spv_ffi::MapFFI { count: #obj_field_name_quote.len(), keys: #keys_conversion, values: #values_conversion }})))
                },
                _ => panic!("to_path (map): Unknown field {:?} {:?}", field_name, args)
            }
        },
        _ => define_field(field_name_quote,obj_field_name_quote)
    }
}

fn define_option(field_name: &Ident, arguments: &PathArguments) -> TokenStream2 {
    let field_name_quote = quote!(#field_name);
    let obj_field_name_quote = obj_field_name(field_name_quote.clone());
    match arguments {
        PathArguments::AngleBracketed(args) => {
            match args.args.first() {
                Some(GenericArgument::Type(Type::Path(inner_path))) => {
                    let path = &inner_path.path;
                    let last_segment = path.segments.last().unwrap();
                    match last_segment.ident.to_string().as_str() {
                        "i8" | "u8" | "i16" | "u16" |
                        "i32" | "u32" | "i64" | "u64" |
                        "i128" | "u128" | "isize" | "usize" =>
                        // TODO: MAX/MIN? use optional primitive?
                            define_field(field_name_quote, unwrap_or(obj_field_name_quote, quote!(0))),
                        "bool" => define_field(field_name_quote, unwrap_or(obj_field_name_quote, quote!(false))),
                        "Vec" => {
                            // let conversion = to_vec(path, field_name);

                            match &last_segment.arguments {
                                PathArguments::AngleBracketed(args) => match &args.args.iter().collect::<Vec<_>>()[..] {
                                    [GenericArgument::Type(Type::Path(type_values))] => {
                                        println!("to_opt_vec.1: {:?} {:?}", field_name, path);
                                        let obj = obj();
                                        let vec = quote!(vec);
                                        let transformer = match conversion_type_for_path(&type_values.path) {
                                            ConversionType::Simple => {
                                                quote!(clone())
                                            },
                                            ConversionType::Complex => {
                                                let mapper = package_boxed_expression(ffi_to_conversion(quote!(o)));
                                                quote!(iter().map(|o| #mapper).collect())
                                            },
                                            ConversionType::Map => panic!("define_option: Map nested in Vec not supported yet"),
                                            ConversionType::Vec => panic!("define_option: Vec nested in Vec not supported yet"),
                                        };
                                        let conversion = package_boxed_expression(quote!(dash_spv_ffi::VecFFI::new(#vec.#transformer)));
                                        define_field(quote!(#field_name), quote! {
                                match #obj.#field_name {
                                    Some(vec) => #conversion,
                                    None => std::ptr::null_mut()
                                }
                            })

                                    },
                                    _ => panic!("convert_vec_arg_type: bad args {:?}", last_segment)
                                },
                                _ => panic!("convert_vec_arg_type: Unknown args {:?}", last_segment)
                            }
                        },
                        _ => define_field(field_name_quote, ffi_to_opt_conversion(obj_field_name_quote))
                    }
                },
                _ => define_field(field_name_quote,obj_field_name_quote)
            }
        },
        _ => define_field(field_name_quote,obj_field_name_quote)
    }
}

fn to_path(f: &Field, type_path: &TypePath, _type_ptr: Option<&TypePtr>) -> TokenStream2 {
    let field_name = &f.ident.clone().unwrap();
    let field_name_quote = quote!(#field_name);
    let obj_field_name_quote = obj_field_name(field_name_quote.clone());
    let path = &type_path.path;
    let last_segment = path.segments.last().unwrap();
    match last_segment.ident.to_string().as_str() {
        "i8" | "u8" | "i16" | "u16" |
        "i32" | "u32" | "i64" | "u64" |
        "i128" | "u128" | "isize" | "usize" | "bool" => define_field(field_name_quote, obj_field_name_quote),
        "VarInt" => define_field(field_name_quote, quote!(#obj_field_name_quote.0)),
        "Vec" => define_vec(field_name, &last_segment.arguments),
        "BTreeMap" | "HashMap" => define_map(field_name, &last_segment.arguments),
        "Option" => define_option(field_name, &last_segment.arguments),
        _ => define_field( field_name_quote, ffi_to_conversion(obj_field_name_quote))
    }
}

fn to_vec_ptr(f: &Field, _type_ptr: &TypePtr, _type_arr: &TypeArray) -> TokenStream2 {
    let field_name = &f.ident;
    let expr = package_boxed_expression(quote!(o));
    define_field(quote!(#field_name), package_boxed_vec_expression(iter_map_collect(obj_field_name(quote!(#field_name)), quote!(|o| #expr))))
}

fn ffi_from_conversion(field_value: TokenStream2) -> TokenStream2 {
    let package = package();
    let interface = interface();
    let ffi_from = ffi_from();
    quote!(#package::#interface::#ffi_from(#field_value))
}

fn ffi_to_conversion(field_value: TokenStream2) -> TokenStream2 {
    let package = package();
    let interface = interface();
    let ffi_to = ffi_to();
    quote!(#package::#interface::#ffi_to(#field_value))
}

fn ffi_from_opt_conversion(field_value: TokenStream2) -> TokenStream2 {
    let package = package();
    let interface = interface();
    let ffi_from_opt = ffi_from_opt();
    quote!(#package::#interface::#ffi_from_opt(#field_value))
}

fn ffi_to_opt_conversion(field_value: TokenStream2) -> TokenStream2 {
    let package = package();
    let interface = interface();
    let ffi_to_opt = ffi_to_opt();
    quote!(#package::#interface::#ffi_to_opt(#field_value))
}

fn to_field(field_name: &Ident) -> TokenStream2 {
    define_field(quote!(#field_name), ffi_to_conversion(obj_field_name(quote!(#field_name))))
}

fn to_opt_field(field_name: &Ident) -> TokenStream2 {
    define_field(quote!(#field_name), ffi_to_opt_conversion(obj_field_name(quote!(#field_name))))
}

fn to_ptr(f: &Field, type_ptr: &TypePtr) -> TokenStream2 {
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

fn to_arr(f: &Field, _type_array: &TypeArray) -> TokenStream2 {
    let field_name = &f.ident.clone().unwrap();
    if let Type::Path(type_path) = &f.ty {
        if type_path.path.segments.last().unwrap().ident == "Option" {
            return to_opt_field(field_name)
        }
    }
    to_field(field_name)
}

fn conversion_type_for_path(path: &Path) -> ConversionType {
    let last_segment = path.segments.last().unwrap();
    match last_segment.ident.to_string().as_str() {
        // std convertible
        "i8" | "u8" | "i16" | "u16" |
        "i32" | "u32" | "i64" | "u64" |
        "i128" | "u128" | "isize" | "usize" | "bool" => ConversionType::Simple,
        "BTreeMap" | "HashMap" => ConversionType::Map,
        "Vec" => ConversionType::Vec,
        _ => ConversionType::Complex
    }

}

fn should_use_direct_conversion(path: &Path) -> bool {
    let last_segment = path.segments.last().unwrap();
    match last_segment.ident.to_string().as_str() {
        // std convertible
        "i8" | "u8" | "i16" | "u16" |
        "i32" | "u32" | "i64" | "u64" |
        "i128" | "u128" | "isize" | "usize" | "bool" => true,
        _ => false
    }
}

fn convert_path_to_field_type(path: &Path) -> TokenStream2 {
    let mut cloned_segments = path.segments.clone();
    let last_segment = cloned_segments.iter_mut().last().unwrap();
    let field_type = &last_segment.ident;
    println!("convert_path_to_field_type: {:?}", field_type.to_string().as_str());
    match field_type.to_string().as_str() {
        // std convertible
        "i8" | "u8" | "i16" | "u16" |
        "i32" | "u32" | "i64" | "u64" |
        "i128" | "u128" | "isize" | "usize" | "bool" => quote!(#field_type),
        "String" => quote!(*mut std::os::raw::c_char),
        "UInt128" => quote!(*mut [u8; 16]),
        "UInt160" => quote!(*mut [u8; 20]),
        "UInt256" => quote!(*mut [u8; 32]),
        "UInt384" => quote!(*mut [u8; 48]),
        "UInt512" => quote!(*mut [u8; 64]),
        "UInt768" => quote!(*mut [u8; 96]),
        "VarInt" => quote!(u64),
        _ => {
            last_segment.ident = Ident::new(&format!("{}FFI", last_segment.ident), last_segment.ident.span());
            let field_type = cloned_segments.into_iter().map(|segment| quote_spanned! { segment.span() => #segment }).collect::<Vec<_>>();
            let full_path = quote!(#(#field_type)::*);
            quote!(*mut #full_path)
        }
    }
}

// fn convert_path_to_field_type_2(path: &Path) -> TokenStream2 {
//     let mut cloned_segments = path.segments.clone();
//     let last_segment = cloned_segments.iter_mut().last().unwrap();
//     let field_type = &last_segment.ident;
//     match field_type.to_string().as_str() {
//         // std convertible
//         "i8" | "u8" | "i16" | "u16" |
//         "i32" | "u32" | "i64" | "u64" |
//         "i128" | "u128" | "isize" | "usize" | "bool" => quote!(#field_type),
//         "String" => quote!(std::os::raw::c_char),
//         "UInt128" => quote!([u8; 16]),
//         "UInt160" => quote!([u8; 20]),
//         "UInt256" => quote!([u8; 32]),
//         "UInt384" => quote!([u8; 48]),
//         "UInt512" => quote!([u8; 64]),
//         "UInt768" => quote!([u8; 96]),
//         "VarInt" => quote!(u64),
//         _ => {
//             last_segment.ident = Ident::new(&format!("{}FFI", last_segment.ident), last_segment.ident.span());
//             let field_type = cloned_segments.into_iter().map(|segment| quote_spanned! {segment.span() => #segment}).collect::<Vec<_>>();
//             let full_path = quote!(#(#field_type)::*);
//             quote!(#full_path)
//         }
//     }
// }

fn ffi_struct_name(field_type: &Ident) -> Ident {
    format_ident!("{}FFI", field_type)
}

fn convert_struct_to_var(field_name: &Ident, path: &Path) -> TokenStream2 {
    define_field(quote!(pub #field_name), convert_path_to_field_type(path))
}

fn convert_map_arg_type(path: &Path) -> TokenStream2 {
    let last_segment = path.segments.last().unwrap();
    match last_segment.ident.to_string().as_str() {
        "BTreeMap" | "HashMap" => match &last_segment.arguments {
            PathArguments::AngleBracketed(args) => match &args.args.iter().collect::<Vec<_>>()[..] {
                [GenericArgument::Type(Type::Path(type_keys)), GenericArgument::Type(Type::Path(type_values))] => {
                    let key_type = convert_map_arg_type(&type_keys.path);
                    let value_type = convert_map_arg_type(&type_values.path);
                    quote!(*mut dash_spv_ffi::MapFFI<#key_type, #value_type>)
                },
                _ => panic!("convert_map_arg_type: bad args {:?}", last_segment)
            },
            _ => panic!("convert_map_arg_type: Unknown args {:?}", last_segment)
        },
        _ => convert_path_to_field_type(path)
    }
}

fn convert_vec_arg_type(path: &Path) -> TokenStream2 {
    let last_segment = path.segments.last().unwrap();
    println!("convert_vec_arg_type: {:?}", last_segment.ident.to_string().as_str());
    match last_segment.ident.to_string().as_str() {
        "Vec" => match &last_segment.arguments {
            PathArguments::AngleBracketed(args) => match &args.args.iter().collect::<Vec<_>>()[..] {
                [GenericArgument::Type(Type::Path(type_values))] => {
                    let value_type = convert_vec_arg_type(&type_values.path);
                    println!("convert_vec_arg_type.2: {:?} ====> {:?}", type_values, value_type);
                    quote!(*mut dash_spv_ffi::VecFFI<#value_type>)
                },
                _ => panic!("convert_vec_arg_type: bad args {:?}", last_segment)
            },
            _ => panic!("convert_vec_arg_type: Unknown args {:?}", last_segment)
        },
        _ => convert_path_to_field_type(path)
    }
}

fn convert_map_to_var(field_name: &Ident, path_keys: &Path, path_values: &Path) -> TokenStream2 {
    println!("convert_map_to_var.1: {:?} ===> {:?}, {:?}", field_name, path_keys, path_values);
    let converted_field_value_keys = convert_map_arg_type(path_keys);
    let converted_field_value_values = convert_map_arg_type(path_values);
    println!("convert_map_to_var.2: {:?} ===> {:?}, {:?}", field_name, converted_field_value_keys, converted_field_value_values);
    quote!(pub #field_name: *mut dash_spv_ffi::MapFFI<#converted_field_value_keys, #converted_field_value_values>)
}

// fn convert_vec_to_var(field: &Field) -> TokenStream2 {
fn convert_vec_to_var(field_name: &Ident, path: &Path) -> TokenStream2 {
    match &path.segments.last().unwrap().arguments {
        PathArguments::AngleBracketed(args) => {
            match &args.args.iter().collect::<Vec<_>>()[..] {
                [GenericArgument::Type(Type::Path(inner_path))] => {
                    let path = &inner_path.path;
                    println!("convert_vec_to_var.1: {:?} ===> {:?}", field_name, path);
                    let converted_field_value_values = convert_vec_arg_type(path);
                    println!("convert_vec_to_var.2: {:?} ===> {:?}", field_name, converted_field_value_values);
                    quote!(pub #field_name: *mut dash_spv_ffi::VecFFI<#converted_field_value_values>)
                }
                _ => panic!("from_path: Unknown field {:?} {:?}", field_name, args)
            }
        }
        _ => panic!("from_path: Unknown field {:?}", field_name)
    }

    // let segments = &path.segments;
    // let last_segment = segments.last().unwrap();
    // let converted_type = match &last_segment.arguments {
    //     PathArguments::AngleBracketed(args) => {
    //         match args.args.first() {
    //             Some(GenericArgument::Type(Type::Path(inner_path))) => {
    //                 let field = convert_path_to_field_type(&inner_path.path);
    //                 quote!(*mut #field)
    //             },
    //             _ => panic!("Can't convert_vec_to_var (first arg) {:?} {:?}", field_name, path)
    //         }
    //     },
    //     _ => panic!("Can't convert_vec_to_var (arguments) {:?} {:?}", field_name, path)
    // };
    // let field_name_count = format_ident!("{}_count", field_name);
    // let count_definition = define_field(quote!(pub #field_name_count), quote!(usize));
    // let type_definition = define_field(quote!(pub #field_name), converted_type);
    // quote!(#count_definition, #type_definition)
}

fn convert_path_arguments(field_name: &Ident, path_args: &PathArguments) -> TokenStream2 {
    // let field_name = &field.ident.clone().unwrap();
    match &path_args {
        PathArguments::AngleBracketed(args) => {
            match &args.args.iter().collect::<Vec<_>>()[..] {
                [GenericArgument::Type(Type::Path(type_keys)), GenericArgument::Type(Type::Path(type_values))] =>
                    convert_map_to_var(field_name, &type_keys.path, &type_values.path),
                [GenericArgument::Type(Type::Path(inner_path))] =>
                    convert_struct_to_var(field_name, &inner_path.path),
                _ => panic!("from_path: Unknown field {:?} {:?}", field_name, args)
            }
        }
        _ => panic!("from_path: Unknown field {:?}", field_name)
    }
}

fn extract_struct_field(f: &Field) -> Box<dyn ToTokens> {
    let field_name = &f.ident.clone().unwrap();
    let field_type = &f.ty;
    Box::new(match field_type {
        // Type::Array(_type_arr) => convert_vec_to_var(field_name, _type_arr),
        Type::Path(type_path) => {
            let path = &type_path.path;
            let segment = path.segments.last().unwrap();
            let ident = &segment.ident;
            let args = &segment.arguments;
            match ident.to_string().as_str() {
                "Vec" => convert_vec_to_var(field_name, path),
                "BTreeMap" | "HashMap" => convert_path_arguments(field_name, args),
                "Option" => {
                    match &args {
                        PathArguments::AngleBracketed(args) => {
                            match &args.args.iter().collect::<Vec<_>>()[..] {
                                [GenericArgument::Type(Type::Path(inner_path))] => {
                                    let unwrapped_path = &inner_path.path;
                                    let unwrapped_segment = unwrapped_path.segments.last().unwrap();
                                    let unwrapped_ident = &unwrapped_segment.ident;
                                    let unwrapped_args = &unwrapped_segment.arguments;
                                    match unwrapped_ident.to_string().as_str() {
                                        "Vec" => convert_vec_to_var(field_name, unwrapped_path),
                                        "BTreeMap" | "HashMap" => convert_path_arguments(field_name, unwrapped_args),
                                        _ => convert_struct_to_var(field_name, unwrapped_path)
                                    }
                                },
                                _ => panic!("from_path: Unknown field {:?} {:?}", field_name, args)
                            }
                        }
                        _ => panic!("from_path: Unknown field {:?}", field_name)
                    }
                },
                _ => convert_struct_to_var(field_name, path),
            }
        },
        _ => panic!("Can't extract struct field")
    })
}

fn impl_interface(ffi_name: TokenStream2, target_name: TokenStream2, ffi_from_conversion: TokenStream2, ffi_to_conversion: TokenStream2) -> TokenStream2 {
    let package = package();
    let interface = interface();
    let ffi = ffi();
    let obj = obj();
    let ffi_from = ffi_from();
    let ffi_to = ffi_to();
    let ffi_from_opt = ffi_from_opt();
    let ffi_to_opt = ffi_to_opt();
    quote! {
        impl #package::#interface<#target_name> for #ffi_name {
            unsafe fn #ffi_from(#ffi: *mut #ffi_name) -> #target_name { #ffi_from_conversion }
            unsafe fn #ffi_to(#obj: #target_name) -> *mut #ffi_name { #ffi_to_conversion }
            unsafe fn #ffi_from_opt(#ffi: *mut #ffi_name) -> Option<#target_name> {
                (!#ffi.is_null()).then_some(<Self as #package::#interface<#target_name>>::#ffi_from(#ffi))
            }
            unsafe fn #ffi_to_opt(#obj: Option<#target_name>) -> *mut #ffi_name {
                #obj.map_or(std::ptr::null_mut(), |o| <Self as #package::#interface<#target_name>>::#ffi_to(o))
            }
        }
    }
}

fn from_unnamed_struct(fields: &FieldsUnnamed, target_name: Ident, input: &DeriveInput) -> TokenStream {
    let obj = obj();
    let ffi_deref = ffi_deref();
    let interface_impl = match fields.unnamed.first().unwrap().ty.clone() {
        Type::Path(ffi_name) => {
            impl_interface(
                quote!(#ffi_name),
                quote!(#target_name),
                quote!(#target_name(#ffi_deref.0)),
                package_boxed_expression(quote!(#ffi_name(#obj.0))),
            )
        },
        Type::Array(ffi_name) => {
            impl_interface(
                quote!(#ffi_name),
                quote!(#target_name),
                quote!(#target_name(#ffi_deref)),
                package_boxed_expression(quote!(#obj.0)),
            )
        },
        _ => panic!("Expected array type")
    };
    let expanded = quote! {
        #input
        #interface_impl
    };
    //println!("{}", expanded);
    TokenStream::from(expanded)
}

fn from_named_struct(fields: &FieldsNamed, target_name: Ident, input: &DeriveInput) -> TokenStream {
    let ffi_name = input.ident.clone();
    let conversions_to_ffi = fields.named.iter().map(|f| match &f.ty {
        Type::Ptr(type_ptr) => to_ptr(f, type_ptr),
        Type::Array(type_array) => to_arr(f, type_array),
        Type::Path(type_path) => to_path(f, type_path, None),
        _ => panic!("from_named_struct: Unknown field {:?}", f.ident),
    }).collect::<Vec<_>>();
    let conversions_from_ffi = fields.named.iter().map(|f| {
        let field_name = &f.ident;
        Box::new(define_field(quote!(#field_name),match &f.ty {
            Type::Ptr(type_ptr) => from_ptr(f, type_ptr),
            Type::Path(type_path) => from_path(f, type_path, None),
            _ => ffi_deref_field_name(quote!(#field_name)),
        }))
    }).collect::<Vec<_>>();
    let struct_fields = fields.named.iter().map(|f| extract_struct_field(f)).collect::<Vec<_>>();
    let ffi_name = ffi_struct_name(&ffi_name);
    let ffi_struct = create_struct(quote!(#ffi_name), struct_fields);
    let interface_impl = impl_interface(
        quote!(#ffi_name),
        quote!(#target_name),
        quote!(#target_name { #(#conversions_from_ffi,)* }),
        package_boxed_expression(quote!(#ffi_name { #(#conversions_to_ffi,)* })),
    );
    let expanded = quote! {
        #input
        #ffi_struct
        #interface_impl
    };
    println!("{}", expanded);
    TokenStream::from(expanded)
}

fn from_enum_variant(variant: &Variant, _index: usize) -> TokenStream2 {
    let variant_name = &variant.ident;
    match &variant.discriminant {
        Some((_, Expr::Lit(lit, ..))) => quote!(#variant_name = #lit),
        None => {
            let extract_field_value_type = |field: &Field| {
                let field_type = &field.ty;
                match field_type {
                    Type::Array(_type_arr) => quote!(*mut *mut #field_type, usize),
                    Type::Path(type_path) => {
                        let path = &type_path.path;
                        let segment = path.segments.last().unwrap();
                        let field_type = &segment.ident;
                        let args = &segment.arguments;
                        match field_type.to_string().as_str() {
                            "Vec" => quote!(*mut *mut #field_type, usize),
                            "BTreeMap" | "HashMap" => {
                                match &args {
                                    PathArguments::AngleBracketed(args) => {
                                        match &args.args.iter().collect::<Vec<_>>()[..] {
                                            // BTreeMap / HashMap
                                            [GenericArgument::Type(Type::Path(type_keys)), GenericArgument::Type(Type::Path(type_values))] => {
                                                let field_value_keys = &type_keys.path.segments.last().unwrap().ident;
                                                let field_value_values = &type_values.path.segments.last().unwrap().ident;
                                                quote!(*mut dash_spv_ffi::MapFFI<#field_value_keys, #field_value_values>)
                                                // quote!(*mut *mut #field_value_keys, *mut *mut #field_value_values, usize)
                                            },
                                            // [GenericArgument::Type(Type::Path(inner_path))] => convert_path_to_field_type(&inner_path.path),
                                            _ => panic!("from_enum_variant: Unknown field {:?}", args)
                                        }
                                    }
                                    _ => panic!("from_enum_variant: Unknown field")
                                }

                            },
                            "Option" => {
                                match &args {
                                    PathArguments::AngleBracketed(args) => {
                                        let args = &args.args.iter().collect::<Vec<_>>();
                                        match &args[..] {
                                            [GenericArgument::Type(Type::Path(inner_path))] => convert_path_to_field_type(&inner_path.path),
                                            _ => panic!("from_enum_variant: Unknown field {:?}", args)
                                        }
                                    }
                                    _ => panic!("from_enum_variant: Unknown field")
                                }
                            },
                            _ => {
                                let converted_type = convert_path_to_field_type(path);
                                quote!(#variant_name(#converted_type))
                            }
                        }
                    },
                    _ => panic!("from_enum_variant: Can't extract struct field")
                }
            };
            let enum_fields = match &variant.fields {
                Fields::Named(ref fields) => fields.named.iter().map(|f| Box::new(extract_field_value_type(f))).collect::<Vec<_>>(),
                Fields::Unnamed(ref fields) => fields.unnamed.iter().map(|f| Box::new(extract_field_value_type(f))).collect::<Vec<_>>(),
                Fields::Unit => vec![Box::new(quote!(#variant_name))],
            };
            quote!(#(#enum_fields)*)
        },
        _ => panic!("Error variant discriminant")
    }
}

fn from_enum(data_enum: &DataEnum, target_name: Ident, input: &DeriveInput) -> TokenStream {
    let variants = &data_enum.variants;
    let variant_fields = variants
        .iter()
        .enumerate()
        .map(|(index, variant)| from_enum_variant(variant, index))
        .collect::<Vec<_>>();
    let ffi_name = ffi_struct_name(&target_name);

    let converted = quote! {
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Hash, Ord)]
        pub enum #ffi_name {
            #(#variant_fields,)*
        }
    };
    let to_ffi_arms = data_enum.variants.iter().map(|v| {
        let ident = &v.ident;
        match &v.fields {
            Fields::Unnamed(fields) => {
                let field = &fields.unnamed.first().unwrap();
                if let Type::Path(type_path) = &field.ty {
                    let obj = obj();
                    let conversion = if should_use_direct_conversion(&type_path.path) {
                        obj.clone()
                    } else {
                        ffi_to_conversion(obj.clone())
                    };
                    define_lambda(quote!(#target_name::#ident(#obj)), quote!(#ffi_name::#ident(#conversion)))
                } else {
                    panic!("Unsupported field type in enum variant");
                }
            },
            Fields::Unit => define_lambda(quote!(#target_name::#ident), quote!(#ffi_name::#ident)),
            _ => panic!("Unsupported fields in enum variant"),
        }
    });

    let from_ffi_arms = data_enum.variants.iter().map(|v| {
        let ident = &v.ident;
        match &v.fields {
            Fields::Unnamed(fields) => {
                let mut variant_fields = vec![];
                let mut converted_fields = vec![];
                fields.unnamed.iter().enumerate().for_each(|(index, field)| {
                        if let Type::Path(type_path) = &field.ty {
                            let field_indexed = format_ident!("o_{}", index);
                            variant_fields.push(quote!(#field_indexed));
                            converted_fields.push(if should_use_direct_conversion(&type_path.path) {
                                quote!(#field_indexed)
                            } else {
                                ffi_from_conversion(quote!(#field_indexed)) });
                        } else {
                            panic!("Unsupported field type in enum variant");
                        }
                    });
                define_lambda(quote!(#ffi_name::#ident(#(#variant_fields,)*)), quote!(#target_name::#ident(#(#converted_fields,)*)))
            },
            Fields::Unit => define_lambda(quote!(#ffi_name::#ident), quote!(#target_name::#ident)),
            _ => panic!("Unsupported fields in enum variant"),
        }
    });
    let obj = obj();
    let ffi_deref = ffi_deref();
    let interface_impl = impl_interface(
        quote!(#ffi_name),
        quote!(#target_name),
        quote!(match #ffi_deref { #(#from_ffi_arms),* }),
        package_boxed_expression(quote!(match #obj { #(#to_ffi_arms),* })),
    );

    let expanded = quote! {
        #input
        #converted
        #interface_impl
    };
    println!("{}", expanded);
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn impl_ffi_conv(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let attrs = parse_macro_input!(attr as AttributeArgs);
    let target_name = match attrs.first() {
        Some(NestedMeta::Lit(literal)) => format_ident!("{}", literal.to_token_stream().to_string()),
        Some(NestedMeta::Meta(Meta::Path(path))) => path.segments.first().unwrap().ident.clone(),
        _ => {
            // use default rules
            // for unnamed structs like UInt256 -> #target_name = [u8; 32]
            // for named structs -> generate ($StructName)FFI
            input.ident.clone()
        },
    };
    match input.data {
        Data::Struct(DataStruct { fields: ref f, ..}) => {
            match f {
                Fields::Named(ref fields) => from_named_struct(fields, target_name, &input),
                Fields::Unnamed(ref fields) => from_unnamed_struct(fields, target_name, &input),
                Fields::Unit => panic!("Fields::Unit not supported"),
            }
        },
        Data::Enum(ref data_enum) => from_enum(data_enum, target_name, &input),
        _ => panic!("FFIConv can only be derived for structs & enums")
    }
}

#[proc_macro_attribute]
pub fn impl_ffi_fn_conv(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let mut function = parse_macro_input!(item as ItemFn);
    println!("impl_ffi_fn_conv: input: {:?}", function);
    // Create the FFI function name
    let ffi_fn_name = format!("{}_ffi", function.sig.ident);

    // Build the FFI function signature
    let ffi_fn_ident = Ident::new(&ffi_fn_name, function.sig.ident.span());
    let inputs = &function.sig.inputs;
    let output = &function.sig.output;

    // Generate the FFI function
    let ffi_fn = quote! {
        #[no_mangle] pub extern "C" fn #ffi_fn_ident(#inputs) #output {
            #function.sig.ident(#inputs)
        }
    };
    println!("impl_ffi_fn_conv: output: {:?}", ffi_fn);
    // Convert the generated function back into a TokenStream and return it
    ffi_fn.into()

}

// #[proc_macro_attribute]
// pub fn ffi_chain_settings(_attr: TokenStream, item: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(item as ItemTrait);
//
//     let name = &input.ident;
//     let ffi_name = format_ident!("{}FFI", name);
//     let methods = input.items.into_iter().filter_map(|item| {
//         if let TraitItem::Method(method) = item {
//             Some(method)
//         } else {
//             None
//         }
//     });
//
//     // Generating FFI methods for the trait
//     let ffi_methods = methods.map(|method| {
//         let name = &method.sig.ident;
//         let ffi_name = format_ident!("{}_ffi", name);
//         let args = method.sig.inputs.iter().map(|arg| {
//             if let FnArg::Typed(arg) = arg {
//                 let arg_name = &arg.pat;
//                 quote! { #arg_name: <#arg_name as FFIConv<_>>::ffi_to(#arg_name) }
//             } else {
//                 panic!("Unexpected argument type");
//             }
//         });
//
//         quote! {
//             unsafe fn #ffi_name(&self, #(#args),*) -> *mut _ {
//                 <_ as FFIConv<_>>::ffi_to(self.#name(#(#args)*))
//             }
//         }
//     });
//
//     let expanded = quote! {
//         pub trait #ffi_name {
//             #(#ffi_methods)*
//         }
//
//         impl #name for ChainType {
//             // You need to put actual implementations here...
//         }
//
//         impl #name for DevnetType {
//             // You need to put actual implementations here...
//         }
//
//         impl #ffi_name for ChainType {
//             // FFI implementations...
//         }
//
//         impl #ffi_name for DevnetType {
//             // FFI implementations...
//         }
//     };
//
//     TokenStream::from(expanded)
// }