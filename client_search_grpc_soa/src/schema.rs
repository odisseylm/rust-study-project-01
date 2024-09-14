

diesel::table! {
// diesel_async::table! {
    #[allow(non_snake_case)]
    // !!!??? Strange, diesel uses quotes for table/column names ??!!?? wtf!!!
    // postgres uses lowercase
    // TODO: how to disable using quoted table/column names in diesel SQL?
    #[sql_name = "clients"]
    CLIENTS (client_id) {
        client_id -> Uuid, // Varchar,
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
