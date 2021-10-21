const OPTION_PREFIXES: [&str; 3] = ["std::option::Option<", "option::Option<", "Option<"];
const VEC_PREFIXES: [&str; 3] = ["std::vec::Vec<", "vec::Vec<", "Vec<"];

pub fn strip_ty_option(ty: &str) -> &str {
    for prefix in OPTION_PREFIXES {
        if ty.starts_with(prefix) {
            return &ty[prefix.len()..(ty.len() - 1)];
        }
    }

    ty
}

pub fn is_ty_option(ty: &str) -> bool {
    OPTION_PREFIXES.iter().any(|prefix| ty.starts_with(prefix))
}

pub fn is_ty_vec(ty: &str) -> bool {
    VEC_PREFIXES.iter().any(|prefix| ty.starts_with(prefix))
}
