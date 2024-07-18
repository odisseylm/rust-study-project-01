use utoipa::openapi::{OpenApi, PathItem};


// A bit hacky method.
// utoipa >= 5 should support nest method and we will be able to remove it.
pub fn nest_open_api(prefix: &str, open_api: &::utoipa::openapi::OpenApi) -> OpenApi {
    let paths_with_prefix: indexmap::IndexMap<String, PathItem> = open_api.paths.paths.iter()
        .map(|e| ( [prefix, e.0.as_str()].concat(), e.1.clone()))
        // .collect::<PathsMap<String, PathItem>>()
        .collect::<indexmap::IndexMap<String, PathItem>>();

    let mut nested = open_api.clone();
    nested.paths.paths = paths_with_prefix;
    nested
}
//
// fn nest_open_api <
//     U: ::utoipa::openapi::OpenApi,
// > (open_api: U) -> OpenApi {
//     u
// }

