use chrono::Utc;
use serde::{ Deserialize, Serialize };
use crate::entities::amount::Amount;
use crate::entities::id::Id;
// use chrono::serde::*;



#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[readonly::make]
pub struct Account {
    pub id: Id,
    pub user_id: Id,
    pub amount: Amount,
    // pub created_at: std::time::Instant,
    // pub updated_at: std::time::Instant,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,

    // // Never serialized.
    // #[serde(skip_serializing)]
    // hash: String,
    //
    // // Use a method to decide whether the field should be skipped.
    // #[serde(skip_serializing_if = "Map::is_empty")]
    // metadata: Map<String, String>,
}


impl Account {
    pub fn new(args: new::Args) -> Self {
        Account {
            id: args.id,
            user_id: args.user_id,
            amount: args.amount,
            created_at: args.created_at,
            updated_at: args.updated_at,
        }
    }
}

pub mod new {
    use chrono::Utc;
    use crate::entities::amount::Amount;
    use crate::entities::id::Id;

    pub struct Args {
        pub id: Id,
        pub user_id: Id,
        pub amount: Amount,
        pub created_at: chrono::DateTime<Utc>,
        pub updated_at: chrono::DateTime<Utc>,
    }
}


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
    use chrono::{FixedOffset, Utc};
    use crate::entities::account::{Account, new};
    use crate::entities::amount::Amount;
    use crate::entities::id::Id;
    use crate::util::TestResultUnwrap;

    fn aa() {
        let account = Account::new(new::Args {
            id: Id::from_str("1").test_unwrap(),
            user_id: Id::from_str("2").test_unwrap(),
            amount: Amount::from_str("123.44 USD").test_unwrap(),
            created_at: datetime_from_str("2022-05-31 10:29:30 +02:00"),
            updated_at: datetime_from_str("2024-05-31 22:29:57 +02:00"),
        });

        let id = &account.id;
        // let mut id: & Id = &account.id;
        // id.0 = "443".to_string();

        let as_id: Result<&Id, _> = TryInto::<&Id>::try_into(&account.id);
        println!("### as_id: {:?}", as_id);
    }

    fn datetime_from_str(str: &str) -> chrono::DateTime<Utc> {
        use std::str::FromStr;
        chrono::DateTime::<FixedOffset>::from_str(str).test_unwrap().to_utc()
    }
}
