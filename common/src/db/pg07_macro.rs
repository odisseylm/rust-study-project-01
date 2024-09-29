

#[macro_export] macro_rules! generate_pg07_delegate_type_info {
    ($Type:ty, $DelegateType:ty) => {

        #[allow(unused_imports, unused_qualifications)]
        impl sqlx_07::Type<sqlx_postgres_07::Postgres> for $Type {
            #[inline]
            fn type_info() -> <sqlx_postgres_07::Postgres as sqlx_07::Database>::TypeInfo {
                <$DelegateType as sqlx_07::Type<sqlx_postgres_07::Postgres>>::type_info()
            }
            #[inline]
            fn compatible(ty: &<sqlx_postgres_07::Postgres as sqlx_07::Database>::TypeInfo) -> bool {
                <$DelegateType as sqlx_07::Type<sqlx_postgres_07::Postgres> >::compatible(ty)
            }
        }

    };
}


#[macro_export] macro_rules! generate_pg07_ref_delegate_type_info {
    ($Type:ident, $DelegateType:ty) => { // ($Type:ty, $DelegateType:ty) => {

        #[allow(unused_imports, unused_qualifications)]
        impl<'a> sqlx_07::Type<sqlx_postgres_07::Postgres> for $Type<'a> {
            #[inline]
            fn type_info() -> <sqlx_postgres_07::Postgres as sqlx_07::Database>::TypeInfo {
                <$DelegateType as sqlx_07::Type<sqlx_postgres_07::Postgres>>::type_info()
            }
            #[inline]
            fn compatible(ty: &<sqlx_postgres_07::Postgres as sqlx_07::Database>::TypeInfo) -> bool {
                <$DelegateType as sqlx_07::Type<sqlx_postgres_07::Postgres> >::compatible(ty)
            }
        }

    };
}


#[macro_export] macro_rules! generate_pg07_delegate_encode {
    ($Type:ty, $DelegateType:ty) => {

        #[allow(unused_imports, unused_qualifications)]
        impl<'r> sqlx_07::Encode<'r, sqlx_postgres_07::Postgres> for $Type {
            #[inline]
            fn encode_by_ref(
                &self,
                buf: &mut <sqlx_postgres_07::Postgres as sqlx_07::database::HasArguments<'r>>::ArgumentBuffer,
            ) -> sqlx_07::encode::IsNull {
                <& $DelegateType as sqlx_07::Encode<sqlx_postgres_07::Postgres> >::encode(&self.0, buf)
            }
        }
    }
}


#[macro_export] macro_rules! generate_pg07_ref_delegate_encode {
    ($Type:ident, $DelegateType:ty) => { // ($Type:ty, $DelegateType:ty) => {

        #[allow(unused_imports, unused_qualifications)]
        impl<'r,'a> sqlx_07::Encode<'r, sqlx_postgres_07::Postgres> for $Type<'a> {
            #[inline]
            fn encode_by_ref(
                &self,
                buf: &mut <sqlx_postgres_07::Postgres as sqlx_07::database::HasArguments<'r>>::ArgumentBuffer,
            ) -> sqlx_07::encode::IsNull { // for sqlx 0.7
                <& $DelegateType as sqlx_07::Encode<sqlx_postgres_07::Postgres> >::encode(&self.0, buf)
            }
        }
    }
}


#[macro_export] macro_rules! generate_pg07_delegate_decode {
    ($Type:ident, $DelegateType:ty) => {

        #[allow(unused_imports, unused_qualifications)]
        impl<'r> sqlx_07::Decode<'r, sqlx_postgres_07::Postgres> for $Type {
            #[inline]
            fn decode(
                value: <sqlx_postgres_07::Postgres as sqlx_07::database::HasValueRef<'r>>::ValueRef
            ) -> Result<Self, sqlx_07::error::BoxDynError> {
                let v = < $DelegateType as sqlx_07::Decode<'r, sqlx_postgres_07::Postgres> > ::decode(value) ?;
                Ok( $Type (v)) // T O D O: how to use '$Type:ty' there??
            }
        }

    }
}


#[macro_export] macro_rules! generate_pg07_delegate_decode_from_str {
    ($Type:ident, $DelegateType:ty) => {

        #[allow(unused_imports, unused_qualifications)]
        impl<'r> sqlx_07::Decode<'r, sqlx_postgres_07::Postgres> for $Type {
            #[inline]
            fn decode(
                value: <sqlx_postgres_07::Postgres as sqlx_07::database::HasValueRef<'r>>::ValueRef
            ) -> Result<Self, sqlx_07::error::BoxDynError> {
                let as_str = <String as sqlx_07::Decode<'r, sqlx_postgres_07::Postgres>>::decode(value) ?;
                Ok( $Type (<$DelegateType as core::str::FromStr>::from_str(&as_str) ?))
            }
        }

    };
}

#[macro_export] macro_rules! generate_pg07_decode_from_str {
    ($Type:ty) => {
        generate_pg07_decode_from_str! { $Type, core::str::FromStr, from_str }
    };
    ($Type:ty, $from_db_str_fn:ident) => {

        #[allow(unused_imports, unused_qualifications)]
        impl<'r> sqlx_07::Decode<'r, sqlx_postgres_07::Postgres> for $Type {
            #[inline]
            fn decode(
                value: <sqlx_postgres_07::Postgres as sqlx_07::database::HasValueRef<'r>>::ValueRef
            ) -> Result<Self, sqlx_07::error::BoxDynError> {
                let as_str: &str = <&str as sqlx_07::Decode<'r, sqlx_postgres_07::Postgres>>::decode(value) ?;
                Ok( <$Type>::$from_db_str_fn(as_str) ?)
            }
        }

    };
    ($Type:ty, $FromStrTrait:ty, $from_db_str_fn:ident) => {

        #[allow(unused_imports, unused_qualifications)]
        impl<'r> sqlx_07::Decode<'r, sqlx_postgres_07::Postgres> for $Type {
            #[inline]
            fn decode(
                value: <sqlx_postgres_07::Postgres as sqlx_07::database::HasValueRef<'r>>::ValueRef
            ) -> Result<Self, sqlx_07::error::BoxDynError> {
                let as_str: &str = <&str as sqlx_07::Decode<'r, sqlx_postgres_07::Postgres>>::decode(value) ?;
                Ok( <$Type as $FromStrTrait>::$from_db_str_fn(as_str) ?)
            }
        }

    };
}


#[macro_export] macro_rules! generate_pg07_encode_from_as_str {
    ($Type:ident) => { // ($Type:ty) => {
        generate_pg07_encode_from_as_str! { $Type, as_str }
    };
    ($Type:ident, $as_str_fn:ident) => { // ($Type:ty) => {

        #[allow(unused_imports, unused_qualifications)]
        impl<'q> sqlx_07::Encode<'q, sqlx_postgres_07::Postgres> for $Type {
            fn encode_by_ref(
                &self,
                buf: &mut <sqlx_postgres_07::Postgres as sqlx_07::database::HasArguments<'q>>::ArgumentBuffer,
            ) -> sqlx_07::encode::IsNull {
                let str: &str = self.$as_str_fn();
                <&str as sqlx_07::Encode<sqlx_postgres_07::Postgres>>::encode(str, buf)
            }
        }
    };
}
