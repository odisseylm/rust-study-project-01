use std::collections::HashSet;
use std::hash::Hash;
use std::marker::PhantomData;
use num::PrimInt; // Integer
// use num::Integer;
use super::super::permission::{ PermissionSet, PermissionProcessError };


/*
pub trait AsBitMask <IntType: Integer> {
    fn as_bit(&self) -> IntType;
}
*/

/*
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
*/

pub struct BitsPermissionSet < //
    P: PrimInt + Hash + Sync + Send,
    WP: Into<P> + TryFrom<P,Error=CErr> + Copy + Clone + Eq + Hash + Sync + Send,
    CErr: std::error::Error + Sync + Send,
> where PermissionProcessError: From<CErr> {
    value: P,
    _pd: PhantomData<WP>,
}


impl <
    P: PrimInt + Hash + Sync + Send,
    WP: Into<P> + TryFrom<P,Error=CErr> + Copy + Clone + Eq + Hash + Sync + Send,
    CErr: std::error::Error + Sync + Send,
> PermissionSet for BitsPermissionSet<P,WP,CErr>
    where PermissionProcessError: From<CErr> {
    type Permission = WP;

    #[inline]
    fn has_permission(&self, permission: &Self::Permission) -> bool {
        let self_value: P = self.value.into();
        let perm_bit: P = (*permission).into();
        self_value & perm_bit != P::zero()
    }

    fn to_hash_set(&self) -> Result<HashSet<Self::Permission>, PermissionProcessError> {
        // use BitCounts;
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
    where PermissionProcessError: From<CErr> {
    fn new_perm(value: WP) -> BitsPermissionSet<P,WP,CErr> {
        BitsPermissionSet { value: value.into(), _pd: PhantomData }
    }
    fn new_raw(value: P) -> BitsPermissionSet<P,WP,CErr> {
        BitsPermissionSet { value, _pd: PhantomData }
    }
}




#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::convert::Infallible;
    use crate::util::TestResultUnwrap;
    use super::{ BitsPermissionSet };
    use super::super::super::permission::{ PermissionSet, PermissionProcessError, predefined::Role };


    #[test]
    fn bits_permission_set_for_u32() {
        type U32BitsPermissionSet = BitsPermissionSet<u32, u32, Infallible>;
        type U64BitsPermissionSet = BitsPermissionSet<u64, u64, Infallible>;

        let ps = U32BitsPermissionSet::new();
        assert!(!ps.has_permission(&1));
        assert!(!ps.has_permission(&2));
        assert!(!ps.has_permission(&4));
        assert_eq!(ps.value.count_ones(), 0);

        let ps = U64BitsPermissionSet::from_permission(2);
        assert!(!ps.has_permission(&1));
        assert!(ps.has_permission(&2));
        assert!(!ps.has_permission(&4));
        assert_eq!(ps.value.count_ones(), 1);

        let ps = U32BitsPermissionSet::merge(
            U32BitsPermissionSet::from_permission(2),
            U32BitsPermissionSet::from_permission(8),
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

        type RoleSet = BitsPermissionSet<u32, Role, PermissionProcessError>;

        let ps = RoleSet::new();
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!(!ps.has_permission(&Role::Read));
        assert!(!ps.has_permission(&Role::Write));
        let ps_as_bits: u32 = ps.value.into();
        assert_eq!(ps_as_bits.count_ones(), 0);

        let ps = RoleSet::from_permission(Role::Read);
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!(ps.has_permission(&Role::Read));
        assert!(!ps.has_permission(&Role::Write));
        let ps_as_bits: u32 = ps.value.into();
        assert_eq!(ps_as_bits.count_ones(), 1);

        let ps = RoleSet::merge(
            RoleSet::from_permission(Role::Write),
            RoleSet::from_permission(Role::SuperUser),
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
    fn bits_permission_set_for_enum_2() {

        type RoleSet = BitsPermissionSet<u32, Role, PermissionProcessError>;

        let ps = RoleSet::from_permission(Role::Read);
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!( ps.has_permission(&Role::Read));
        assert!(!ps.has_permission(&Role::Write));
        assert!(!ps.has_permission(&Role::User));
        assert!(!ps.has_permission(&Role::Admin));
        assert!(!ps.has_permission(&Role::SuperUser));

        let ps = RoleSet::from_permission2(Role::Read, Role::Write);
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!( ps.has_permission(&Role::Read));
        assert!( ps.has_permission(&Role::Write));
        assert!(!ps.has_permission(&Role::User));
        assert!(!ps.has_permission(&Role::Admin));
        assert!(!ps.has_permission(&Role::SuperUser));

        let ps = RoleSet::from_permission3(Role::Read, Role::Write, Role::User);
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!( ps.has_permission(&Role::Read));
        assert!( ps.has_permission(&Role::Write));
        assert!( ps.has_permission(&Role::User));
        assert!(!ps.has_permission(&Role::Admin));
        assert!(!ps.has_permission(&Role::SuperUser));

        let ps = RoleSet::from_permission4(Role::Read, Role::Write, Role::User, Role::Admin);
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!( ps.has_permission(&Role::Read));
        assert!( ps.has_permission(&Role::Write));
        assert!( ps.has_permission(&Role::User));
        assert!( ps.has_permission(&Role::Admin));
        assert!(!ps.has_permission(&Role::SuperUser));

        let ps = RoleSet::merge(
            RoleSet::from_permission(Role::Write),
            RoleSet::from_permission(Role::Admin),
        );
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!(!ps.has_permission(&Role::Read));
        assert!( ps.has_permission(&Role::Write));
        assert!(!ps.has_permission(&Role::SuperUser));
        assert!( ps.has_permission(&Role::Admin));
        assert_eq!(ps.value.count_ones(), 2);

        assert_eq!(ps.to_hash_set().test_unwrap(), vec!(Role::Write, Role::Admin).into_iter().collect::<HashSet<Role>>());
        assert_eq!(ps.to_hash_set().test_unwrap(), HashSet::from_iter(vec!(Role::Write, Role::Admin).into_iter()));
        assert_eq!(ps.to_hash_set().test_unwrap(), HashSet::from_iter(vec!(Role::Write, Role::Admin).iter().cloned()));
        assert_eq!(ps.to_hash_set().test_unwrap(), HashSet::from_iter(vec!(Role::Write, Role::Admin)));

        let mut ps = RoleSet::from_permission2(Role::Read, Role::Write);
        ps.merge_with_mut(RoleSet::from_permission(Role::Admin));
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!( ps.has_permission(&Role::Read));
        assert!( ps.has_permission(&Role::Write));
        assert!(!ps.has_permission(&Role::User));
        assert!( ps.has_permission(&Role::Admin));
        assert!(!ps.has_permission(&Role::SuperUser));
    }

}
