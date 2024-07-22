
// For universal usage, since seems maps have no the same API...
pub trait MapOpsExt {
    type Key;
    type Value;
    type Map;

    fn first_entry(&self) -> Option<(&Self::Key, &Self::Value)>;
}

impl <K,V> MapOpsExt for indexmap::IndexMap<K,V> {
    type Key = K;
    type Value = V;
    type Map = indexmap::IndexMap<K,V>;

    #[inline]
    fn first_entry(&self) -> Option<(&Self::Key, &Self::Value)> {
        self.first()
    }
}

impl <K: Ord,V> MapOpsExt for std::collections::BTreeMap<K,V> {
    type Key = K;
    type Value = V;
    type Map = indexmap::IndexMap<K,V>;

    #[inline]
    fn first_entry(&self) -> Option<(&Self::Key, &Self::Value)> {
        self.first_key_value()
    }
}
