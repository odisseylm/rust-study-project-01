use core::fmt::Debug;
use core::hash::Hash;
use core::num::NonZeroUsize;
use core::time::Duration;
use std::future::Future;
use std::time::Instant;
use ::quick_cache::Equivalent;
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
    fn from(value: CacheError) -> Self {
        AuthUserProviderError::CacheError(anyhow::Error::new(value))
    }
}


pub enum TtlMode {
    NoTtl,
    DefaultCacheTtl,
    Ttl(Duration)
}


// !!! Actually cache can be synchronized or not !!!
// Be careful! Probably it should be wrapped with async mutex/RwLock (like in SqlUserProvider)
#[async_trait::async_trait]
pub trait AsyncCache {
    type Key: Debug + Send + Sync;
    type Value: Debug + Send + Sync;

    fn with_capacity(capacity: NonZeroUsize)
        -> Result<Self,CacheError> where Self: Sized;
    fn with_capacity_and_ttl(capacity: NonZeroUsize, ttl: Duration)
        -> Result<Self, CacheError> where Self: Sized;

    async fn put(&mut self, key: Self::Key, ttl: TtlMode, value: Self::Value)
                 -> Result<(),CacheError>;
    async fn get(&mut self, key: &Self::Key) -> Result<Option<Self::Value>,CacheError>;

    // Comparing with 'get', it does not return None, since fetch must return value or fail.
    //
    // 'ttl' param is used for new fetched value.
    //    For validating current value expiration, previous ttl is used.
    async fn get_or_fetch<F, Fut, FetchErr>(&mut self, key: Self::Key, ttl: TtlMode, fetch: F)
                                            -> Result<Self::Value,CacheOrFetchError<FetchErr>>
        where
            F: FnOnce(Self::Key) -> Fut + Send,
            // TODO: try to minimize generic params count
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

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct LruAsyncCacheImpl <K, V:Clone> {
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

    async fn put(&mut self, key: Self::Key, ttl: TtlMode, value: Self::Value) -> Result<(), CacheError> where Self: Sized {
        self.int_cache.put(key, TtlEntry::from(value, ttl, self.default_ttl));
        Ok(())
    }

    async fn get(&mut self, key: &Self::Key) -> Result<Option<Self::Value>, CacheError> {
        ttl_entry_to_res(self.int_cache.get(key).map(|v|v.clone()))
    }
}


pub mod lru {
    pub type LruAsyncCache<K,V> = super::LruAsyncCacheImpl<K,V>;
}


#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct QuickAsyncCacheImpl <K, V:Clone> {
    default_ttl: Option<Duration>,
    #[derivative(Debug="ignore")]
    #[derivative(Debug(format_with="path::to::my_fmt_fn"))]
    // int_cache: ::quick_cache::sync::Cache<K,TtlEntry<V>>,
    int_cache: ::quick_cache::unsync::Cache<K,TtlEntry<V>>,
}


#[async_trait::async_trait]
impl <
    K: Debug + Hash + Eq + Equivalent<K> + Send + Sync,
    V: Debug + Send + Sync + Clone,
> AsyncCache for QuickAsyncCacheImpl<K,V> {
    type Key = K;
    type Value = V;

    fn with_capacity(capacity: NonZeroUsize) -> Result<Self,CacheError> {
        Ok(QuickAsyncCacheImpl {
            default_ttl: None,
            int_cache: ::quick_cache::unsync::Cache::new(capacity.get()),
        })
    }

    fn with_capacity_and_ttl(capacity: NonZeroUsize, ttl: Duration) -> Result<Self,CacheError> {
        Ok(QuickAsyncCacheImpl {
            default_ttl: Some(ttl),
            int_cache: ::quick_cache::unsync::Cache::new(capacity.get()),
        })
    }

    async fn put(&mut self, key: Self::Key, ttl: TtlMode, value: Self::Value)
        -> Result<(), CacheError> {
        self.int_cache.insert(key, TtlEntry::from(value, ttl, self.default_ttl));
        Ok(())
    }

    async fn get(&mut self, key: &Self::Key) -> Result<Option<Self::Value>, CacheError> {
        let option = self.int_cache.get(key).map(|v|v.clone());
        ttl_entry_to_res(option)
    }
}


pub mod quick_cache {
    pub type QuickAsyncCache<K,V> = super::QuickAsyncCacheImpl<K,V>;
}
