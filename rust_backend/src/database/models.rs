use crate::schema::*;
use chrono::NaiveDateTime;

use diesel::Insertable;
use diesel::{sql_types::Nullable, Queryable, Selectable};
use serde::Serialize;
use uuid::Uuid;

#[derive(diesel_derive_enum::DbEnum, Debug, Serialize)]
#[ExistingTypePath = "crate::schema::sql_types::Buildstatus"]
pub enum Buildstatus {
    NotStarted,
    Started,
    Done,
    Failed,
}

#[derive(Insertable, Queryable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub salt: String,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub salt: String,
}

#[derive(Insertable, Queryable)]
#[diesel(table_name = files)]
pub struct NewFile {
    pub id: Uuid,
    pub filename: String,
    pub programming_language: String,
    pub filesize: i32,
    pub lastchanges: NaiveDateTime,
    pub file_content: Option<Vec<u8>>,
    pub owner_uuid: Uuid,
    pub build_status: Buildstatus,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::files)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct File {
    pub id: Uuid,
    pub filename: String,
    pub programming_language: String,
    pub filesize: i32,
    pub lastchanges: NaiveDateTime,
    pub file_content: Option<Vec<u8>>,
    pub owner_uuid: Uuid,
    pub build_status: Buildstatus,
}
