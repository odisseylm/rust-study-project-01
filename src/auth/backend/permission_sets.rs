use std::collections::HashSet;
use std::hash::Hash;
use num::{ Integer, PrimInt };
use super::authz_backend::PermissionSet;


pub trait AsBitMask <IntType: Integer> {
    fn as_bit(&self) -> IntType;
}


trait BitCounts {
    fn call_count_ones(self) -> u32;
}

impl BitCounts for u32 {
    #[inline]
    fn call_count_ones(self) -> u32 {
        // core::num::Wrapping  core::num::bignum::
        self.count_ones()
    }
}
impl BitCounts for u64 {
    #[inline]
    fn call_count_ones(self) -> u32 {
        // core::num::Wrapping  core::num::bignum::
        self.count_ones()
    }
}


// pub struct BitsPermissionSet<P: Integer + BitAnd + BitOr + BitCounts + Copy + Clone + Eq + Hash + Sync + Send>(P);
pub struct BitsPermissionSet<P: PrimInt + Hash + Sync + Send>(P);

// impl<P: Integer + BitAnd + BitOr + BitCounts + Copy + Clone + Eq + Hash + Sync + Send> PermissionSet for BitsPermissionSet<P>
impl<P: PrimInt + Hash + Sync + Send> PermissionSet for BitsPermissionSet<P>
    // where <P as BitAnd>::Output: PartialEq<P>,
{
    type Permission = P;

    #[inline]
    fn has_permission(&self, permission: &Self::Permission) -> bool {
        self.0 & *permission != P::zero()

        // let v1: P = self.0;
        // let v2: P = *permission;
        // let zero: P = P::zero();
        //
        // let and_res = v1 & v2;
        // let is_ok = !(and_res == zero);
        // is_ok
    }

    fn to_hash_set(&self) -> HashSet<Self::Permission> {
        use core::mem::size_of;
        let mut as_set = HashSet::<Self::Permission>::with_capacity(self.0.count_ones() as usize);
        for i in 0..size_of::<Self::Permission>()*8 {
            let as_bit = P::one() << i;
            if (as_bit & self.0) != P::zero() {
                as_set.insert(as_bit as Self::Permission);
            }
        }
        as_set
    }

    #[inline]
    fn new() -> Self {
        BitsPermissionSet(P::zero())
    }

    #[inline]
    fn from_permission(permission: Self::Permission) -> Self {
        BitsPermissionSet(permission)
    }

    #[inline]
    fn from_permission2(perm1: Self::Permission, perm2: Self::Permission) -> Self {
        let res: Self::Permission = perm1 | perm2;
        BitsPermissionSet(res)
    }

    #[inline]
    fn from_permission3(perm1: Self::Permission, perm2: Self::Permission, perm3: Self::Permission) -> Self {
        let res: Self::Permission = perm1 | perm2 | perm3;
        BitsPermissionSet(res)
    }

    #[inline]
    fn from_permission4(perm1: Self::Permission, perm2: Self::Permission, perm3: Self::Permission, perm4: Self::Permission) -> Self {
        let res: Self::Permission = perm1 | perm2 | perm3 | perm4;
        BitsPermissionSet(res)
    }

    #[inline]
    fn merge_with_mut(&mut self, another: Self) {
        self.0 = self.0 | another.0;
    }

    #[inline]
    fn merge(set1: Self, set2: Self) -> Self {
        BitsPermissionSet(set1.0 | set2.0)
    }
}


#[derive(Clone, Debug)]
pub struct HashPermissionSet<P: Clone + core::fmt::Debug + Eq + Hash>(HashSet<P>);

impl <P: Clone + core::fmt::Debug + Eq + Hash + Send + Sync> PermissionSet for HashPermissionSet<P> {
    type Permission = P;

    #[inline]
    fn has_permission(&self, permission: &Self::Permission) -> bool {
        self.0.contains(permission)
    }

    fn to_hash_set(&self) -> HashSet<Self::Permission> {
        self.0.clone()
    }

    #[inline]
    fn new() -> Self {
        HashPermissionSet(HashSet::<P>::new())
    }

    #[inline]
    fn from_permission(permission: Self::Permission) -> Self {
        let mut set = HashSet::<P>::with_capacity(1);
        set.insert(permission);
        HashPermissionSet(set)
    }

    #[inline]
    fn from_permission2(perm1: Self::Permission, perm2: Self::Permission) -> Self {
        let mut set = HashSet::<P>::with_capacity(1);
        set.insert(perm1);
        set.insert(perm2);
        HashPermissionSet(set)
    }

    #[inline]
    fn from_permission3(perm1: Self::Permission, perm2: Self::Permission, perm3: Self::Permission) -> Self {
        let mut set = HashSet::<P>::with_capacity(1);
        set.insert(perm1);
        set.insert(perm2);
        set.insert(perm3);
        HashPermissionSet(set)
    }

    #[inline]
    fn from_permission4(perm1: Self::Permission, perm2: Self::Permission, perm3: Self::Permission, perm4: Self::Permission) -> Self {
        let mut set = HashSet::<P>::with_capacity(1);
        set.insert(perm1);
        set.insert(perm2);
        set.insert(perm3);
        set.insert(perm4);
        HashPermissionSet(set)
    }

    #[inline]
    fn merge_with_mut(&mut self, another: Self) {
        for perm in another.0 {
            self.0.insert(perm);
        }
    }

    #[inline]
    fn merge(set1: Self, set2: Self) -> Self {
        let mut set = HashSet::<P>::with_capacity(set1.0.len() + set2.0.len());
        for perm in set1.0 {
            set.insert(perm);
        }
        for perm in set2.0 {
            set.insert(perm);
        }
        HashPermissionSet(set)
    }
}


#[cfg(test)]
mod tests {
    use crate::auth::backend::authz_backend::{ PermissionSet };
    use super::{ BitsPermissionSet, HashPermissionSet };

    #[test]
    fn bits_permission_set_for_u32() {

        let ps = BitsPermissionSet::<u32>::new();
        assert!(!ps.has_permission(&1));
        assert!(!ps.has_permission(&2));
        assert!(!ps.has_permission(&4));
        assert_eq!(ps.0.count_ones(), 0);

        let ps = BitsPermissionSet::<u64>::from_permission(2);
        assert!(!ps.has_permission(&1));
        assert!(ps.has_permission(&2));
        assert!(!ps.has_permission(&4));
        assert_eq!(ps.0.count_ones(), 1);

        let ps = BitsPermissionSet::<u32>::merge(BitsPermissionSet::<u32>::from_permission(2), BitsPermissionSet::<u32>::from_permission(8));
        assert!(!ps.has_permission(&1));
        assert!(ps.has_permission(&2));
        assert!(!ps.has_permission(&4));
        assert!(ps.has_permission(&8));
        assert!(!ps.has_permission(&16));
        assert_eq!(ps.0.count_ones(), 2);
    }

    #[test]
    fn hash_permission_set_for_u32() {

        let ps = HashPermissionSet::<u32>::new();
        assert!(!ps.has_permission(&1));
        assert!(!ps.has_permission(&2));
        assert!(!ps.has_permission(&4));

        let ps = BitsPermissionSet::from_permission(2);
        assert!(!ps.has_permission(&1));
        assert!(ps.has_permission(&2));
        assert!(!ps.has_permission(&4));

        let ps = HashPermissionSet::<u32>::merge(HashPermissionSet::<u32>::from_permission(2), HashPermissionSet::<u32>::from_permission(8));
        assert!(!ps.has_permission(&1));
        assert!(ps.has_permission(&2));
        assert!(!ps.has_permission(&4));
        assert!(ps.has_permission(&8));
        assert!(!ps.has_permission(&16));
        assert_eq!(ps.0.len(), 2);
    }
}
