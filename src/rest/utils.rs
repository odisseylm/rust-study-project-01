use std::future::Future;
use axum::Json;
use extension_trait::extension_trait;
use crate::rest::error_rest::AppRestError;


#[extension_trait]
pub impl<T,Fut> RestFutureToJson<T> for Fut where Fut: Future<Output = Result<T, AppRestError>> {
    fn to_json(self) -> impl Future<Output = Result<Json<T>, AppRestError>> {
        async { self.await.map(|data|Json(data)) }
    }
}


// async fn rest_to_json34 <T, F: Future<Output = Result<T, AppRestError>> >
//     (fut: F) -> impl Future<Output = Result<Json<T>, AppRestError>> {
//     async { fut.await.map(|data|Json(data)) }
// }
//
// fn rest_to_json35 <T, F: Future<Output = Result<T, AppRestError>> >
//     (fut: F) -> impl Future<Output = Result<Json<T>, AppRestError>> {
//     async { fut.await.map(|data|Json(data)) }
// }
