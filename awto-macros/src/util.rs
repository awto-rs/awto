use bae::TryFromAttributes;

use crate::error::Error;

pub struct Field<Attr> {
    pub attrs: Attr,
    pub field: syn::Field,
}

pub fn parse_struct_fields<Attr>(data: syn::Data) -> Result<Vec<Field<Attr>>, Error>
where
    Attr: Default + TryFromAttributes,
{
    let fields = match data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) => named,
        _ => return Err(Error::InputNotStruct),
    };

    fields
        .into_iter()
        .map(|field| {
            Ok(Field {
                attrs: Attr::try_from_attributes(&field.attrs)
                    .map_err(Error::Syn)?
                    .unwrap_or_default(),
                field,
            })
        })
        .collect::<Result<_, _>>()
}
