use chrono::Utc;
use mvv_common::{
    entity::amount::Amount,
    generate_from_str_new_type_delegate,
    generate_into_inner_delegate,
    generate_pg_delegate_decode,
    generate_pg_delegate_encode,
    generate_pg_delegate_type_info,
};
use crate::entity::ClientId;
// -------------------------------------------------------------------------------------------------



#[derive(Debug, Clone, PartialEq, derive_more::Display)] // derive_more::FromStr)]
#[display("{}", _0)]
pub struct AccountId( #[allow(dead_code)] uuid::Uuid);
// pub type AccountIdFormatError = mvv_common::entity::id::parse::IdFormatError;
pub type AccountIdFormatError = mvv_common::uuid::UuidFormatError;

// impl DataFormatError for AccountIdFormatError { }

generate_into_inner_delegate! { AccountId, uuid::Uuid }
generate_from_str_new_type_delegate! { AccountId, uuid::Uuid, mvv_common::uuid::UuidFormatError }

generate_pg_delegate_type_info! { AccountId, uuid::Uuid }
generate_pg_delegate_encode!    { AccountId, uuid::Uuid }
generate_pg_delegate_decode!    { AccountId, uuid::Uuid }



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



#[cfg(test)]
mod tests {
    use mvv_auth::util::test_unwrap::TestResultUnwrap;
    use crate::entity::IbanWrapper;

    #[test]
    fn account_id_from_str() {
        use core::str::FromStr;
        iban::Iban::from_str("UA85 399622 0000 0002 6001 2335 661").test_unwrap();
        // iban::Iban::from_str("UA35 334851 0000 2600 9001 2345 67").test_unwrap();
        iban::Iban::from_str("UA21 3223 1300 0002 6007 2335 6600 1").test_unwrap();
        iban::Iban::from_str("UA90 3515 3300 0002 6006 0359 0071 2").test_unwrap();
        iban::Iban::from_str("UA90 305299 2990004149123456789").test_unwrap();
        iban::Iban::from_str("UA20 38080500000000026034 4816 3").test_unwrap();
        iban::Iban::from_str("UA90 380805 000000 0026006780269").test_unwrap();

        iban::Iban::from_str("UA713736572172926969841832393").test_unwrap();
        iban::Iban::from_str("UA948614766857337364625464668").test_unwrap();
        iban::Iban::from_str("UA565117374274826517545533479").test_unwrap();
        iban::Iban::from_str("UA496826153843944716538382928").test_unwrap();

        IbanWrapper::from_str("UA713736572172926969841832393").test_unwrap();
        IbanWrapper::from_str("UA948614766857337364625464668").test_unwrap();
        IbanWrapper::from_str("UA565117374274826517545533479").test_unwrap();
        IbanWrapper::from_str("UA496826153843944716538382928").test_unwrap();
    }

}