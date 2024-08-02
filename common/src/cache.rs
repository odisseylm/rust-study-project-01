pub mod quick_cache;
pub mod lru;
pub mod associative_cache;


//--------------------------------------------------------------------------------------------------
use core::fmt::Debug;
use core::hash::Hash;
use core::num::NonZeroUsize;
use core::time::Duration;
use std::future::Future;
use std::time::Instant;
use log::info;
use mvv_auth::AuthUserProviderError;
//--------------------------------------------------------------------------------------------------



#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CacheError {
    #[error("CacheError")]
    CacheError(anyhow::Error),
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
    fn from(err: CacheError) -> Self {
        match err {
            CacheError::CacheError(err) => AuthUserProviderError::CacheError(err),
        }
    }
}


pub enum TtlMode {
    NoTtl,
    DefaultCacheTtl,
    Ttl(Duration)
}


// Optional trait.
// This functionality moved from AsyncCache trait because some underlying implementations
// have fixed size as const generic param (for example associative-cache)
pub trait CacheFactory {
    fn with_capacity(capacity: NonZeroUsize)
                     -> Result<Self,CacheError> where Self: Sized;
    fn with_capacity_and_ttl(capacity: NonZeroUsize, ttl: Duration)
                             -> Result<Self, CacheError> where Self: Sized;
}


// !!! Actually cache can be synchronized or not !!!
// Be careful! Probably it should be wrapped with async mutex/RwLock (like in SqlUserProvider)
#[async_trait::async_trait]
pub trait AsyncCache {
    type Key: Debug + Hash + Send + Sync;
    type Value: Debug + Send + Sync;

    async fn put(&mut self, key: Self::Key, ttl: TtlMode, value: Self::Value)
                 -> Result<(),CacheError>;
    async fn get(&mut self, key: &Self::Key) -> Result<Option<Self::Value>,CacheError>;

    // Comparing with 'get', it does not return None, since fetch must return value or fail.
    //
    // 'ttl' param is used for new fetched value.
    //    For validating current value expiration, previous ttl is used.
    async fn get_or_fetch<Fut, FetchErr>(&mut self, key: Self::Key, ttl: TtlMode, fetch: impl FnOnce(Self::Key) -> Fut + Send)
                                            -> Result<Self::Value,CacheOrFetchError<FetchErr>>
        where
            // F: FnOnce(Self::Key) -> Fut + Send,
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
        self.put(key.clone(), ttl, value.clone()).await
            .map_err(CacheOrFetchError::CacheError) ?;
        Ok(value)
    }
}



//--------------------------------------------------------------------------------------------------
//          Utility types/functions for easy writing wrappers over existent cache impls
//--------------------------------------------------------------------------------------------------
//
#[derive(Debug, Clone)]
struct TtlEntry <V: Clone> {
    value: V,
    updated_at: Instant,
    ttl: Option<Duration>,
}

impl<V: Clone> TtlEntry<V> {
    fn is_expired(&self) -> bool {
        match self.ttl {
            None => false,
            Some(ttl) => {
                let expired = Instant::now() - self.updated_at > ttl;
                expired
            }
        }
    }
    // internal method
    fn from(value: V, ttl: TtlMode, default_ttl: Option<Duration>) -> TtlEntry<V> {
        match ttl {
            TtlMode::NoTtl =>
                TtlEntry { value, updated_at: Instant::now(), ttl: None },
            TtlMode::DefaultCacheTtl =>
                TtlEntry { value, updated_at: Instant::now(), ttl: default_ttl },
            TtlMode::Ttl(ttl) =>
                TtlEntry { value, updated_at: Instant::now(), ttl: Some(ttl) },
        }
    }
}

fn ttl_entry_to_res<V: Clone>(value_entry_opt: Option<TtlEntry<V>>) -> Result<Option<V>, CacheError> {
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


impl From<CacheOrFetchError<AuthUserProviderError>> for AuthUserProviderError {
    fn from(value: CacheOrFetchError<AuthUserProviderError>) -> Self {
        match value {
            CacheOrFetchError::CacheError(err) =>
                AuthUserProviderError::CacheError(anyhow::Error::new(err)),
            CacheOrFetchError::FetchError(err) => err,
        }
    }
}
