



#[macro_export] macro_rules! generate_diesel2_pg_to_sql_from_as_str {
    ($Type:ident) => {
        generate_diesel2_pg_to_sql_from_as_str! { $Type, as_str}
    };
    ($Type:ident, $as_str_fn:ident) => {

        #[allow(unused_imports, unused_qualifications)]
        impl diesel::serialize::ToSql<diesel::sql_types::VarChar, diesel::pg::Pg> for $Type {
            fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>)
                -> diesel::serialize::Result {
                let as_db_str = self.$as_str_fn();
                out.write_all(as_db_str.as_bytes()) ?;
                Ok(diesel::serialize::IsNull::No)
            }
        }

    };
}



#[macro_export] macro_rules! generate_diesel2_pg_from_sql_from_str {
    ($Type:ty) => {
        generate_diesel2_pg_from_sql_from_str_lossy!{ $Type, from_str }
    };
    ($Type:ty, $from_db_str_fn:ident, $table:literal, $column:literal) => {

        #[allow(unused_imports, unused_qualifications)]
        impl diesel::deserialize::FromSql<diesel::sql_types::VarChar, diesel::pg::Pg> for $Type {
            fn from_sql(pg_bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
                let bytes = pg_bytes.as_bytes();
                let str = std::str::from_utf8(bytes)
                        .map_err(|_utf8err|
                            $crate::db::DbMappingError::incorrect_utf_8_db_value(
                                bytes, $table.into(), $column.into())
                        ) ?;
                let str = str.as_ref();
                let value = <$Type>::$from_db_str_fn(str) ?;
                Ok(value)
            }
        }

    };
}



#[macro_export] macro_rules! generate_diesel2_pg_from_sql_from_str_lossy {
    ($Type:ty) => {
        generate_diesel2_pg_from_sql_from_str_lossy!{ $Type, from_str }
    };
    ($Type:ty, $from_db_str_fn:ident) => {

        #[allow(unused_imports, unused_qualifications)]
        impl diesel::deserialize::FromSql<diesel::sql_types::VarChar, diesel::pg::Pg> for $Type {
            fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
                let str = String::from_utf8_lossy(bytes.as_bytes());
                let str = str.as_ref();
                let value = <$Type>::$from_db_str_fn(str) ?;
                Ok(value)
            }
        }

    };
}
