use proc_macro::TokenStream;
use quote::quote;
use syn::{parse2, parse_macro_input, parse_quote, ItemFn};

/// A procedural macro for automatically wrapping a request handler inside a tokio::task_local!
/// [`LocalKey`] scope for `LANGUAGE`, with the value of the [`ClientLocale`] request guard.
///
/// Use of this macro eliminates the need for writing and maintaining boilerplate code caused
/// by manually wrapping the request handler body inside a `LANGUAGE` scope, while also
/// having to take in a [`ClientLocale`] guard and handling that properly.
///
/// This macro should be used for any endpoint whose request handler calls a translation
/// function at some point, so basically any page or API endpoint (API endpoints need
/// to be localized because errors are also translated)
#[proc_macro_attribute]
pub fn localized(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut f = parse_macro_input!(input as ItemFn);

    // modify the request handler to automatically take in our [`ClientLocale`] request
    // guard (defined in pointercrate-core-api/src/localization.rs)
    f.sig
        .inputs
        .push(parse_quote! { __locale: pointercrate_core_api::localization::ClientLocale });

    let block = &f.block;
    let block = quote! {
        {
            pointercrate_core::localization::LANGUAGE.scope(__locale.into(), async {
                #block
            }).await
        }
    };

    f.block = parse2(block).unwrap();

    TokenStream::from(quote!(#f))
}

/// Identical behaviour to `#[localized]`, but modified to support error catchers.
#[proc_macro_attribute]
pub fn localized_catcher(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut f = parse_macro_input!(input as ItemFn);

    // modify the request handler to automatically take in our [`ClientLocale`] request
    // guard (defined in pointercrate-core-api/src/localization.rs)
    f.sig.inputs.push(parse_quote! { __req: &rocket::Request<'_> });

    let block = &f.block;
    let block = quote! {
        {
            let __locale = match ClientLocale::from_request(__req).await {
                Outcome::Success(locale) => locale,
                _ => panic!("help"),
            };

            pointercrate_core::localization::LANGUAGE.scope(__locale.into(), async {
                #block
            }).await
        }
    };

    f.block = parse2(block).unwrap();

    TokenStream::from(quote!(#f))
}
