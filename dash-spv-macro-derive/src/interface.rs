use syn::{Expr, ExprPath, ExprMethodCall, ExprPtr, ExprUnary, Index, Ident, parse::{Parse, ParseStream}, parse_macro_input, Path, PathSegment, punctuated::Punctuated, Result, token::Colon2, Token, UnOp};
use syn::__private::Span;
use quote::quote;

//impl_syn_extension!()

struct Conversion {
    name: Ident,
    field: Ident,
    index: Expr,
}

impl Parse for Conversion {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![,]>()?;
        let field = input.parse()?;
        input.parse::<Token![,]>()?;
        let index = input.parse()?;
        Ok(Conversion { name, field, index })
    }
}

#[proc_macro]
pub fn impl_syn_extension(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let Conversion { name, field, index } = parse_macro_input!(input as Conversion);
    let output = quote! {
        fn #name(ffi: *mut #name) -> #name {
            dash_spv_ffi::FFIConv::ffi_from((*(*ffi).#field.add(#index)))
        }
    };
    output.into()
}


// Construct the path "dash_spv_ffi::FFIConv::ffi_from"



const ffi_from_ident: Ident = Ident::new("ffi_from", Span::call_site());
const ffi_conv_ident: Ident = Ident::new("FFIConv", Span::call_site());
const dash_spv_ffi_ident: Ident = Ident::new("dash_spv_ffi", Span::call_site());

const ffi_from_segment: PathSegment = PathSegment::from(ffi_from_ident);
const ffi_conv_segment: PathSegment = PathSegment::from(ffi_conv_ident);
const dash_spv_ffi_segment: PathSegment = PathSegment::from(dash_spv_ffi_ident);


// let mut path_segments = Punctuated::new();
// path_segments.push(dash_spv_ffi_segment);
// path_segments.push_value(Colon2::default());
// path_segments.push(ffi_conv_segment);
// path_segments.push_value(Colon2::default());
// path_segments.push(ffi_from_segment);
//
// let ffi_from_path = Path {
// leading_colon: None,
// segments: path_segments,
// };
//
// // Construct the expression for the method call
// let field_name_keys_ident = Ident::new("field_name_keys", Span::call_site());
// let i_index = Index::from(0); // replace with your desired index
//
// let field_expr = Expr::Field(ExprField {
// attrs: Vec::new(),
// base: Box::new(Expr::Ident(ExprIdent {
// attrs: Vec::new(),
// ident: field_name_keys_ident,
// })),
// dot_token: Dot::default(),
// member: Member::Unnamed(i_index),
// });
//
// let deref_field_expr = Expr::Unary(ExprUnary {
// attrs: Vec::new(),
// op: UnOp::Deref(Star::default()),
// expr: Box::new(field_expr),
// });
//
// let deref_ptr_expr = Expr::Unary(ExprUnary {
// attrs: Vec::new(),
// op: UnOp::Deref(Star::default()),
// expr: Box::new(Expr::Paren(ExprParen {
// attrs: Vec::new(),
// paren_token: Paren::default(),
// expr: Box::new(Expr::Ptr(ExprPtr {
// attrs: Vec::new(),
// star_token: Star::default(),
// expr: Box::new(deref_field_expr),
// })),
// })),
// });
//
// let call_expr = Expr::Call(ExprCall {
// attrs: Vec::new(),
// func: Box::new(Expr::Path(ExprPath {
// attrs: Vec::new(),
// qself: None,
// path: ffi_from_path,
// })),
// paren_token: Paren::default(),
// args: iter::once(deref_ptr_expr).collect(),
// });
