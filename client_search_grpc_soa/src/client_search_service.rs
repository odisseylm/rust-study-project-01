use std::collections::HashMap;
use std::sync::Arc;
use anyhow::anyhow;
use chrono::Datelike;
use diesel::{Connection, PgConnection};
use implicit_clone::ImplicitClone;
use tonic::{Request, Response, Status};
use mvv_auth::permission::PermissionSet;
use mvv_common::grpc::TonicErrToStatusExt;
use mvv_common::string::StaticRefOrString;
use crate::auth::{Role, RolePermissionsSet};
use crate::client::ClientInfo;
use crate::dependencies::{Dependencies};
use crate::grpc::mvv::client::search::api::v1::{
    {Client, ClientSearchRequest, ClientSearchResponse, GetClientByIdRequest, GetClientByIdResponse},
    client_search_service_server::ClientSearchService as ClientSearchServiceTrait,
};
//--------------------------------------------------------------------------------------------------



#[allow(dead_code)]
pub fn establish_connection() -> anyhow::Result<PgConnection> {
    // dotenv().ok();

    let postgres_host = std::env::var("POSTGRES_HOST") ?;
    let postgres_db = std::env::var("POSTGRES_DB") ?;
    let postgres_user = std::env::var("POSTGRES_USER") ?;
    let postgres_password = std::env::var("POSTGRES_PASSWORD") ?;

    let database_url = format!("postgres://{postgres_user}:{postgres_password}@{postgres_host}/{postgres_db}");

    let con = PgConnection::establish(&database_url) ?;
        // .expect(&format!("Error connecting to {}", database_url))
    Ok(con)
}


pub struct ClientSearchService {
    pub dependencies: Arc<Dependencies>,
}

impl ClientSearchService {
    pub fn endpoints_roles() -> HashMap<StaticRefOrString, RolePermissionsSet> {
        let read_permissions = RolePermissionsSet::from_permission(Role::Read);
        let read_write_permissions = RolePermissionsSet::from_permissions([Role::Read, Role::Write]);
        HashMap::from([
            ("/mvv.client.search.api.v1.ClientSearchService/Search".into(), read_permissions.implicit_clone()),
            ("/mvv.client.search.api.v1.ClientSearchService/GetClientById".into(), read_permissions.implicit_clone()),
            ("/mvv.client.search.api.v1.ClientSearchService/UpdateClient".into(), read_write_permissions.implicit_clone()),
        ])
    }
}

/*
("/mvv.client.search.api.v1.ClientSearchService/Search".into(), read_permissions.implicit_clone()),
("/mvv.client.search.api.v1.ClientSearchService/GetClientById".into(), read_permissions.implicit_clone()),
("/mvv.client.search.api.v1.ClientSearchService/UpdateClient".into(), read_write_permissions.implicit_clone()),
*/


impl ClientSearchService {
    async fn do_search(&self, request: Request<ClientSearchRequest>) -> anyhow::Result<Vec<Client>> {

        // let mut con = establish_connection() ?;
        let mut con = self.dependencies.diesel_db_pool.get() ?;

        let request = request.get_ref();

        use diesel::prelude::*;
        // use diesel_async::;
        // use crate::schema::*;
        use crate::schema::CLIENTS::dsl::*;

        let mut query = CLIENTS
            .select(ClientInfo::as_select())
            .into_boxed();

        if let Some(ref email_value) = request.user_email {
            query = query.filter(email.eq(email_value.to_lowercase()))
        };

        if let Some(ref first_name_value) = request.first_name {
            // let first_name_value_lc = first_name_value.to_lowercase();
            query = query.filter(first_name.ilike(first_name_value.as_str()))
        };

        if let Some(ref last_name_value) = request.last_name {
            query = query.filter(last_name.ilike(last_name_value.as_str()))
        };

        if let Some(age) = request.age {
            let now: chrono::NaiveDate = chrono::Local::now().naive_local().date();
            let birthday_from: chrono::NaiveDate =
                if now.month() == 2 && now.day() == 29 {
                    let birthday_from = now.with_year(now.year() - age);
                    match birthday_from {
                        None =>
                            // second attempt, see NaiveDate doc
                            now.with_day(28)
                                .ok_or_else(|| anyhow!("Internal error of processing 'age' (setting day).")) ?
                                .with_year(now.year() - age)
                                .ok_or_else(|| anyhow!("Internal error of processing 'age' (setting year).")) ?,
                        Some(birthday_from) => birthday_from,
                    }
                } else {
                    now.with_year(now.year() - age)
                        .ok_or_else(||anyhow!("Internal error of processing 'age' (setting year).")) ?
                };

            query = query.filter(birthday.ge(birthday_from));
        }

        let results: Vec<ClientInfo> = query
            .limit(5)
            .load(&mut con) ?;

        let clients: Vec<Client> = results.into_iter()
            .map(|client|client.into())
            .collect::<Vec<Client>>();

        Ok(clients)
    }

    async fn do_get_client_by_id(&self, client_id_value: &str) -> anyhow::Result<Option<Client>> {

        let mut con = self.dependencies.diesel_db_pool.get() ?;

        use core::str::FromStr;
        use diesel::prelude::*;
        use crate::schema::CLIENTS::dsl::*;

        let client_id_uuid = uuid::Uuid::from_str(client_id_value) ?;

        let client = CLIENTS
            .select(ClientInfo::as_select())
            .filter(client_id.eq(client_id_uuid))
            .first(&mut con)
            .optional()
            .map(|client_opt|client_opt.map(|client|{ let cl: Client = client.into(); cl }))
            ?;

        Ok(client)
    }
}


#[tonic::async_trait]
impl ClientSearchServiceTrait for ClientSearchService {

    async fn search(&self, request: Request<ClientSearchRequest>) -> Result<Response<ClientSearchResponse>, Status> {
        self.do_search(request).await
            .map(|clients| Response::new(
                ClientSearchResponse { success: true, message: None, clients }))
            .to_tonic_internal_err("Client search error")
    }

    async fn get_client_by_id(&self, request: Request<GetClientByIdRequest>) -> Result<Response<GetClientByIdResponse>, Status> {
        self.do_get_client_by_id(request.get_ref().client_id.as_str()).await
            .map(|client| Response::new(GetClientByIdResponse { client }))
            .to_tonic_internal_err("get_client_by_id error")
    }

    async fn update_client(&self, _request: Request<ClientSearchRequest>) -> Result<Response<ClientSearchResponse>, Status> {
        todo!()
    }
}