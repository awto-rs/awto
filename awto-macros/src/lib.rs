mod attributes;
mod derives;
mod error;
mod util;

macro_rules! create_derive {
    ($name:ident $(, $( $attrs:ident ),+)?) => {
        paste::paste! {
            #[proc_macro_derive($name, attributes($( $( $attrs ),*)?))]
            pub fn [< $name:snake >](input: proc_macro::TokenStream) -> proc_macro::TokenStream {
                let input = syn::parse_macro_input!(input as syn::DeriveInput);
                derives::[<expand_derive_ $name:snake>](input)
                    .unwrap_or_else(syn::Error::into_compile_error)
                    .into()
            }
        }
    }
}

create_derive!(Model, awto);
create_derive!(DatabaseModel, awto);
create_derive!(ProtobufModel, awto);
