use core::future::Future;
use axum::Json;
use extension_trait::extension_trait;
// use mvv_account_soa::rest::error_rest::RestAppError;


#[extension_trait]
pub impl <T, Fut, Err> RestFutureToJson<T,Err>
for Fut where Fut: Future<Output = Result<T, Err>> {
    fn to_json(self) -> impl Future<Output = Result<Json<T>, Err>> {
        async { self.await.map(|data|Json(data)) }
    }
}
