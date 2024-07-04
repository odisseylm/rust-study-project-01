use core::fmt::Debug;
use core::hash::Hash;
use core::num::NonZeroUsize;
use core::time::Duration;
use std::future::Future;
use std::time::Instant;
use log::info;
use mvv_auth::AuthUserProviderError;


#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CacheError {
    #[error("CacheError")]
    CacheError,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CacheOrFetchError <FetchErr>
    where FetchErr: std::error::Error
{
    #[error("CacheError")]
    CacheError(CacheError),
    #[error("FetchError")]
    FetchError(FetchErr),
}

impl From<CacheError> for AuthUserProviderError {
    fn from(value: CacheError) -> Self {
        AuthUserProviderError::CacheError(anyhow::Error::new(value))
    }
}
// impl From<CacheOrFetchError<AuthUserProviderError>> for AuthUserProviderError {
//     fn from(value: CacheOrFetchError<AuthUserProviderError>) -> Self {
//         match value {
//             CacheOrFetchError::CacheError(err) =>
//                 AuthUserProviderError::CacheError(anyhow::Error::new(err)),
//             CacheOrFetchError::FetchError(err) =>
//                 err,
//         }
//     }
// }


#[async_trait::async_trait]
pub trait AsyncCache {
    type Key: Debug + Send + Sync;
    type Value: Debug + Send + Sync;

    fn with_capacity(capacity: NonZeroUsize) -> Result<Self,CacheError> where Self: Sized;
    fn with_capacity_and_ttl(capacity: NonZeroUsize, ttl: Duration)
        -> Result<Self, CacheError> where Self: Sized;

    async fn put(&mut self, key: Self::Key, value: Self::Value)
        -> Result<(),CacheError> where Self: Sized;
    async fn put_with_ttl(&mut self, key: Self::Key, value: Self::Value) -> Result<(),CacheError>;
    async fn put_with_custom_ttl(&mut self, key: Self::Key, value: Self::Value, ttl: Duration) -> Result<(),CacheError>;
    async fn get(&mut self, key: &Self::Key) -> Result<Option<Self::Value>,CacheError>;

    // Comparing with 'get', it does not return None, since fetch must return value or fail.
    async fn get_or_fetch<F, Fut, FetchErr>(&mut self, key: Self::Key, fetch: F)
        -> Result<Self::Value,CacheOrFetchError<FetchErr>>
        where
            F: FnOnce(Self::Key) -> Fut + Send,
            Fut: Future<Output = Result<Self::Value,FetchErr>> + Send,
            FetchErr: std::error::Error,
            Self::Key: Clone,
            Self::Value: Clone,
            Self: Sized,
    {
        let cached = self.get(&key).await
            .map_err(CacheOrFetchError::CacheError) ?;
        if let Some(cached) = cached {
            info!("Using cached value: {:?}", cached);
            return Ok(cached);
        }

        let value: Self::Value = fetch(key.clone()).await
            .map_err(CacheOrFetchError::FetchError) ?;
        self.put(key.clone(), value.clone()).await
            .map_err(CacheOrFetchError::CacheError) ?;
        Ok(value)
    }
}


struct TtlEntry <V> {
    value: V,
    updated_at: Instant,
    ttl: Option<Duration>,
}
impl<V> TtlEntry<V> {
    fn is_expired(&self) -> bool {
        match self.ttl {
            None => false,
            Some(ttl) => {
                let expired = Instant::now() - self.updated_at > ttl;
                expired
            }
        }
    }
}


#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct LruAsyncCacheImpl <
    K,
    V,
> {
    default_ttl: Option<Duration>,
    #[derivative(Debug="ignore")]
    #[derivative(Debug(format_with="path::to::my_fmt_fn"))]
    int_cache: ::lru::LruCache<K,TtlEntry<V>>,
}


#[async_trait::async_trait]
impl <
    K: Debug + Hash + Eq + Send + Sync,
    V: Debug + Send + Sync + Clone,
> AsyncCache for LruAsyncCacheImpl<K,V> {
    type Key = K;
    type Value = V;

    fn with_capacity(capacity: NonZeroUsize) -> Result<Self,CacheError> {
        Ok(LruAsyncCacheImpl { default_ttl: None, int_cache: ::lru::LruCache::new(capacity) })
    }

    fn with_capacity_and_ttl(capacity: NonZeroUsize, ttl: Duration) -> Result<Self,CacheError> {
        Ok(LruAsyncCacheImpl { default_ttl: Some(ttl), int_cache: ::lru::LruCache::new(capacity) })
    }

    async fn put(&mut self, key: Self::Key, value: Self::Value) -> Result<(),CacheError> {
        self.int_cache.put(key, TtlEntry { value, updated_at: Instant::now(), ttl: None });
        Ok(())
    }

    async fn put_with_ttl(&mut self, key: Self::Key, value: Self::Value) -> Result<(),CacheError> {
        self.int_cache.put(key, TtlEntry { value, updated_at: Instant::now(), ttl: self.default_ttl });
        Ok(())
    }

    async fn put_with_custom_ttl(&mut self, key: Self::Key, value: Self::Value, ttl: Duration) -> Result<(),CacheError> {
        self.int_cache.put(key, TtlEntry { value, updated_at: Instant::now(), ttl: Some(ttl) });
        Ok(())
    }


    async fn get(&mut self, key: &Self::Key) -> Result<Option<Self::Value>, CacheError> {
        let value_entry_opt = self.int_cache.get(&key);
        match value_entry_opt {
            None => Ok(None),
            Some(value_entry) => {
                if value_entry.is_expired() {
                    Ok(None)
                } else {
                    Ok(Some(value_entry.value.clone()))
                }
            }
        }
    }
}


pub mod lru {
    pub type LruAsyncCache<K,V> = super::LruAsyncCacheImpl<K,V>;
}