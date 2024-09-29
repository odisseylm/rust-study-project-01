


#[macro_export] macro_rules! generate_pg08_delegate_type_info {
    ($Type:ty, $DelegateType:ty) => {

        #[allow(unused_imports, unused_qualifications)]
        impl sqlx_08::Type<sqlx_postgres_08::Postgres> for $Type {
            #[inline]
            fn type_info() -> <sqlx_postgres_08::Postgres as sqlx_08::Database>::TypeInfo {
                <$DelegateType as sqlx_08::Type<sqlx_postgres_08::Postgres>>::type_info()
            }
            #[inline]
            fn compatible(ty: &<sqlx_postgres_08::Postgres as sqlx_08::Database>::TypeInfo) -> bool {
                <$DelegateType as sqlx_08::Type<sqlx_postgres_08::Postgres> >::compatible(ty)
            }
        }

    };
}


#[macro_export] macro_rules! generate_pg08_ref_delegate_type_info {
    ($Type:ident, $DelegateType:ty) => { // ($Type:ty, $DelegateType:ty) => {

        #[allow(unused_imports, unused_qualifications)]
        impl<'a> sqlx_08::Type<sqlx_postgres_08::Postgres> for $Type<'a> {
            #[inline]
            fn type_info() -> <sqlx_postgres_08::Postgres as sqlx_08::Database>::TypeInfo {
                <$DelegateType as sqlx_08::Type<sqlx_postgres_08::Postgres>>::type_info()
            }
            #[inline]
            fn compatible(ty: &<sqlx_postgres_08::Postgres as sqlx_08::Database>::TypeInfo) -> bool {
                <$DelegateType as sqlx_08::Type<sqlx_postgres_08::Postgres> >::compatible(ty)
            }
        }

    };
}


#[macro_export] macro_rules! generate_pg08_delegate_encode {
    ($Type:ty, $DelegateType:ty) => {

        #[allow(unused_imports, unused_qualifications)]
        impl<'r> sqlx_08::Encode<'r, sqlx_postgres_08::Postgres> for $Type {
            #[inline]
            fn encode_by_ref(
                &self,
                buf: &mut <sqlx_postgres_08::Postgres as sqlx_08::Database>::ArgumentBuffer<'r>,
            ) -> Result<sqlx_08::encode::IsNull, sqlx_08::error::BoxDynError> {
                <& $DelegateType as sqlx_08::Encode<sqlx_postgres_08::Postgres> >::encode(&self.0, buf)
            }
        }
    }
}


#[macro_export] macro_rules! generate_pg08_ref_delegate_encode {
    ($Type:ident, $DelegateType:ty) => { // ($Type:ty, $DelegateType:ty) => {

        #[allow(unused_imports, unused_qualifications)]
        impl<'r,'a> sqlx_08::Encode<'r, sqlx_postgres_08::Postgres> for $Type<'a> {
            #[inline]
            fn encode_by_ref(
                &self,
                buf: &mut <sqlx_postgres_08::Postgres as sqlx_08::Database>::ArgumentBuffer<'r>,
            ) -> Result<sqlx_08::encode::IsNull, sqlx_08::error::BoxDynError> {
                <& $DelegateType as sqlx_08::Encode<sqlx_postgres_08::Postgres> >::encode(&self.0, buf)
            }
        }
    }
}


#[macro_export] macro_rules! generate_pg08_delegate_decode {
    ($Type:ident, $DelegateType:ty) => {

        #[allow(unused_imports, unused_qualifications)]
        impl<'r> sqlx_08::Decode<'r, sqlx_postgres_08::Postgres> for $Type {
            #[inline]
            fn decode(
                value: <sqlx_postgres_08::Postgres as sqlx_08::Database>::ValueRef<'r>
            ) -> Result<Self, sqlx_08::error::BoxDynError> {
                let v = < $DelegateType as sqlx_08::Decode<'r, sqlx_postgres_08::Postgres> > ::decode(value) ?;
                Ok( $Type (v)) // T O D O: how to use '$Type:ty' there??
            }
        }

    }
}


#[macro_export] macro_rules! generate_pg08_delegate_decode_from_str {
    ($Type:ident, $DelegateType:ty) => {

        #[allow(unused_imports, unused_qualifications)]
        impl<'r> sqlx_08::Decode<'r, sqlx_postgres_08::Postgres> for $Type {
            #[inline]
            fn decode(
                value: <sqlx_postgres_08::Postgres as sqlx_08::Database>::ValueRef<'r>
            ) -> Result<Self, sqlx_08::error::BoxDynError> {
                let as_str = <String as sqlx_08::Decode<'r, sqlx_postgres_08::Postgres>>::decode(value) ?;
                Ok( $Type (<$DelegateType as core::str::FromStr>::from_str(&as_str) ?))
            }
        }

    };
}

#[macro_export] macro_rules! generate_pg08_decode_from_str {
    ($Type:ty) => {
        generate_pg08_decode_from_str! { $Type, core::str::FromStr, from_str }
    };
    ($Type:ty, $from_db_str_fn:ident) => {

        #[allow(unused_imports, unused_qualifications)]
        impl<'r> sqlx_08::Decode<'r, sqlx_postgres_08::Postgres> for $Type {
            #[inline]
            fn decode(
                value: <sqlx_postgres_08::Postgres as sqlx_08::Database>::ValueRef<'r>
            ) -> Result<Self, sqlx_08::error::BoxDynError> {
                let as_str: &str = <&str as sqlx_08::Decode<'r, sqlx_postgres_08::Postgres>>::decode(value) ?;
                Ok( <$Type>::$from_db_str_fn(as_str) ?)
            }
        }

    };
    ($Type:ty, $FromStrTrait:ty, $from_db_str_fn:ident) => {

        #[allow(unused_imports, unused_qualifications)]
        impl<'r> sqlx_08::Decode<'r, sqlx_postgres_08::Postgres> for $Type {
            #[inline]
            fn decode(
                value: <sqlx_postgres_08::Postgres as sqlx_08::Database>::ValueRef<'r>
            ) -> Result<Self, sqlx_08::error::BoxDynError> {
                let as_str: &str = <&str as sqlx_08::Decode<'r, sqlx_postgres_08::Postgres>>::decode(value) ?;
                Ok( <$Type as $FromStrTrait>::$from_db_str_fn(as_str) ?)
            }
        }

    };
}


#[macro_export] macro_rules! generate_pg08_encode_from_as_str {
    ($Type:ident) => { // ($Type:ty) => {
        generate_pg08_encode_from_as_str! { $Type, as_str }
    };
    ($Type:ident, $as_str_fn:ident) => { // ($Type:ty) => {

        #[allow(unused_imports, unused_qualifications)]
        impl<'q> sqlx_08::Encode<'q, sqlx_postgres_08::Postgres> for $Type {
            fn encode_by_ref(&self, buf: &mut <sqlx_postgres_08::Postgres as sqlx_08::Database>::ArgumentBuffer<'q>) -> Result<sqlx_08::encode::IsNull, sqlx_08::error::BoxDynError> {
                let str: &str = self.$as_str_fn();
                <&str as sqlx_08::Encode<sqlx_postgres_08::Postgres>>::encode(str, buf)
            }
        }
    };
}
