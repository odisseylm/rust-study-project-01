use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;
use super::{ AsyncCache, CacheError, ttl_entry_to_res, TtlEntry, TtlMode };
// use associative_cache::{ AssociativeCache, LruReplacement, LruTimestamp };
//--------------------------------------------------------------------------------------------------



#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct AssociativeAsyncCache <Capacity: associative_cache::Capacity, K, V: Clone> {
    default_ttl: Option<Duration>,
    #[derivative(Debug="ignore")]
    // #[derivative(Debug(format_with="path::to::my_fmt_fn"))] // TODO: print capacity and current size if possible
    int_cache: associative_cache::AssociativeCache <
        K, TtlEntry<V>,
        Capacity,
        associative_cache::HashDirectMapped,
        associative_cache::LruReplacement,
    >,
}

fn create_cache <
    Capacity: associative_cache::Capacity,
    K,
    V: Clone,
    I,
    R: associative_cache::Replacement<TtlEntry<V>, Capacity> + Default,
>() -> associative_cache::AssociativeCache<K, TtlEntry<V>, Capacity, I, R> {
    let cache: associative_cache::AssociativeCache<K, TtlEntry<V>, Capacity, I, R> =
        associative_cache::AssociativeCache:: <K, TtlEntry<V>, Capacity, I, R> ::default();
    cache
}

impl <
    Capacity: associative_cache::Capacity,
    K: Debug + Hash + Eq + Send + Sync,
    V: Debug + Send + Sync + Clone,
> AssociativeAsyncCache<Capacity,K,V> {
    pub fn with_capacity() -> Result<Self, CacheError> {
        Ok(AssociativeAsyncCache { default_ttl: None, int_cache: create_cache() })
    }
    pub fn with_capacity_and_ttl(ttl: Duration) -> Result<Self, CacheError> {
        Ok(AssociativeAsyncCache { default_ttl: Some(ttl), int_cache: create_cache() })
    }
}


#[async_trait::async_trait]
impl <
    Capacity: associative_cache::Capacity + Send,
    K: Debug + Hash + Eq + Send + Sync,
    V: Debug + Send + Sync + Clone,
> AsyncCache for AssociativeAsyncCache<Capacity,K,V> {
    type Key = K;
    type Value = V;

    async fn put(&mut self, key: Self::Key, ttl: TtlMode, value: Self::Value)
        -> Result<(), CacheError> where Self: Sized {
        self.int_cache.insert(key, TtlEntry::from(value, ttl, self.default_ttl));
        Ok(())
    }
    //noinspection DuplicatedCode
    async fn get(&mut self, key: &Self::Key)
        -> Result<Option<Self::Value>, CacheError> {
        ttl_entry_to_res(self.int_cache.get(key).map(|v|v.clone()))
    }
}

impl <V: Clone> associative_cache::LruTimestamp for TtlEntry<V> {
    type Timestamp<'a> = std::time::Instant where Self: 'a;

    #[inline]
    fn get_timestamp(&self) -> Self::Timestamp<'_> {
        self.updated_at
    }

    fn update_timestamp(&self) {
        // I do not need it.
    }
}
