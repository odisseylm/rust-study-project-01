use core::fmt;
use core::fmt::{ Binary, Debug };
use std::collections::HashSet;
use std::convert::Infallible;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::size_of;
use num::PrimInt;
use crate::auth::permission::VerifyRequiredPermissionsResult; // Integer
// use num::Integer;
use super::super::permission::{ PermissionSet, PermissionsToHashSet, PermissionProcessError };


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

// bounds on generic parameters are not enforced in type aliases
pub type IntegerBitsPermissionSet<IntType/*: PrimInt*/> = BitsPermissionSet<IntType, IntType, Infallible>;
// pub type U32BitsPermissionSet = IntegerBitsPermissionSet<u32>;
// pub type U64BitsPermissionSet = IntegerBitsPermissionSet<u64>;

// #[derive(Debug)]
// #[derive(Copy, Clone)]
#[derive(Copy)]
pub struct BitsPermissionSet <
    // Usually BitsIntType is u8, u16, u32, u64
    BitsIntType: PrimInt + Binary + Hash + Debug + Clone + Sync + Send,
    // Usually SingleBitPermType it is enum where enum-variant represents one bit.
    SingleBitPermType: Into<BitsIntType> + TryFrom<BitsIntType,Error=ConvertBitToPermTypeError> + Eq + Hash + Copy + Debug + Clone + Sync + Send,
    ConvertBitToPermTypeError: std::error::Error + Sync + Send,
> where PermissionProcessError: From<ConvertBitToPermTypeError> {
    value: BitsIntType,
    _pd: PhantomData<SingleBitPermType>,
}

impl <
    BitsType: PrimInt + Binary + Hash + Debug + Clone + Sync + Send,
    Perm: Into<BitsType> + TryFrom<BitsType,Error=CErr> + Eq + Hash + Copy + Debug + Clone + Sync + Send,
    CErr: std::error::Error + Sync + Send,
> Clone for BitsPermissionSet<BitsType, Perm,CErr>
    where PermissionProcessError: From<CErr> // Or Rust stupid or I am ??!! Why I have to add it there??
{
    fn clone(&self) -> Self {
        Self { value: self.value, _pd: PhantomData }
    }
    fn clone_from(&mut self, source: &Self) {
        self.value = source.value;
    }
}

impl <
    BitsType: PrimInt + Binary + Hash + Debug + Clone + Sync + Send,
    Perm: Into<BitsType> + TryFrom<BitsType,Error=CErr> + Eq + Hash + Copy + Debug + Clone + Sync + Send,
    CErr: std::error::Error + Sync + Send,
> Debug for BitsPermissionSet<BitsType,Perm,CErr>
    where PermissionProcessError: From<CErr> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let as_bin = format!("{:b}", self.value);
        f.debug_struct("BitsPermissionSet")
            .field("value",  &as_bin)
            .finish()
    }
}

impl <
    BitsType: PrimInt + Binary + Hash + Debug + Clone + Sync + Send,
    Perm: Into<BitsType> + TryFrom<BitsType,Error=CErr> + Eq + Hash + Copy + Debug + Clone + Sync + Send,
    CErr: std::error::Error + Sync + Send,
> PermissionSet for BitsPermissionSet<BitsType,Perm,CErr>
    where PermissionProcessError: From<CErr> {
    type Permission = Perm;

    #[inline]
    fn has_permission(&self, permission: &Self::Permission) -> bool {
        let self_value: BitsType = self.value.into();
        let perm_bit  : BitsType = (*permission).into();
        self_value & perm_bit != BitsType::zero()
    }

    fn is_empty(&self) -> bool {
        self.value == BitsType::zero()
    }

    #[inline]
    fn new() -> Self {
        Self::new_raw(BitsType::zero())
    }

    #[inline]
    fn from_permission(permission: Self::Permission) -> Self {
        Self::new_perm(permission)
    }

    #[inline]
    fn from_permission2(perm1: Self::Permission, perm2: Self::Permission) -> Self {
        let res: BitsType = perm1.into() | perm2.into();
        Self::new_raw(res)
    }

    #[inline]
    fn from_permission3(perm1: Self::Permission, perm2: Self::Permission, perm3: Self::Permission) -> Self {
        let res: BitsType = perm1.into() | perm2.into() | perm3.into();
        Self::new_raw(res)
    }

    #[inline]
    fn from_permission4(perm1: Self::Permission, perm2: Self::Permission, perm3: Self::Permission, perm4: Self::Permission) -> Self {
        let res: BitsType = perm1.into() | perm2.into() | perm3.into() | perm4.into();
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

    fn verify_required_permissions(&self, required_permissions: Self)
        -> Result<VerifyRequiredPermissionsResult<Self::Permission,Self>, PermissionProcessError> {

        let and_bits = self.value & required_permissions.value;
        // if and_bits == BitsType::zero() {
        if and_bits == required_permissions.value {
            return Ok(VerifyRequiredPermissionsResult::RequiredPermissionsArePresent);
        }

        let absent_bits = and_bits ^ required_permissions.value;

        let absent = if absent_bits.count_zeros() == 1 {
            let absent_perm = Perm::try_from(absent_bits) ?;
            VerifyRequiredPermissionsResult::NoPermission(absent_perm)
        } else {
            VerifyRequiredPermissionsResult::NoPermissions(BitsPermissionSet::<BitsType,Perm,CErr>::new_raw(absent_bits))
        };
        Ok(absent)
    }
}

impl <
    BitsType: PrimInt + Binary + Debug + Hash + Sync + Send,
    Perm: Into<BitsType> + TryFrom<BitsType,Error=CErr> + Debug + Copy + Clone + Eq + Hash + Sync + Send,
    CErr: std::error::Error + Sync + Send,
> PermissionsToHashSet for BitsPermissionSet<BitsType,Perm,CErr>
    where PermissionProcessError: From<CErr> {
    type Permission = Perm;

    fn to_hash_set(&self) -> Result<HashSet<Self::Permission>, PermissionProcessError> {
        // use BitCounts;
        let count = self.value.count_ones();
        if count == 0 {
            return Ok(HashSet::new())
        }

        use core::mem::size_of;
        let value: BitsType = self.value.into();

        let mut as_set = HashSet::<Self::Permission>::with_capacity(count as usize);
        for i in 0..size_of::<Self::Permission>()*8 {
            let as_bit = BitsType::one() << i;
            if (as_bit & value) != BitsType::zero() {
                let bit_perm_obj: Self::Permission = Self::Permission::try_from(as_bit) ?;
                as_set.insert(bit_perm_obj);
            }
        }
        Ok(as_set)
    }
}


impl <
    BitsType: PrimInt + Binary + Debug + Hash + Sync + Send,
    Perm: Into<BitsType> + TryFrom<BitsType,Error=CErr> + Debug + Copy + Clone + Eq + Hash + Sync + Send,
    CErr: std::error::Error + Sync + Send,
> BitsPermissionSet<BitsType,Perm,CErr>
    where PermissionProcessError: From<CErr> {
    fn new_perm(value: Perm) -> BitsPermissionSet<BitsType,Perm,CErr> {
        BitsPermissionSet { value: value.into(), _pd: PhantomData }
    }
    fn new_raw(value: BitsType) -> BitsPermissionSet<BitsType,Perm,CErr> {
        BitsPermissionSet { value, _pd: PhantomData }
    }
}

impl <
    BitsType: PrimInt + Binary + Hash + Debug + Clone + Sync + Send,
    Perm: Into<BitsType> + TryFrom<BitsType,Error=CErr> + Eq + Hash + Copy + Debug + Clone + Sync + Send,
    CErr: std::error::Error + Sync + Send,
> fmt::Display for BitsPermissionSet<BitsType,Perm,CErr>
    where PermissionProcessError: From<CErr>, // TODO: !!! Why I have to put it there??? This clarification is not needed there!!!
          Perm: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BitsPermissionSet {{ ") ?;

        let value = self.value;
        let mut some_is_printed = false;
        for i in 0..size_of::<Perm>()*8 {
            let as_bit = BitsType::one() << i;
            if (as_bit & value) != BitsType::zero() {
                let bit_perm_obj: Result<Perm, _> = Perm::try_from(as_bit);
                if some_is_printed { write!(f, ", ") ?; }
                match bit_perm_obj {
                    Ok(perm) => {
                        write!(f, "{}", perm) ?;
                    }
                    Err(_) => {
                        write!(f, "Unexpected bit [{:b}]", as_bit) ?;
                    }
                };
                some_is_printed = true;
            }
        }
        write!(f, " }}")
    }
}





#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::convert::Infallible;
    use crate::util::TestResultUnwrap;
    use super::{ BitsPermissionSet };
    use super::super::super::permission::{ PermissionSet, PermissionsToHashSet, PermissionProcessError, predefined::Role };


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
