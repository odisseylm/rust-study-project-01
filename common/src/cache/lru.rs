use std::fmt::Debug;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::time::Duration;
use super::{ AsyncCache, CacheFactory, CacheError, ttl_entry_to_res, TtlEntry, TtlMode };
//--------------------------------------------------------------------------------------------------



#[derive(Debug)]
pub struct LruAsyncCache <K: Hash + Eq, V:Clone> {
    default_ttl: Option<Duration>,
    int_cache: lru::LruCache<K,TtlEntry<V>>,
}


#[async_trait::async_trait]
impl <
    K: Debug + Hash + Eq + Send + Sync,
    V: Debug + Send + Sync + Clone,
> CacheFactory for LruAsyncCache<K,V> {
    fn with_capacity(capacity: NonZeroUsize) -> Result<Self, CacheError> {
        Ok(LruAsyncCache { default_ttl: None, int_cache: lru::LruCache::new(capacity) })
    }
    fn with_capacity_and_ttl(capacity: NonZeroUsize, ttl: Duration) -> Result<Self, CacheError> {
        Ok(LruAsyncCache { default_ttl: Some(ttl), int_cache: lru::LruCache::new(capacity) })
    }
}


#[async_trait::async_trait]
impl <
    K: Debug + Hash + Eq + Send + Sync,
    V: Debug + Send + Sync + Clone,
> AsyncCache for LruAsyncCache<K,V> {
    type Key = K;
    type Value = V;

    async fn put(&mut self, key: Self::Key, ttl: TtlMode, value: Self::Value) -> Result<(), CacheError> where Self: Sized {
        self.int_cache.put(key, TtlEntry::from(value, ttl, self.default_ttl));
        Ok(())
    }
    //noinspection DuplicatedCode
    async fn get(&mut self, key: &Self::Key) -> Result<Option<Self::Value>, CacheError> {
        ttl_entry_to_res(self.int_cache.get(key).map(|v|v.clone()))
    }
}

