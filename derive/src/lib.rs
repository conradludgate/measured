use group::LabelGroup;
use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput};

// this is some of the worst macro code I have ever written.
// to me in 5 months time, I am so sorry.
// i was really not feeling like dealing with syn so you have to deal with this monstosity of copy-pasta

mod group;

#[proc_macro_derive(LabelGroup, attributes(label))]
pub fn derive_group(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match LabelGroup::try_from(parse_macro_input!(input as DeriveInput)) {
        Ok(output) => output.to_token_stream(),
        Err(err) => err.into_compile_error().into_token_stream(),
    }
    .into()
}
