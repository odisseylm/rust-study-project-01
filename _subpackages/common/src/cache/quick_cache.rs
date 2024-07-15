use std::fmt::Debug;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::time::Duration;
use quick_cache::Equivalent;
use super::{AsyncCache, CacheError, CacheFactory, ttl_entry_to_res, TtlEntry, TtlMode};
//--------------------------------------------------------------------------------------------------



#[derive(Debug)]
pub struct QuickAsyncCache <K, V:Clone> {
    default_ttl: Option<Duration>,
    int_cache: quick_cache::unsync::Cache<K,TtlEntry<V>>,
}


#[async_trait::async_trait]
impl <
    K: Debug + Hash + Eq + Equivalent<K> + Send + Sync,
    V: Debug + Send + Sync + Clone,
> CacheFactory for QuickAsyncCache<K,V> {

    fn with_capacity(capacity: NonZeroUsize) -> Result<Self, CacheError> {
        Ok(QuickAsyncCache {
            default_ttl: None,
            int_cache: quick_cache::unsync::Cache::new(capacity.get()),
        })
    }

    fn with_capacity_and_ttl(capacity: NonZeroUsize, ttl: Duration) -> Result<Self, CacheError> {
        Ok(QuickAsyncCache {
            default_ttl: Some(ttl),
            int_cache: quick_cache::unsync::Cache::new(capacity.get()),
        })
    }
}


#[async_trait::async_trait]
impl <
    K: Debug + Hash + Eq + Equivalent<K> + Send + Sync,
    V: Debug + Send + Sync + Clone,
> AsyncCache for QuickAsyncCache<K,V> {
    type Key = K;
    type Value = V;

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
