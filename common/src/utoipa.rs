use std::collections::HashSet;
use std::ffi::OsString;
use std::ops::Sub;
use itertools::Itertools;
use log::info;
use utoipa::openapi::{ OpenApi, PathItem };
use crate::string::remove_optional_suffix;
//--------------------------------------------------------------------------------------------------



// A bit hacky method.
// utoipa >= 5 should support nest method and we will be able to remove it.
pub fn nest_open_api(prefix: &str, open_api: &OpenApi) -> OpenApi {
    let paths_with_prefix: indexmap::IndexMap<String, PathItem> = open_api.paths.paths.iter()
        .map(|e| ( [prefix, e.0.as_str()].concat(), e.1.clone()))
        .collect::<indexmap::IndexMap<String, PathItem>>();

    let mut nested = open_api.clone();
    nested.paths.paths.clear();
    nested.paths.paths.extend(paths_with_prefix);
    nested
}


pub fn axum_path_from_open_api(open_api_path: String) -> String {
    let axum_path_str = open_api_path
        // maybe not elegant, but the easiest approach
        .replace("/{", "/:")
        .replace("}/", "/")
        .replace("}?", "?")
    ;
    if !axum_path_str.contains('?') {
        remove_optional_suffix(axum_path_str, "}")
    } else {
        axum_path_str
    }
}

#[extension_trait::extension_trait]
pub impl <
    S: Clone + Send + Sync + 'static,
> AxumOpenApiRouterExt<S> for axum::Router<S> {

    /// It is designed for usage with macros `axum_open_api_axum_route`.
    /// Usage:
    /// ```no_build
    ///     use mvv_common::AxumOpenApiRouterExt;
    ///     use mvv_common::axum_open_api_axum_route as open_api_route;
    ///
    ///     let r = Router::new()
    ///         .route_from_open_api(open_api_route!(rest_get_client_account::<AccountS>))
    ///         ...
    /// ```
    fn route_from_open_api <
        UT: utoipa::Path,
        T: 'static,
        H: axum::handler::Handler<T,S>,
    > (self, route_entry: (&UT, H)) -> Self {
        let (open_api_path, handler) = route_entry;
        self.route_from_open_api_internal(open_api_path, handler)
    }

    fn route_from_open_api_internal <
        UT: utoipa::Path,
        T: 'static,
        H: axum::handler::Handler<T,S>,
    > (self, _open_api_path: &UT, handler: H) -> Self {

        use axum::routing::MethodRouter;
        use utoipa::openapi::PathItemType;

        let open_api_path = UT::path();
        let path_item = UT::path_item(None);

        #[cfg(debug_assertions)] // validation only in debug mode
        validate_path_params(&open_api_path, &path_item);

        let path_item_type = path_item.operations.first()
            .expect(&format!("Missed path type (HTTP method) in OpenApi macro for {}", open_api_path))
            .0;
        let axum_path: String = axum_path_from_open_api(open_api_path);

        use axum::routing::{ get, post, put, patch, delete, options, head, trace };

        let mr: MethodRouter<S> = match path_item_type {
            PathItemType::Get => get(handler),
            PathItemType::Post => post(handler),
            PathItemType::Put => put(handler),
            PathItemType::Delete => delete(handler),
            PathItemType::Options => options(handler),
            PathItemType::Head => head(handler),
            PathItemType::Patch => patch(handler),
            PathItemType::Trace => trace(handler),
            // PathItemType::Connect => connect(handler),
            other => panic!("Unsupported path item type {:?}", other),
        };

        self.route(&axum_path, mr)
    }
}

fn validate_path_params(open_api_path: &str, open_api_path_item: &PathItem) {
    // TODO: add support of '?' and query params
    let url_path_params: HashSet<_> = open_api_path.split('/')
        .filter(|s|s.starts_with('{') && s.ends_with('}'))
        .map(|s|{
            let s = s.strip_prefix('{').unwrap_or(s);
            let s = s.strip_suffix('}').unwrap_or(s);
            s})
        .collect::<HashSet<_>>();

    let op: &utoipa::openapi::path::Operation = open_api_path_item.operations.first()
        .expect("Path should contain 1 operation")
        .1;

    // let annotated_path_params = path_item.parameters.as_ref()
    let annotated_path_params = op.parameters.as_ref()
        .map(|parameters|parameters
            .iter()
            .map(|p|p.name.as_str())
            .collect::<HashSet<_>>())
        .unwrap_or(HashSet::new());

    let mut diff = url_path_params.sub(&annotated_path_params);
    let diff2 = annotated_path_params.sub(&url_path_params);
    diff.extend(diff2);
    // let diff = diff1.union(&diff2).collect::<HashSet<_>>();
    // let diff = { let mut diff = diff1; diff.extend(diff2); diff };

    if !diff.is_empty() {
        let path_params_str = diff.iter().join(", ");
        let operation_id = op.operation_id.as_ref().map(|op_id| op_id.as_str()).unwrap_or("");
        panic!("Mismatched path params [{}] for operation [{} = {}].", path_params_str, operation_id, open_api_path);
    }
}


/// The best approach!
/// Usage:
/// ```no_build
///     use mvv_common::AxumOpenApiRouterExt;
///     use mvv_common::axum_open_api_axum_route as open_api_route;
///
///     let r = Router::new()
///         .route_from_open_api(open_api_route!(rest_get_client_account::<AccountS>))
///         ...
/// ```
#[macro_export] macro_rules! axum_open_api_axum_route {
    ($rest_method:path) => {
        (
            & mvv_proc_macro::utoipa_path_obj! { $rest_method },
            $rest_method,
        )
    };
}


/// My first so-so approach.
/// Usage:
/// ```no_build
///     use mvv_common::axum_path_from_open_api as axum_path;
///
///     let r = Router::new()
///         .route(
///             &axum_path! { rest_get_client_accounts },
///             GET(rest_get_client_accounts::<AccountS>)
///         )
///         ...
/// ```
#[macro_export] macro_rules! axum_path_from_open_api {
    // REST method should have '#[utoipa::path(...)]'
    // which is translated into structure with name __path_[your_method_name]
    //
    // pub struct __path_your_method;
    // impl utoipa::Path for __path_your_method {
    //     fn path() -> String { "/client/{client_id}/account/{account_id}" }
    //     fn path_item(...) { ... }
    ($rest_method:ident) => {
        place_macro::place! {
            {
                // Making ident __path_your_method and calling its static methods.
                //
                let open_api_path_str = <  __identifier__(__path_, $rest_method) as utoipa::Path>::path();
                let axum_path_str = $crate::utoipa::axum_path_from_open_api(open_api_path_str);
                axum_path_str
            }
        }
    };
}


#[macro_export] macro_rules! axum_route_from_open_api {
    ($route:ident, $rest_method:path) => {
        {
            #[allow(unused_imports)]
            use $crate::utoipa::AxumOpenApiRouterExt;
            let route = $route.route_from_open_api_internal(
                & mvv_proc_macro::utoipa_path_obj! { $rest_method },
                $rest_method,
            );
            route
        }
    };
}


#[macro_export] macro_rules! axum_route_from_open_api_raw {
    ($route:ident, $rest_method_name:ident) => {
        place_macro::place! {
            {
                #[allow(unused_imports)]
                use $crate::utoipa::OpenApiRouterExt;
                let route = $route.route_from_open_api_internal(
                    & __identifier__(__path_, $rest_method_name),
                    $rest_method_name,
                );
                route
            }
        }
    };
    // use it in case if your method has generic params
    // Usage:
    // let r: Router<Arc<AccountRest<AccountS>>> = Router::new();
    // let r = axum_route_from_open_api_raw!(r,
    //         call_rest_get_client_account,
    //         call_rest_get_client_account::<AccountS>
    //     );
    ($route:ident, $rest_method_name:ident, $rest_method:path) => {
        place_macro::place! {
            {
                #[allow(unused_imports)]
                use $crate::utoipa::OpenApiRouterExt;
                let route = $route.route_from_open_api_internal(
                    & __identifier__(__path_, $rest_method_name),
                    $rest_method,
                );
                route
            }
        }
    };
}


#[macro_export] macro_rules! axum_route_from_open_api_with_gen_params {
    ($route:ident, $rest_method_name:ident, $($gen_param:ty),+) => {
        place_macro::place! {
            {
                #[allow(unused_imports)]
                use $crate::utoipa::OpenApiRouterExt;
                let route = $route.route_from_open_api_internal(
                    & __identifier__(__path_, $rest_method_name),
                    $rest_method_name :: < $($gen_param),+ >,
                );
                route
            }
        }
    };
}

pub fn to_generate_open_api() -> bool {
    std::env::args_os().contains(&OsString::from("--generate-open-api"))
}

pub enum UpdateApiFile {
    Always,
    /// To avoid regeneration of dependant client stubs.
    IfModelChanged,
}

pub fn generate_open_api(open_api: &OpenApi, module_name: &str, update_api_file: UpdateApiFile, dir: Option<&str>) -> Result<(), anyhow::Error> {

    use std::path::Path;

    let dir = dir.map(Path::new)
        .unwrap_or(Path::new("."));

    let file = dir.join(&format!("{module_name}-openapi.json"));
    info!("Generating Open API file for [{module_name}] => [{file:?}]");

    let open_api_json = open_api.to_pretty_json() ?;

    let to_update_file: bool = match update_api_file {
        UpdateApiFile::Always => true,
        UpdateApiFile::IfModelChanged => {
            if !file.exists() { true }
            else {
                let file_content = std::fs::read_to_string(&file) ?;
                file_content != open_api_json
            }
        }
    };

    if to_update_file {
        std::fs::write(&file, &open_api_json) ?;
        info!("Open API file [{file:?}] is generated.");
    } else {
        info!("Open API file [{file:?}] is not changed.");
    }

    Ok(())
}


//--------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::test::TestSringOps;
    use super::axum_path_from_open_api;

    #[test]
    fn axum_path_from_open_api_test() {

        assert_eq!(
            axum_path_from_open_api("/client/{client_id}/account/all".to_test_string()),
            "/client/:client_id/account/all",
        );
        assert_eq!(
            axum_path_from_open_api("/client/{client_id}/account/{account_id}/".to_test_string()),
            "/client/:client_id/account/:account_id/",
        );
        assert_eq!(
            axum_path_from_open_api("/client/{client_id}/account/{account_id}".to_test_string()),
            "/client/:client_id/account/:account_id",
        );

        // Now ony simple approach is used. Let's improve it when it is really needed.
        assert_eq!(
            axum_path_from_open_api("/client/{client_id}/account/{account_id}?param1={value1}".to_test_string()),
            "/client/:client_id/account/:account_id?param1={value1}",
        );
    }

}
