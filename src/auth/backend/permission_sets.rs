use std::collections::HashSet;
use std::hash::Hash;
use std::marker::PhantomData;
use num::{ Integer, PrimInt };
use super::authz_backend::{PermissionFormatError, PermissionSet};


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


pub struct BitsPermissionSet < //
    P: PrimInt + Hash + Sync + Send,
    WP: Into<P> + TryFrom<P,Error=CErr> + Copy + Clone + Eq + Hash + Sync + Send,
    CErr: std::error::Error + Sync + Send,
> where PermissionFormatError: From<CErr> {
    value: P,
    _pd: PhantomData<WP>,
}


impl <
    P: PrimInt + Hash + Sync + Send,
    WP: Into<P> + TryFrom<P,Error=CErr> + Copy + Clone + Eq + Hash + Sync + Send,
    CErr: std::error::Error + Sync + Send,
> PermissionSet for BitsPermissionSet<P,WP,CErr>
    where PermissionFormatError: From<CErr> {
    type Permission = WP;

    #[inline]
    fn has_permission(&self, permission: &Self::Permission) -> bool {
        let self_value: P = self.value.into();
        let perm_bit: P = (*permission).into();
        self_value & perm_bit != P::zero()
    }

    fn to_hash_set(&self) -> Result<HashSet<Self::Permission>, PermissionFormatError> {
        use BitCounts;
        let count = self.value.count_ones();
        if count == 0 {
            return Ok(HashSet::new())
        }

        use core::mem::size_of;
        let value: P = self.value.into();

        let mut as_set = HashSet::<Self::Permission>::with_capacity(count as usize);
        for i in 0..size_of::<Self::Permission>()*8 {
            let as_bit = P::one() << i;
            if (as_bit & value) != P::zero() {
                let bit_perm_obj: Self::Permission = Self::Permission::try_from(as_bit) ?;
                as_set.insert(bit_perm_obj);
            }
        }
        Ok(as_set)
    }

    #[inline]
    fn new() -> Self {
        Self::new_raw(P::zero())
    }

    #[inline]
    fn from_permission(permission: Self::Permission) -> Self {
        Self::new_perm(permission)
    }

    #[inline]
    fn from_permission2(perm1: Self::Permission, perm2: Self::Permission) -> Self {
        let res: P = perm1.into() | perm2.into();
        Self::new_raw(res)
    }

    #[inline]
    fn from_permission3(perm1: Self::Permission, perm2: Self::Permission, perm3: Self::Permission) -> Self {
        let res: P = perm1.into() | perm2.into() | perm3.into();
        Self::new_raw(res)
    }

    #[inline]
    fn from_permission4(perm1: Self::Permission, perm2: Self::Permission, perm3: Self::Permission, perm4: Self::Permission) -> Self {
        let res: P = perm1.into() | perm2.into() | perm3.into() | perm4.into();
        Self::new_raw(res)
    }

    #[inline]
    fn merge_with_mut(&mut self, another: Self) {
        self.value = self.value | another.value;
    }

    #[inline]
    fn merge(set1: Self, set2: Self) -> Self {
        Self::new_raw(set1.value | set2.value)
    }
}


impl <
    P: PrimInt + Hash + Sync + Send,
    WP: Into<P> + TryFrom<P,Error=CErr> + Copy + Clone + Eq + Hash + Sync + Send,
    CErr: std::error::Error + Sync + Send,
> BitsPermissionSet<P,WP,CErr>
    where PermissionFormatError: From<CErr> {
    fn new_perm(value: WP) -> BitsPermissionSet<P,WP,CErr> {
        BitsPermissionSet { value: value.into(), _pd: PhantomData }
    }
    fn new_raw(value: P) -> BitsPermissionSet<P,WP,CErr> {
        BitsPermissionSet { value, _pd: PhantomData }
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

    fn to_hash_set(&self) -> Result<HashSet<Self::Permission>, PermissionFormatError> {
        Ok(self.0.clone())
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
    use std::convert::Infallible;
    use crate::auth::backend::authz_backend::{PermissionFormatError, PermissionSet};
    use crate::auth::backend::std_authz_backend::{ Role };
    use super::{ BitsPermissionSet, HashPermissionSet };

    #[test]
    fn bits_permission_set_for_u32() {

        let ps = BitsPermissionSet::<u32,u32,Infallible>::new();
        assert!(!ps.has_permission(&1));
        assert!(!ps.has_permission(&2));
        assert!(!ps.has_permission(&4));
        assert_eq!(ps.value.count_ones(), 0);

        let ps = BitsPermissionSet::<u64,u64,Infallible>::from_permission(2);
        assert!(!ps.has_permission(&1));
        assert!(ps.has_permission(&2));
        assert!(!ps.has_permission(&4));
        assert_eq!(ps.value.count_ones(), 1);

        let ps = BitsPermissionSet::<u32,u32,Infallible>::merge(
            BitsPermissionSet::<u32,u32,Infallible>::from_permission(2),
            BitsPermissionSet::<u32,u32,Infallible>::from_permission(8),
        );
        assert!(!ps.has_permission(&1));
        assert!(ps.has_permission(&2));
        assert!(!ps.has_permission(&4));
        assert!(ps.has_permission(&8));
        assert!(!ps.has_permission(&16));
        assert_eq!(ps.value.count_ones(), 2);
    }

    #[test]
    fn bits_permission_set_for_enum() {

        let ps = BitsPermissionSet::<u32, Role, PermissionFormatError>::new();
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!(!ps.has_permission(&Role::Read));
        assert!(!ps.has_permission(&Role::Write));
        let ps_as_bits: u32 = ps.value.into();
        assert_eq!(ps_as_bits.count_ones(), 0);

        let ps = BitsPermissionSet::<u32, Role, PermissionFormatError>::from_permission(Role::Read);
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!(ps.has_permission(&Role::Read));
        assert!(!ps.has_permission(&Role::Write));
        let ps_as_bits: u32 = ps.value.into();
        assert_eq!(ps_as_bits.count_ones(), 1);

        let ps = BitsPermissionSet::<u32, Role, PermissionFormatError>::merge(
            BitsPermissionSet::<u32, Role, PermissionFormatError>::from_permission(Role::Write),
            BitsPermissionSet::<u32, Role, PermissionFormatError>::from_permission(Role::SuperUser),
        );
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!(!ps.has_permission(&Role::Read));
        assert!(ps.has_permission(&Role::Write));
        assert!(ps.has_permission(&Role::SuperUser));
        assert!(!ps.has_permission(&Role::Admin));
        let ps_as_bits: u32 = ps.value.into();
        assert_eq!(ps_as_bits.count_ones(), 2);
    }

    #[test]
    fn hash_permission_set_for_u32() {

        let ps = HashPermissionSet::<u32>::new();
        assert!(!ps.has_permission(&1));
        assert!(!ps.has_permission(&2));
        assert!(!ps.has_permission(&4));

        let ps = HashPermissionSet::<u32>::from_permission(2);
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
