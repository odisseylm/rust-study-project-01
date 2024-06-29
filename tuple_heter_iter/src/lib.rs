
cfg_if::cfg_if! {
    if #[cfg(feature = "tuple_len_64")] {
        pub const MAX_TUPLE_LEN: usize = 64;
        tuple_heter_iter_macro::generate_all_tuple_ops!{64}
    } else if #[cfg(feature = "tuple_len_32")] {
        pub const MAX_TUPLE_LEN: usize = 32;
        tuple_heter_iter_macro::generate_all_tuple_ops!{32}
    } else {
        pub const MAX_TUPLE_LEN: usize = 16;
        tuple_heter_iter_macro::generate_all_tuple_ops!{16}
    }
}
