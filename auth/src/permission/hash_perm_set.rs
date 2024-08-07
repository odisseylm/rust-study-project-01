use std::collections::HashSet;
use core::fmt::{ self, Debug, Display};
use core::hash::Hash;
use std::borrow::Borrow;

use crate::{
    util::fmt::iterable_to_display,
    permission::{ PermissionSet, PermissionsToHashSet, VerifyRequiredPermissionsResult, PermissionProcessError },
};


#[derive(Clone, Debug)]
pub struct HashPermissionSet<Perm: Clone + Debug + Eq + Hash>(HashSet<Perm>);

impl <Perm: Clone + Debug + Eq + Hash + Send + Sync> HashPermissionSet<Perm> {
    #[inline]
    #[allow(dead_code)] // it is optional method and is not used in prod code (sine HashPermissionSet is not used)
    fn has_permission<Q: ?Sized>(&self, permission: &Q) -> bool
        where
            Perm: Borrow<Q>,
            Q: Hash + Eq,    {
        self.0.contains(permission)
    }
}

impl <Perm: Clone + Debug + Eq + Hash + Send + Sync> PermissionSet for HashPermissionSet<Perm> {
    type Permission = Perm;

    #[inline]
    fn has_permission(&self, permission: &Self::Permission) -> bool {
        self.0.contains(permission)
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    fn new() -> Self {
        HashPermissionSet(HashSet::<Perm>::new())
    }

    #[inline]
    fn from_permission(permission: Self::Permission) -> Self {
        let mut set = HashSet::<Perm>::with_capacity(1);
        set.insert(permission);
        HashPermissionSet(set)
    }

    fn from_permissions<const N: usize>(permissions: [Self::Permission; N]) -> Self {
        let mut set = HashSet::<Perm>::with_capacity(N);
        for perm in permissions {
            set.insert(perm);
        }
        HashPermissionSet(set)
    }

    /*
    #[inline]
    fn from_permission2(perm1: Self::Permission, perm2: Self::Permission) -> Self {
        let mut set = HashSet::<Perm>::with_capacity(1);
        set.insert(perm1);
        set.insert(perm2);
        HashPermissionSet(set)
    }

    #[inline]
    fn from_permission3(perm1: Self::Permission, perm2: Self::Permission, perm3: Self::Permission) -> Self {
        let mut set = HashSet::<Perm>::with_capacity(1);
        set.insert(perm1);
        set.insert(perm2);
        set.insert(perm3);
        HashPermissionSet(set)
    }

    #[inline]
    fn from_permission4(perm1: Self::Permission, perm2: Self::Permission, perm3: Self::Permission, perm4: Self::Permission) -> Self {
        let mut set = HashSet::<Perm>::with_capacity(1);
        set.insert(perm1);
        set.insert(perm2);
        set.insert(perm3);
        set.insert(perm4);
        HashPermissionSet(set)
    }
    */

    #[inline]
    fn merge_with_mut(&mut self, another: Self) {
        for perm in another.0 {
            self.0.insert(perm);
        }
    }

    #[inline]
    fn merge(set1: Self, set2: Self) -> Self {
        let mut set = HashSet::<Perm>::with_capacity(set1.0.len() + set2.0.len());
        for perm in set1.0 {
            set.insert(perm);
        }
        for perm in set2.0 {
            set.insert(perm);
        }
        HashPermissionSet(set)
    }

    fn verify_required_permissions(&self, required_permissions: Self)
        -> Result<VerifyRequiredPermissionsResult<Self>, PermissionProcessError> {

        let missed = required_permissions.0.into_iter()
            .filter(|req|!self.0.contains(&req))
            .collect::<HashSet<Self::Permission>>();

        if missed.is_empty() {
            Ok(VerifyRequiredPermissionsResult::RequiredPermissionsArePresent)
        } else {
            Ok(VerifyRequiredPermissionsResult::NoPermissions(HashPermissionSet(missed)))
        }
    }
}


impl <P: Clone + Debug + Eq + Hash + Send + Sync> PermissionsToHashSet for HashPermissionSet<P> {
    type Permission = P;
    fn to_hash_set(&self) -> Result<HashSet<Self::Permission>, PermissionProcessError> {
        Ok(self.0.clone())
    }
}


impl <P: Display + Clone + Debug + Eq + Hash + Send + Sync> Display
    for HashPermissionSet<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // iter_to_display(self.0.iter(), "Permissions (set)", f)
        iterable_to_display(&self.0, "Permissions (set)", f)
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use super::HashPermissionSet;
    use crate::permission::{ PermissionSet, PermissionsToHashSet, predefined::Role, };
    use crate::test::TestResultUnwrap;
    use crate::util::test_unwrap::TestSringOps;

    #[test]
    fn hash_permission_set_for_u32() {

        let ps = HashPermissionSet::<u32>::new();
        assert!(!ps.has_permission(&1));
        assert!(!ps.has_permission(&2));
        assert!(!ps.has_permission(&4));

        let ps = HashPermissionSet::<u32>::from_permission(2);
        assert!(!ps.has_permission(&1));
        assert!( ps.has_permission(&2));
        assert!(!ps.has_permission(&4));

        let ps = HashPermissionSet::<u32>::merge(HashPermissionSet::<u32>::from_permission(2), HashPermissionSet::<u32>::from_permission(8));
        assert!(!ps.has_permission(&1));
        assert!( ps.has_permission(&2));
        assert!(!ps.has_permission(&4));
        assert!( ps.has_permission(&8));
        assert!(!ps.has_permission(&16));
        assert_eq!(ps.0.len(), 2);
    }


    #[test]
    fn hash_permission_set_for_string() {

        let ps = HashPermissionSet::<String>::new();
        assert!(!ps.has_permission("1"));
        assert!(!ps.has_permission(&"1".to_test_string()));
        assert!(!ps.has_permission("2"));
        assert!(!ps.has_permission("4"));

        let ps = HashPermissionSet::<String>::from_permission("2".to_test_string());
        assert!(! <HashPermissionSet<String> as PermissionSet>::has_permission(&ps, &"1".to_test_string()));
        // assert!(! <HashPermissionSet<String> as PermissionSet>::has_permission(&ps, &"1"));
        assert!(!ps.has_permission(&"1".to_test_string()));
        assert!(!ps.has_permission("1"));
        assert!( ps.has_permission("2"));
        assert!(!ps.has_permission("4"));

        let ps = HashPermissionSet::<String>::merge(
            HashPermissionSet::<String>::from_permission("2".to_test_string()),
            HashPermissionSet::<String>::from_permission("8".to_test_string()),
        );
        assert!(!ps.has_permission("1"));
        assert!( ps.has_permission("2"));
        assert!(!ps.has_permission("4"));
        assert!( ps.has_permission("8"));
        assert!(!ps.has_permission("16"));
        assert_eq!(ps.0.len(), 2);
    }


    #[test]
    fn hash_permission_set_for_str() {

        let ps = HashPermissionSet::<&'static str>::new();
        assert!(!ps.has_permission(&"1"));
        assert!(!ps.has_permission(&"2"));
        assert!(!ps.has_permission(&"4"));

        let ps = HashPermissionSet::<&'static str>::from_permission("2");
        assert!(!ps.has_permission(&"1"));
        assert!( ps.has_permission(&"2"));
        assert!(!ps.has_permission(&"4"));

        let ps = HashPermissionSet::<&'static str>::merge(
            HashPermissionSet::<&'static str>::from_permission("2"),
            HashPermissionSet::<&'static str>::from_permission("8"),
        );
        assert!(!ps.has_permission(&"1"));
        assert!( ps.has_permission(&"2"));
        assert!(!ps.has_permission(&"4"));
        assert!( ps.has_permission(&"8"));
        assert!(!ps.has_permission(&"16"));
        assert_eq!(ps.0.len(), 2);
    }


    #[test]
    fn hash_permission_set_for_role() {

        let ps = HashPermissionSet::<Role>::new();
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!(!ps.has_permission(&Role::Read));
        assert!(!ps.has_permission(&Role::Write));

        let ps = HashPermissionSet::<Role>::from_permission(Role::Read);
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!( ps.has_permission(&Role::Read));
        assert!(!ps.has_permission(&Role::Write));
        assert!(!ps.has_permission(&Role::User));
        assert!(!ps.has_permission(&Role::Admin));
        assert!(!ps.has_permission(&Role::SuperUser));

        let ps = HashPermissionSet::<Role>::from_permissions([Role::Read, Role::Write]);
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!( ps.has_permission(&Role::Read));
        assert!( ps.has_permission(&Role::Write));
        assert!(!ps.has_permission(&Role::User));
        assert!(!ps.has_permission(&Role::Admin));
        assert!(!ps.has_permission(&Role::SuperUser));

        let ps = HashPermissionSet::<Role>::from_permissions([Role::Read, Role::Write, Role::User]);
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!( ps.has_permission(&Role::Read));
        assert!( ps.has_permission(&Role::Write));
        assert!( ps.has_permission(&Role::User));
        assert!(!ps.has_permission(&Role::Admin));
        assert!(!ps.has_permission(&Role::SuperUser));

        let ps = HashPermissionSet::<Role>::from_permissions(
            [Role::Read, Role::Write, Role::User, Role::Admin]);
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!( ps.has_permission(&Role::Read));
        assert!( ps.has_permission(&Role::Write));
        assert!( ps.has_permission(&Role::User));
        assert!( ps.has_permission(&Role::Admin));
        assert!(!ps.has_permission(&Role::SuperUser));

        let ps = HashPermissionSet::<Role>::merge(
            HashPermissionSet::<Role>::from_permission(Role::Write),
            HashPermissionSet::<Role>::from_permission(Role::Admin),
        );
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!(!ps.has_permission(&Role::Read));
        assert!( ps.has_permission(&Role::Write));
        assert!(!ps.has_permission(&Role::SuperUser));
        assert!( ps.has_permission(&Role::Admin));
        assert_eq!(ps.0.len(), 2);

        assert_eq!(ps.to_hash_set().test_unwrap(), vec!(Role::Write, Role::Admin).into_iter().collect::<HashSet<Role>>());
        assert_eq!(ps.to_hash_set().test_unwrap(), HashSet::from_iter(vec!(Role::Write, Role::Admin).into_iter()));
        assert_eq!(ps.to_hash_set().test_unwrap(), HashSet::from_iter(vec!(Role::Write, Role::Admin).iter().cloned()));
        assert_eq!(ps.to_hash_set().test_unwrap(), HashSet::from_iter(vec!(Role::Write, Role::Admin)));

        let mut ps = HashPermissionSet::<Role>::from_permissions([Role::Read, Role::Write]);
        ps.merge_with_mut(HashPermissionSet::<Role>::from_permission(Role::Admin));
        assert!(!ps.has_permission(&Role::Anonymous));
        assert!( ps.has_permission(&Role::Read));
        assert!( ps.has_permission(&Role::Write));
        assert!(!ps.has_permission(&Role::User));
        assert!( ps.has_permission(&Role::Admin));
        assert!(!ps.has_permission(&Role::SuperUser));
    }
}
