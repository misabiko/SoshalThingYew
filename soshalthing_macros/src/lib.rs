extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, ItemEnum, LitStr, Type, parse_macro_input, Token, parse::{Parse, Parser, ParseStream}, punctuated::Punctuated, Ident, Field, Variant, Fields};
use syn::parse::Nothing;

#[derive(Debug)]
struct ServiceArgs {
	name: LitStr,
	article_type: Type,
	article_id_type: Type,
	//endpoints: Option<EndpointsArg>,
}

/*#[derive(Debug)]
struct EndpointsArg {
	bracket_token: token::Bracket,
	endpoints: Punctuated<Type, Token![,]>,
}*/

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
			/*endpoints: if input.is_empty() {
				None
			}else {
				Some(input.parse()?)
			},*/
		})
	}
}

/*impl Parse for EndpointsArg {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let content;
		Ok(Self {
			bracket_token: bracketed!(content in input),
			endpoints: content.parse_terminated(Type::parse)?,
		})
	}
}*/

#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
	let args = parse_macro_input!(args as ServiceArgs);
	let mut item_struct = parse_macro_input!(input as ItemStruct);

	let article_type = &args.article_type;
	let article_id_type = &args.article_id_type;
	if let Fields::Named(ref mut fields) = item_struct.fields {
		fields.named.push(
			Field::parse_named
				.parse2(quote! { articles: std::collections::HashMap<#article_id_type, ArticleRc<#article_type>> })
				.unwrap()
		);
	}

	let service_name = &args.name;
	let gen = quote! {
		use crate::services::ServiceInfo;

		pub const SERVICE_INFO: ServiceInfo = ServiceInfo {
			name: #service_name,
		};

		#item_struct
	};

	gen.into()
}

#[derive(Debug)]
struct ArticleActionsArgs {
	pub actions: Vec<Ident>,
}

impl Parse for ArticleActionsArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let actions = Punctuated::<Ident, Token![,]>::parse_terminated(input)?
			.into_iter().collect::<Vec<Ident>>();

		Ok(ArticleActionsArgs { actions })
	}
}

#[proc_macro_attribute]
pub fn service_article_actions(args: TokenStream, input: TokenStream) -> TokenStream {
	let args = parse_macro_input!(args as ArticleActionsArgs);
	let mut item_struct = parse_macro_input!(input as ItemStruct);

	if let Fields::Named(ref mut fields) = item_struct.fields {
		fields.named.push(
			Field::parse_named
				.parse2(quote! { actions_agent: Dispatcher<ArticleActionsAgent> })
				.unwrap()
		);
	}

	let actions = args.actions.iter();
	let gen = quote! {
		use crate::services::article_actions::ServiceInfoActions;

		pub const SERVICE_ARTICLE_ACTIONS: ServiceInfoActions = ServiceInfoActions {
			#(#actions: true,)*
			..ServiceInfoActions {
				like: false,
				repost: false,
				fetch_data: false,
			}
		};

		#item_struct
	};

	gen.into()
}

/*#[derive(Debug)]
struct ArticleActionsMsgArgs {
	pub actions: Vec<Ident>,
}

impl Parse for ArticleActionsMsgArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let actions = Punctuated::<Ident, Token![,]>::parse_terminated(input)?
			.into_iter().collect::<Vec<Ident>>();

		Ok(ArticleActionsMsgArgs { actions })
	}
}

#[proc_macro_attribute]
pub fn article_actions_msg(args: TokenStream, input: TokenStream) -> TokenStream {
	let _ = parse_macro_input!(args as Nothing);
	let mut item_enum = parse_macro_input!(input as ItemEnum);

	if SERVICE_ARTICLE_ACTIONS.like {
		item_enum.variants.push(
			Variant::parse
				.parse2(quote! { Like(HandlerId, ArticleWeak) })
				.unwrap()
		);
	}
	if SERVICE_ARTICLE_ACTIONS.repost {
		item_enum.variants.push(
			Variant::parse
				.parse2(quote! { Repost(HandlerId, ArticleWeak) })
				.unwrap()
		);
	}
	if SERVICE_ARTICLE_ACTIONS.fetch_data {
		item_enum.variants.push(
			Variant::parse
				.parse2(quote! { FetchData(HandlerId, ArticleWeak) })
				.unwrap()
		);
	}

	let gen = quote! { #item_enum };
	gen.into()
}*/

/*#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
*/