use utoipa::openapi::{ OpenApi, PathItem };
use crate::string::remove_optional_suffix;
// use place_macro::place;
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


pub fn open_api_path_to_axum(open_api_path: String) -> String {
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
> OpenApiRouterExt<S> for axum::Router<S> {
    fn route_from_open_api <
        UT: utoipa::Path,
        T: 'static,
        H: axum::handler::Handler<T,S>,
    > (self, _open_api_path: &UT, handler: H) -> Self {

        use axum::routing::MethodRouter;
        use utoipa::openapi::PathItemType;

        let open_api_path = UT::path();
        let path_item = UT::path_item(None);
        let path_item_type = path_item.operations.first()
            .expect(&format!("Missed path type (HTTP method) in OpenApi macro for {}", open_api_path))
            .0;
        let axum_path: String = open_api_path_to_axum(open_api_path);

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


#[macro_export] macro_rules! open_api_path_to_axum {
    // REST method should have '#[utoipa::path(...)]'
    // which is translated into structure with name __path_[your_method_name]
    //
    // pub struct __path_your_method;
    // impl utoipa::Path for __path_your_method {
    //     fn path() -> String { "/client/{client_id}/account/{account_id}" }
    //     fn path_item(...) { ... }
    ($rest_method:ident) => {
        place! {
            {
                // Making ident __path_your_method and calling its static methods.
                //
                let open_api_path_str = <  __identifier__(__path_, $rest_method) as utoipa::Path>::path();
                let axum_path_str = $crate::utoipa::open_api_path_to_axum(open_api_path_str);
                axum_path_str
            }
        }
    };
}


#[macro_export] macro_rules! route_from_open_api_raw {
    ($route:ident, $rest_method_name:path, $rest_method:path) => {
        place! {
            {
                let route = $route.route_from_open_api(
                    & __identifier__(__path_, $rest_method_name),
                    $rest_method,
                );
                route
            }
        }
    };
}


#[macro_export] macro_rules! route_from_open_api_with_gen_params {
    ($route:ident, $rest_method_name:ident, $($gen_param:ty),+) => {
        place! {
            {
                let route = $route.route_from_open_api(
                    & __identifier__(__path_, $rest_method_name),
                    $rest_method_name :: < $($gen_param),+ >,
                );
                route
            }
        }
    };
}


//--------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::test::TestSringOps;
    use super::open_api_path_to_axum;

    #[test]
    fn open_api_path_to_axum_test() {

        assert_eq!(
            open_api_path_to_axum("/client/{client_id}/account/all".to_test_string()),
            "/client/:client_id/account/all",
        );
        assert_eq!(
            open_api_path_to_axum("/client/{client_id}/account/{account_id}/".to_test_string()),
            "/client/:client_id/account/:account_id/",
        );
        assert_eq!(
            open_api_path_to_axum("/client/{client_id}/account/{account_id}".to_test_string()),
            "/client/:client_id/account/:account_id",
        );

        // Now ony simple approach is used. Let's improve it when it is really needed.
        assert_eq!(
            open_api_path_to_axum("/client/{client_id}/account/{account_id}?param1={value1}".to_test_string()),
            "/client/:client_id/account/:account_id?param1={value1}",
        );
    }

}
