extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, LitStr};
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn service(_args: TokenStream, input: TokenStream) -> TokenStream {
	//TODO Parse first argument without quotes
	let service_name = parse_macro_input!(_args as LitStr);
	let /*mut*/ item_struct = parse_macro_input!(input as ItemStruct);

	/*if let syn::Fields::Named(ref mut fields) = item_struct.fields {
		fields.named.push(
			syn::Field::parse_named
				.parse2(quote! { })
		);
	}*/

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