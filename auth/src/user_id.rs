use core::fmt;
//--------------------------------------------------------------------------------------------------


/// Just wrapper to avoid unneeded cloning and converting to string (for moving data).
///
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum UserId {
    u32(u32),
    i32(i32),
    u64(u64),
    i64(i64),
    u128(u128),
    i128(i128),
    String(String),
    StaticStr(&'static str),
    Uuid(uuid::Uuid),
    //
    // Feel free to add new required subtypes.
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserId::u32(ref v)  => v.fmt(f),
            UserId::i32(ref v)  => v.fmt(f),
            UserId::u64(ref v)  => v.fmt(f),
            UserId::i64(ref v)  => v.fmt(f),
            UserId::u128(ref v) => v.fmt(f),
            UserId::i128(ref v) => v.fmt(f),
            UserId::String(ref v) => v.fmt(f),
            UserId::StaticStr(ref v) => v.fmt(f),
            UserId::Uuid(ref v) => v.fmt(f),
        }
    }
}

impl From<u32>  for UserId {  fn from(v: u32)  -> Self { UserId::u32(v)  }  }
impl From<i32>  for UserId {  fn from(v: i32)  -> Self { UserId::i32(v)  }  }
impl From<u64>  for UserId {  fn from(v: u64)  -> Self { UserId::u64(v)  }  }
impl From<i64>  for UserId {  fn from(v: i64)  -> Self { UserId::i64(v)  }  }
impl From<u128> for UserId {  fn from(v: u128) -> Self { UserId::u128(v) }  }
impl From<i128> for UserId {  fn from(v: i128) -> Self { UserId::i128(v) }  }
impl From<String>       for UserId {  fn from(v: String)       -> Self { UserId::String(v)    }  }
impl From<&'static str> for UserId {  fn from(v: &'static str) -> Self { UserId::StaticStr(v) }  }
impl From<uuid::Uuid>   for UserId {  fn from(v: uuid::Uuid)   -> Self { UserId::Uuid(v)      }  }
