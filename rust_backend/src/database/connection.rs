use crate::models::NewUser;
use crate::schema::files;
use crate::Pool;
use chrono::NaiveDateTime;
use diesel::{prelude::*, r2d2::ConnectionManager};
use dotenv::dotenv;
use uuid::Uuid;

pub async fn connect_to_db() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let config = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .test_on_check_out(true)
        .build(config)
        .expect("Could not build connection pool")
}

pub async fn get_connection(
    pool: &Pool<ConnectionManager<PgConnection>>,
) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, r2d2::Error> {
    let conn = pool.get();
    return conn;
}

#[derive(Insertable, Queryable)]
#[table_name = "files"]
pub struct InsertedFile {
    pub id: uuid::Uuid,
    pub filename: String,
    pub programming_language: String,
    pub filesize: i32,
    pub lastchanges: NaiveDateTime,
    pub file_content: Option<Vec<u8>>,
    pub owner_uuid: Uuid,
    pub build_status: crate::models::Buildstatus,
}

pub async fn create_user(conn: &mut PgConnection) -> Result<Uuid, diesel::result::Error> {
    let new_id = uuid::Uuid::new_v4();
    let new_user = NewUser {
        id: new_id, /*, other fields...*/
    };
    diesel::insert_into(crate::schema::users::table)
        .values(&new_user)
        .execute(conn)?;
    Ok(new_id)
}

pub async fn upload_file(
    conn: &mut PgConnection,
    filename: &str,
    file_path: &str,
    language: &String,
) -> Result<Uuid, Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(file_path).map_err(|_err| diesel::result::Error::NotFound)?;
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content)
        .map_err(|_err| diesel::result::Error::NotFound)?;
    let user_uuid = create_user(conn).await?;

    let file_size = file_content.len() as i32;

    let new_file = InsertedFile {
        id: uuid::Uuid::new_v4(),
        filename: filename.to_string(),
        programming_language: language.to_string(),
        filesize: file_size,
        lastchanges: NaiveDateTime::default(),
        file_content: Some(file_content),
        owner_uuid: user_uuid,
        build_status: crate::models::Buildstatus::NotStarted,
    };

    let file_id = diesel::insert_into(files::table)
        .values(new_file)
        .get_result::<InsertedFile>(conn)?;
    info!("{}", file_id.id);

    Ok(file_id.id)
}

pub async fn get_build_status(
    conn: &mut PgConnection,
    file_id: Uuid,
) -> Result<crate::models::Buildstatus, diesel::result::Error> {
    use crate::schema::files::dsl::*;

    let result = files
        .filter(id.eq(file_id))
        .select(build_status)
        .first(conn);

    result
}

pub fn update_build_status(
    conn: &mut PgConnection,
    file_id: Uuid,
    new_status: crate::models::Buildstatus,
) -> Result<crate::models::Buildstatus, diesel::result::Error> {
    use crate::schema::files::dsl::*;
    diesel::update(files.filter(id.eq(file_id)))
        .set(build_status.eq(new_status))
        .execute(conn)?;

    let updated_status = files
        .filter(id.eq(file_id))
        .select(build_status)
        .first::<crate::models::Buildstatus>(conn);

    updated_status
}
