

diesel::table! {
// diesel_async::table! {
    #[allow(non_snake_case)]
    CLIENTS (client_id) {
        // client_id -> Uuid,
        client_id -> Varchar,
        email -> Varchar,
        first_name -> Varchar,
        last_name -> Varchar,
        phone -> Varchar,
        birthday -> Date,
        active -> Bool,
        business_user -> Bool,
        super_business_user -> Bool,
    }
}
