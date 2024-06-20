use anyhow::anyhow;
use super::super::{
    permission::{ PermissionProcessError },
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(strum_macros::EnumIter, strum_macros::FromRepr)]
// #[derive(sqlx::FromRow)]
#[repr(u32)]
#[non_exhaustive]
pub enum Role {
    // Unknown   = 0,
    Anonymous = 1 << 0,
    Read      = 1 << 1,
    Write     = 1 << 2,
    User      = 1 << 3,
    Admin     = 1 << 4,
    SuperUser = 1 << 5,
    // !!! Other a few variants can appear in the future. !!!

    //
    // If you need 'match' (by some reason...) for this enum, please create your own enum
    // and use it instead of this enum.
}


impl Into<u32> for Role {
    #[inline(always)]
    fn into(self) -> u32 { self as u32 }
}


// It would be nice to directly implement
//  * impl TryInto<HashSet<Role>> for u32 { }
//  * impl TryFrom<u32> for HashSet<Role> { }
//
// but our lovely std rust error 'only traits defined in the current crate can be implemented for types defined outside of the crate' :-(


impl TryFrom<u32> for Role {
    type Error = PermissionProcessError;
    #[inline]
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let as_role: Option<Role> = Role::from_repr(value);
        as_role.ok_or_else(||PermissionProcessError::ConvertError(anyhow!("No Role for [{}]", value)))
    }
}


/*
#[inherent::inherent]
impl AsBitMask<u32> for Role {
    pub fn as_bit(&self) -> u32 {
        *self as u32
    }
}
*/


/*
Alternatives:

1) pub mod PublicFlags {
    pub const PublicFlagVersion: u8 = 0x01;
    pub const PublicFlagReset: u8 = 0x02;
    pub const NoncePresent: u8 = 0x04;
    pub const IdPresent: u8 = 0x08;
    pub const PktNumLen4: u8 = 0x30;
    pub const PktNumLen2: u8 = 0x20;
    pub const PktNumLen1: u8 = 0x10;
    pub const Multipath: u8 = 0x40;
}


2) bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct PublicFlags: u8 {
        const PUBLIC_FLAG_VERSION = 0x01;
        const PUBLIC_FLAG_RESET = 0x02;
        const NONCE_PRESENT = 0x04;
        const ID_PRESENT = 0x08;
        const PKT_NUM_LEN_4 = 0x30;
        const PKT_NUM_LEN_2 = 0x20;
        const PKT_NUM_LEN_1 = 0x10;
        const MULTIPATH = 0x40;
    }
}

fn main() {
    let flag = PublicFlags::PUBLIC_FLAG_VERSION | PublicFlags::ID_PRESENT;
    assert!((flag & PublicFlags::MULTIPATH).is_empty());
    assert!(flag.contains(PublicFlags::ID_PRESENT));
}
*/
