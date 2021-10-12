pub enum Error {
    InputNotStruct,
    Syn(syn::Error),
}
