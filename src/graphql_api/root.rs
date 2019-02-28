use juniper::graphql_object;
use juniper::FieldError;
use juniper::FieldResult;
use juniper::RootNode;

use cis_profile::schema::Profile;

use crate::cis::client::CisClient;
use crate::cis::client::GetBy;
use crate::graphql_api::input::InputProfile;

pub struct Query {
    pub cis_client: CisClient,
}

fn field_error(msg: &str, e: impl std::fmt::Display) -> FieldError {
    let error = format!("{}: {}", msg, e);
    FieldError::new(msg, graphql_value!({ "internal_error": error }))
}

fn get_profile(username: Option<String>, cis_client: &CisClient) -> FieldResult<Profile> {
    let username = username.unwrap_or_else(|| String::from("fiji"));
    let profile = cis_client.get_user_by(&username, &GetBy::PrimaryUsername, None)?;
    Ok(profile)
}

graphql_object!(Query: () |&self| {
    field apiVersion() -> &str {
        "1.0"
    }
    field profile(&executor, username: Option<String>) -> FieldResult<Profile> {
        get_profile(username, &self.cis_client)
    }
});

pub struct Mutation {
    pub cis_client: CisClient,
}

fn update_profile(update: InputProfile, cis_client: &CisClient) -> FieldResult<Profile> {
    let user_id = "ad|Mozilla-LDAP|FMerz";
    let mut profile = cis_client.get_user_by(user_id, &GetBy::UserId, None)?;
    update
        .update_profile(&mut profile, &cis_client.get_secret_store())
        .map_err(|e| field_error("unable update/sign profle", e))?;
    let ret = cis_client.update_user(profile)?;
    info!("update returned: {}", ret);
    let updated_profile = cis_client.get_user_by(user_id, &GetBy::UserId, None)?;
    Ok(updated_profile)
}

graphql_object!(Mutation: () |&self| {
    field apiVersion() -> &str {
        "1.0"
    }
    field profile(&executor, update: InputProfile) -> FieldResult<Profile> {
        update_profile(update, &self.cis_client)
    }
});

pub type Schema = RootNode<'static, Query, Mutation>;