
pub(crate) const MAX_TUPLE_LEN: usize =
    if cfg!(feature = "tuple_len_64") {
        64
    } else if cfg!(feature = "tuple_len_32") {
        32
    } else {
        16
    };