extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, LitStr, Type, parse_macro_input, Token, parse::{Parse, Parser, ParseStream}, punctuated::Punctuated, token, bracketed};

#[derive(Debug)]
struct ServiceArgs {
	name: LitStr,
	article_type: Type,
	article_id_type: Type,
	_endpoints: Option<EndpointsArg>,
}

#[derive(Debug)]
struct EndpointsArg {
	_bracket_token: token::Bracket,
	_endpoints: Punctuated<Type, Token![,]>,
}

impl Parse for ServiceArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		//Should be using Punctuated, but into_iter().next() returns options, not results
		let name = input.parse()?;
		let _ = input.parse::<Token![,]>();
		let article_type = input.parse()?;
		let _ = input.parse::<Token![,]>();
		let article_id_type = input.parse()?;
		let _ = input.parse::<Token![,]>();

		Ok(Self {
			name,
			article_type,
			article_id_type,
			_endpoints: if input.is_empty() {
				None
			}else {
				Some(input.parse()?)
			},
		})
	}
}

impl Parse for EndpointsArg {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let content;
		Ok(Self {
			_bracket_token: bracketed!(content in input),
			_endpoints: content.parse_terminated(Type::parse)?,
		})
	}
}

#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
	let args = parse_macro_input!(args as ServiceArgs);
	let mut item_struct = parse_macro_input!(input as ItemStruct);

	let article_type = &args.article_type;
	let article_id_type = &args.article_id_type;
	if let syn::Fields::Named(ref mut fields) = item_struct.fields {
		fields.named.push(
			syn::Field::parse_named
				.parse2(quote! { articles: std::collections::HashMap<#article_id_type, ArticleRc<#article_type>> })
				.unwrap()
		);
	}

	let service_name = &args.name;
	let gen = quote! {
		use crate::services::ServiceInfo;

		pub const SERVICE_INFO: ServiceInfo = ServiceInfo {
			name: #service_name
		};

		#item_struct
	};

	gen.into()
}

/*#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
*/