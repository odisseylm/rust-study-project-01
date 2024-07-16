use chrono::Utc;
use mvv_common::entity::amount::Amount;
use mvv_common::generate_from_str_new_type_delegate;
use crate::entity::ClientId;
// -------------------------------------------------------------------------------------------------



#[derive(Debug, Clone, PartialEq, derive_more::Display)] // derive_more::FromStr)]
#[display(fmt = "{}", _0)]
pub struct AccountId( #[allow(dead_code)] uuid::Uuid);
// pub type AccountIdFormatError = mvv_common::entity::id::parse::IdFormatError;
pub type AccountIdFormatError = uuid::Error;

impl AccountId {
    pub fn into_inner(self) -> uuid::Uuid { self.0 }
}

// generate_from_str_new_type_delegate! { AccountId, Id, AccountIdFormatError }
generate_from_str_new_type_delegate! { AccountId, uuid::Uuid, parse_str, uuid::Error }


#[derive(Debug)]
#[readonly::make]
pub struct Account {
    pub id: AccountId, // internal ID
    pub iban: iban::Iban,
    pub client_id: ClientId,
    pub name: String, // TODO: Use type AccountName
    pub amount: Amount,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

pub struct AccountParts {
    pub id: AccountId, // internal ID
    pub iban: iban::Iban,
    pub client_id: ClientId,
    pub name: String,
    pub amount: Amount,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}


impl Account {
    #[inline]
    pub fn new(args: new::Args) -> Self {
        Account {
            id: args.id,
            iban: args.iban,
            client_id: args.client_id,
            name: args.name,
            amount: args.amount,
            created_at: args.created_at,
            updated_at: args.updated_at,
        }
    }
}


impl Account {
    pub fn into_parts(self) -> AccountParts {
        AccountParts {
            id: self.id,
            iban: self.iban,
            client_id: self.client_id,
            name: self.name,
            amount: self.amount,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}


// Attempt to emulate named args in rust.
pub mod new {
    pub type Args = super::AccountParts;
}


/*
#[allow(non_camel_case_types)]
#[derive(Debug)]
#[readonly::make]
pub struct SSS_RO {
    pub x: i32,
}


#[allow(non_camel_case_types)]
#[derive(Debug)]
// #[readonly::make]
pub struct SSS_RO2 {
    pub x: i32,
}
*/


/*
Generated code by 'readonly'

#[cfg(not(doc))]
#[repr(C)]
pub struct Account {
    id: Id,
    user_id: Id,
    amount: Amount,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

const _: () = {
    #[doc(hidden)]
    #[repr(C)]
    pub struct ReadOnlyAccount {
        pub id: Id,
        user_id: Id,
        amount: Amount,
        created_at: chrono::DateTime<Utc>,
        updated_at: chrono::DateTime<Utc>,
    }
    #[doc(hidden)]
    impl core::ops::Deref for Account {
        type Target = ReadOnlyAccount;
        fn deref(&self) -> &Self::Target {
            unsafe { &*(self as *const Self as *const Self::Target) }
        }
    }
};
*/


/*
#[cfg(test)]
mod tests {
    use chrono::{ FixedOffset, Utc };
    use crate::entity::account::{ Account, AccountId, new };
    use mvv_common::entity::amount::Amount;
    use crate::entity::user::UserId;
    use mvv_common::test::TestResultUnwrap;

    fn _aa() {
        let account = Account::new(new::Args {
            id: AccountId::from_str("1").test_unwrap(),
            user_id: UserId::from_str("2").test_unwrap(),
            amount: Amount::from_str("123.44 USD").test_unwrap(),
            created_at: datetime_from_str("2022-05-31 10:29:30 +02:00"),
            updated_at: datetime_from_str("2024-05-31 22:29:57 +02:00"),
        });

        let _id = &account.id;
        // let mut id: & Id = &account.id;

        let as_id: Result<&AccountId, _> = TryInto::<&AccountId>::try_into(&account.id);
        println!("### as_id: {:?}", as_id);
    }

    #[allow(dead_code)] // actually it is really used
    fn datetime_from_str(str: &str) -> chrono::DateTime<Utc> {
        use core::str::FromStr;
        chrono::DateTime::<FixedOffset>::from_str(str).test_unwrap().to_utc()
    }
}
*/