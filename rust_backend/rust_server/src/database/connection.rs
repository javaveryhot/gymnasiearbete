use crate::database::models::{InsertedFile, NewFile, NewSessionToken, NewUser, User};

use crate::Error;
use crate::Result;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use dotenv::dotenv;
use uuid::Uuid;

#[allow(clippy::module_name_repetitions)]
pub async fn establish_connection() -> Result<AsyncPgConnection> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    AsyncPgConnection::establish(&database_url)
        .await
        .map_err(|err| Error::DatabaseConnectionFail)
}

pub async fn create_user(new_user: NewUser) -> Result<Uuid> {
    let mut conn = establish_connection().await?;
    diesel::insert_into(crate::schema::users::table)
        .values(&new_user)
        .execute(&mut conn)
        .await
        .map_err(|err| Error::DatabaseConnectionFail)?;
    Ok(new_user.id)
}

pub async fn upload_file(file: NewFile) -> Result<Uuid> {
    use crate::schema::files::dsl::files;
    let mut conn = establish_connection().await?;

    info!("{:?}", file);

    match diesel::insert_into(files)
        .values(file)
        .get_result::<InsertedFile>(&mut conn)
        .await
    {
        Ok(file_id) => {
            info!("{}", file_id.id);
            Ok(file_id.id)
        }
        Err(err) => {
            error!("Error inserting file: {}", err);
            Err(Error::DatabaseConnectionFail)
        }
    }
}

pub async fn get_build_status(file_id: Uuid) -> Result<crate::database::models::Buildstatus> {
    use crate::schema::files::dsl::*;

    let mut conn = establish_connection().await?;

    let result = files
        .filter(id.eq(file_id))
        .select(build_status)
        .first(&mut conn)
        .await
        .map_err(|err| Error::DatabaseConnectionFail);

    result
}

pub async fn update_build_status(
    file_id: Uuid,
    new_status: crate::database::models::Buildstatus,
) -> Result<crate::database::models::Buildstatus> {
    use crate::schema::files::dsl::*;

    let mut conn = establish_connection().await?;

    diesel::update(files.filter(id.eq(file_id)))
        .set(build_status.eq(new_status))
        .execute(&mut conn)
        .await
        .map_err(|err| Error::DatabaseQueryFail)?;

    return files
        .filter(id.eq(file_id))
        .select(build_status)
        .first::<crate::database::models::Buildstatus>(&mut conn)
        .await
        .map_err(|err| Error::DatabaseQueryFail);
}

pub async fn username_exists(target_username: &str) -> Result<bool> {
    use crate::schema::users::dsl::*;
    let mut conn = establish_connection().await?;
    let result = users
        .filter(username.eq(target_username))
        .first::<User>(&mut conn)
        .await
        .map_err(|err| Error::DatabaseConnectionFail);
    match result {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub async fn get_user_from_username(_username: &str) -> Result<User> {
    let mut conn = establish_connection().await?;
    use crate::schema::users::dsl::*;
    let result = users
        .filter(username.eq(_username))
        .first::<User>(&mut conn)
        .await
        .map_err(|_| Error::DatabaseConnectionFail);
    result
}
#[derive(Clone)]
pub struct UploadToken {
    pub user_uuid: Uuid,
    pub token: String,
    pub expiration_date: NaiveDateTime,
}

pub async fn upload_session_token(up_token: UploadToken) -> Result<()> {
    let mut conn = establish_connection().await?;
    use crate::schema::session_tokens::dsl::*;

    let new_token = NewSessionToken {
        token: &up_token.token,
        user_uuid: up_token.user_uuid,
        expiration_date: up_token.expiration_date,
    };

    diesel::insert_into(session_tokens)
        .values(new_token)
        .execute(&mut conn)
        .await
        .map_err(|err| Error::DatabaseQueryFail)?;

    Ok(())
}

pub async fn get_user(user_id: Uuid) -> Result<User> {
    let mut conn = establish_connection().await?;
    use crate::schema::users::dsl::*;
    let result = users
        .filter(id.eq(user_id))
        .first::<User>(&mut conn)
        .await
        .map_err(|err| Error::DatabaseQueryFail);
    result
}

// Get the user from the token, return a Result containing a Some(User) if the token is valid, None otherwise.
pub async fn get_token_owner(token_str: &String) -> Result<Option<User>> {
    let mut conn = establish_connection().await?;
    use crate::schema::session_tokens::dsl::*;
    let result: Uuid = session_tokens
        .filter(token.eq(token_str))
        .select(user_uuid)
        .first(&mut conn)
        .await
        .map_err(|err| Error::DatabaseQueryFail)?;

    let user = get_user(result).await?;
    if user.id == Uuid::nil() {
        return Ok(None);
    }
    return Ok(Some(user));
}

pub async fn get_files_from_user(user_id: Uuid) -> Result<Vec<Uuid>> {
    let mut conn = establish_connection().await?;
    use crate::schema::files::dsl::*;

    let file_ids = files
        .filter(owner_uuid.eq(user_id))
        .select(id)
        .load::<Uuid>(&mut conn)
        .await
        .map_err(|err| Error::DatabaseQueryFail);

    file_ids
}
