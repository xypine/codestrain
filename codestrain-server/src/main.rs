use std::collections::{HashMap, HashSet};

//use extism::*;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::response::status::Created;
use rocket::serde::Serialize;
use rocket::serde::{json::Json, Deserialize};
use rocket::{routes, Config};
use rocket_authorization::oauth::OAuth;
use rocket_authorization::Credential;
use rocket_db_pools::{sqlx, Connection, Database};

use futures::stream::TryStreamExt;
use thiserror::Error;
use time::PrimitiveDateTime;
use uuid::Uuid;

#[derive(Database)]
#[database("sqlx")]
struct Db(sqlx::Pool<sqlx::Postgres>);

type CoolerResult<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct User {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    id: Option<Uuid>,
    name: String,
    #[serde(skip_serializing)]
    password: String,
    email: String,
    #[serde(skip_deserializing)]
    created_at: Option<PrimitiveDateTime>,
    #[serde(skip_deserializing)]
    updated_at: Option<PrimitiveDateTime>,
}
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum NewUserError {
    #[error("Invalid email address")]
    InvalidEmail,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Invalid name")]
    InvalidName,
    #[error("User already exists")]
    #[from(sqlx::Error)]
    UserExists,
    #[error("Internal error")]
    InternalError,
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for NewUserError {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        // log `self` to your favored error tracker, e.g.
        // sentry::capture_error(&self);
        println!("{:?}", self);

        match self {
            Self::InvalidEmail => Status::BadRequest,
            Self::InvalidPassword => Status::BadRequest,
            Self::InvalidName => Status::BadRequest,
            Self::UserExists => Status::Conflict,
            // in our simplistic example, we're happy to respond with the default 500 responder in all cases
            _ => Status::InternalServerError,
        }
        .respond_to(req)
    }
}

#[rocket::post("/user", data = "<new_user>")]
async fn new_user(
    mut db: Connection<Db>,
    mut new_user: Json<User>,
) -> Result<Created<Json<User>>, NewUserError> {
    let r = sqlx::query!(
        r#"
        INSERT INTO users (name, password, email)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        new_user.name,
        new_user.password,
        new_user.email
    )
    .fetch_optional(&mut **db)
    .await;

    match r {
        Ok(Some(row)) => {
            // Set the new user's id from the returned value and return as before
            new_user.id = Some(row.id);
            Ok(Created::new("/user").body(new_user))
        }
        Err(sqlx::Error::Database(e)) => {
            if e.constraint().is_some() {
                // The constraint was violated, so we know the user already exists
                return Err(NewUserError::UserExists);
            }
            Err(NewUserError::InternalError)
        }
        _ => Err(NewUserError::InternalError),
    }
}

#[rocket::get("/user")]
async fn users(mut db: Connection<Db>) -> CoolerResult<Json<Vec<User>>> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, name, email, password, created_at, updated_at
        FROM users
        ORDER BY id ASC
        "#
    )
    .fetch(&mut **db)
    .try_collect::<Vec<_>>()
    .await?;

    Ok(Json(users))
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct Session {
    id: Uuid,
    user_id: Uuid,
    token: String,
    created_at: PrimitiveDateTime,
    expires_at: PrimitiveDateTime,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct PublicSession {
    token: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct Login {
    email: String,
    password: String,
}
#[derive(Error, Debug)]
pub enum LoginError {
    #[error("Invalid email address")]
    InvalidCredentials,
    #[error("Internal error")]
    DbErrorCredentialsCheck(sqlx::Error),
    #[error("Internal error")]
    DbErrorSessionCleanup(sqlx::Error),
    #[error("Internal error")]
    DbErrorNewSession(sqlx::Error),
    #[error("Internal error")]
    DbError(#[from] sqlx::Error),
}
impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for LoginError {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        println!("{:?}", self);

        match self {
            Self::InvalidCredentials => Status::Unauthorized,
            _ => Status::InternalServerError,
        }
        .respond_to(req)
    }
}

#[rocket::post("/login", data = "<credentials>")]
async fn login(
    mut db: Connection<Db>,
    credentials: Json<Login>,
) -> Result<Json<Session>, LoginError> {
    // Validate user credentials
    let r = sqlx::query!(
        r#"
        SELECT id, name, email, password, created_at, updated_at
        FROM users
        WHERE email = $1 AND password = $2
        "#,
        credentials.email,
        credentials.password
    )
    .fetch_optional(&mut **db)
    .await
    .map_err(LoginError::DbError)?;

    match r {
        Some(user) => {
            // delete old sessions
            sqlx::query!(
                r#"
                DELETE FROM sessions
                WHERE user_id = $1 AND expires_at < NOW()
                "#,
                user.id
            )
            .execute(&mut **db)
            .await
            .map_err(LoginError::DbErrorSessionCleanup)?;
            // generate token
            let session_token = uuid::Uuid::new_v4().to_string();
            // create session in db
            let row = sqlx::query!(
                r#"
                INSERT INTO sessions (user_id, token)
                VALUES ($1, $2) RETURNING id, created_at, expires_at
                "#,
                user.id,
                session_token
            )
            .fetch_one(&mut **db)
            .await
            .map_err(LoginError::DbErrorNewSession)?;

            // return session
            let session = Session {
                id: row.id,
                user_id: user.id,
                token: session_token,
                created_at: row.created_at,
                expires_at: row.expires_at,
            };
            Ok(Json(session))
        }
        None => Err(LoginError::InvalidCredentials),
    }
}

#[rocket::post("/logout", data = "<credentials>")]
async fn logout(mut db: Connection<Db>, credentials: Json<PublicSession>) -> CoolerResult<()> {
    // Validate user credentials
    sqlx::query!(
        r#"
        DELETE
        FROM sessions
        WHERE token = $1
        "#,
        credentials.token
    )
    .execute(&mut **db)
    .await?;
    Ok(())
}

#[rocket::get("/me")]
async fn validate_session(
    mut db: Connection<Db>,
    auth: Credential<OAuth>,
) -> Result<Json<User>, LoginError> {
    let user = get_user_from_token(&mut db, &auth.token)
        .await
        .ok_or(LoginError::InvalidCredentials)?;

    Ok(Json(user))
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct Strain {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    id: Option<Uuid>,
    name: String,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    creator_id: Option<Uuid>,
}

#[rocket::get("/strain?<creator_id>")]
async fn strains(
    mut db: Connection<Db>,
    creator_id: Option<String>,
) -> CoolerResult<Json<Vec<Strain>>> {
    let creator_id = creator_id.map(|s| Uuid::parse_str(&s).ok()).flatten();
    println!("Requested strains for creator {:?}", creator_id);
    let strains = sqlx::query_as!(
        Strain,
        r#"
        SELECT id, name, creator_id
        FROM strains
        WHERE creator_id = $1 OR $1 IS NULL
        ORDER BY updated_at DESC
        "#,
        creator_id
    )
    .fetch(&mut **db)
    .try_collect::<Vec<_>>()
    .await?;

    Ok(Json(strains))
}

async fn get_user_from_token(db: &mut Connection<Db>, token: &str) -> Option<User> {
    let r = sqlx::query_as!(
        User,
        r#"
        SELECT users.id, users.name, users.email, users.password, users.created_at, users.updated_at
        FROM users
        INNER JOIN sessions ON sessions.user_id = users.id
        WHERE sessions.token = $1 AND sessions.expires_at > NOW()
        "#,
        token
    )
    .fetch_optional(&mut ***db)
    .await
    .ok()?;

    r
}

#[rocket::post("/strain", data = "<new_strain>")]
async fn new_strain(
    mut db: Connection<Db>,
    mut new_strain: Json<Strain>,
    auth: Credential<OAuth>,
) -> Result<Created<Json<Strain>>, NewUserError> {
    let user = get_user_from_token(&mut db, &auth.token)
        .await
        .ok_or(NewUserError::InternalError)?;
    new_strain.creator_id = Some(user.id.unwrap());
    let r = sqlx::query!(
        r#"
        INSERT INTO strains (name, creator_id)
        VALUES ($1, $2)
        RETURNING id
        "#,
        new_strain.name,
        new_strain.creator_id,
    )
    .fetch_optional(&mut **db)
    .await;

    match r {
        Ok(Some(row)) => {
            // Set the new user's id from the returned value and return as before
            new_strain.id = Some(row.id);
            Ok(Created::new("/strain").body(new_strain))
        }
        Err(sqlx::Error::Database(e)) => {
            println!("{:?}", e);
            if e.constraint().is_some() {
                // The constraint was violated, so we know the user already exists
                return Err(NewUserError::UserExists);
            }
            Err(NewUserError::InternalError)
        }
        _ => Err(NewUserError::InternalError),
    }
}

use serde_with::base64::Base64;
use serde_with::serde_as;
#[serde_as]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct StrainVersion {
    code: String,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    id: Option<Uuid>,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    strain_id: Option<Uuid>,
    #[serde_as(as = "Option<Base64>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    wasm: Option<Vec<u8>>,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    created_at: Option<PrimitiveDateTime>,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    updated_at: Option<PrimitiveDateTime>,
}

#[rocket::post("/strain/<strain_id_str>/version", data = "<new_strain_version>")]
async fn new_strain_version(
    mut db: Connection<Db>,
    mut new_strain_version: Json<StrainVersion>,
    strain_id_str: String,
    auth: Credential<OAuth>,
) -> Result<Created<Json<StrainVersion>>, LoginError> {
    let strain_id = Uuid::parse_str(&strain_id_str).map_err(|_| LoginError::InvalidCredentials)?;
    let user = get_user_from_token(&mut db, &auth.token)
        .await
        .ok_or(LoginError::InvalidCredentials)?;
    let strain = sqlx::query!(
        r#"
        SELECT creator_id
        FROM strains
        WHERE id = $1
        "#,
        strain_id
    )
    .fetch_one(&mut **db)
    .await?;
    if user.id.unwrap() != strain.creator_id {
        return Err(LoginError::InvalidCredentials);
    }
    new_strain_version.strain_id = Some(strain_id);
    let r = sqlx::query!(
        r#"
        INSERT INTO strain_versions (strain_id, code, wasm)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        new_strain_version.strain_id,
        new_strain_version.code,
        new_strain_version.wasm,
    )
    .fetch_one(&mut **db)
    .await?;

    new_strain_version.id = Some(r.id);
    Ok(Created::new("/strain_version").body(new_strain_version))
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct StrainVersionMeta {
    id: Uuid,
    strain_id: Uuid,
    created_at: PrimitiveDateTime,
    updated_at: PrimitiveDateTime,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct StrainWithVersionMeta {
    id: Uuid,
    name: String,
    creator_id: Uuid,
    versions: Vec<StrainVersionMeta>,
    created_at: PrimitiveDateTime,
    updated_at: PrimitiveDateTime,
}

#[rocket::get("/strain/<strain_id_str>")]
async fn strain(
    mut db: Connection<Db>,
    strain_id_str: String,
) -> CoolerResult<Json<StrainWithVersionMeta>> {
    let strain_id = Uuid::parse_str(&strain_id_str.to_string()).expect("Invalid strain id");
    let strain = sqlx::query!(
        r#"
        SELECT id, name, creator_id, created_at, updated_at
        FROM strains
        WHERE id = $1
        "#,
        strain_id
    )
    .fetch_one(&mut **db)
    .await?;
    let strain_versions = sqlx::query_as!(
        StrainVersionMeta,
        r#"
        SELECT id, strain_id, created_at, updated_at
        FROM strain_versions
        WHERE strain_id = $1
        ORDER BY created_at DESC
        "#,
        strain_id
    )
    .fetch(&mut **db)
    .try_collect::<Vec<_>>()
    .await?;

    Ok(Json(StrainWithVersionMeta {
        id: strain.id,
        name: strain.name,
        creator_id: strain.creator_id,
        versions: strain_versions,
        created_at: strain.created_at,
        updated_at: strain.updated_at,
    }))
}

#[rocket::delete("/strain/<strain_id_str>")]
async fn delete_strain(
    mut db: Connection<Db>,
    strain_id_str: String,
    auth: Credential<OAuth>,
) -> Result<(), LoginError> {
    let strain_id =
        Uuid::parse_str(&strain_id_str.to_string()).map_err(|_| LoginError::InvalidCredentials)?;
    let user = get_user_from_token(&mut db, &auth.token)
        .await
        .ok_or(LoginError::InvalidCredentials)?;
    let strain = sqlx::query!(
        r#"
        SELECT creator_id
        FROM strains
        WHERE id = $1
        "#,
        strain_id
    )
    .fetch_one(&mut **db)
    .await?;
    if user.id.unwrap() != strain.creator_id {
        return Err(LoginError::InvalidCredentials);
    }
    sqlx::query!(
        r#"
        DELETE
        FROM strains
        WHERE id = $1
        "#,
        strain_id
    )
    .execute(&mut **db)
    .await?;
    Ok(())
}

#[rocket::get("/strain/<strain_id_str>/version/<version_id_str>")]
async fn strain_version(
    mut db: Connection<Db>,
    strain_id_str: String,
    version_id_str: String,
) -> CoolerResult<Json<StrainVersion>> {
    let strain_id = Uuid::parse_str(&strain_id_str.to_string()).expect("Invalid strain id");
    let version_id =
        Uuid::parse_str(&version_id_str.to_string()).expect("Invalid strain version id");
    let strain_version = sqlx::query_as!(
        StrainVersion,
        r#"
        SELECT id, strain_id, code, created_at, updated_at, NULL::bytea AS wasm
        FROM strain_versions
        WHERE id = $1 AND strain_id = $2
        "#,
        version_id,
        strain_id
    )
    .fetch_one(&mut **db)
    .await?;

    Ok(Json(strain_version))
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct BattleRequest {
    strain_a: Uuid,
    strain_b: Uuid,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct BattleResult {
    id: Uuid,
    arena_size: i32,
    strain_a: Uuid,
    strain_b: Uuid,
    winner: Option<Uuid>,
    score_a: i32,
    score_b: i32,
    log: Vec<BattleLog>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct BattleLog {
    x: u32,
    y: u32,
    last: bool,
    allowed: bool,
}

use extism::*;

async fn get_latest_strain_version(
    db: &mut Connection<Db>,
    strain_id: Uuid,
) -> CoolerResult<StrainVersion> {
    let strain_version = sqlx::query_as!(
        StrainVersion,
        r#"
        SELECT id, strain_id, code, created_at, updated_at, wasm
        FROM strain_versions
        WHERE strain_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        strain_id
    )
    .fetch_one(&mut ***db)
    .await?;

    Ok(strain_version)
}

#[rocket::post("/battles/request", data = "<battle_request>")]
async fn battle(
    mut db: Connection<Db>,
    battle_request: Json<BattleRequest>,
    auth: Credential<OAuth>,
) -> CoolerResult<Json<BattleResult>> {
    const BOARD_SIZE: usize = 10;
    let user = get_user_from_token(&mut db, &auth.token)
        .await
        .ok_or(LoginError::InvalidCredentials)
        .expect("Invalid credentials");
    let battle_id = Uuid::new_v4();
    println!(
        "battle{battle_id}| user {} requested a battle between {} and {}",
        user.id.unwrap(),
        battle_request.strain_a,
        battle_request.strain_b
    );
    println!("battle{battle_id}| Loading strain versions");
    let strain_a = get_latest_strain_version(&mut db, battle_request.strain_a).await?;
    let strain_b = get_latest_strain_version(&mut db, battle_request.strain_b).await?;
    println!("battle{battle_id}| Loading wasm");

    let wasm_a = strain_a.wasm.expect("No wasm for strain a");
    let wasm_b = strain_b.wasm.expect("No wasm for strain b");

    let url = Wasm::data(wasm_a);
    let manifest = Manifest::new([url]);
    let mut plugin_a = Plugin::new(&manifest, [], true).expect("Failed to load plugin");

    let url = Wasm::data(wasm_b);
    let manifest = Manifest::new([url]);
    let mut plugin_b = Plugin::new(&manifest, [], true).expect("Failed to load plugin");

    println!("battle{battle_id}| Plugins loaded!");

    // None = empty
    // Some(true) = player a
    // Some(false) = player b
    let mut board: HashMap<(u32, u32), Option<bool>> = HashMap::new();
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            board.insert((x as u32, y as u32), None);
        }
    }

    let mut winner: Option<Uuid> = None;
    let mut log = vec![];

    // true = player a
    // false = player b
    let mut turn = false;
    loop {
        let empty = board
            .iter()
            .filter(|(_, v)| v.is_none())
            .map(|(k, _)| k)
            .collect::<HashSet<_>>();
        let occupied = board
            .iter()
            .filter(|(_, v)| v.is_some())
            .map(|(k, _)| k)
            .collect::<HashSet<_>>();
        let allowed = empty
            .iter()
            .filter(|(x, y)| {
                occupied
                    .iter()
                    .filter(|(ox, oy)| board.get(&(*ox, *oy)) == Some(&Some(turn)))
                    .any(|(ox, oy)| (ox.max(x) - ox.min(x)) <= 1 && (oy.max(y) - oy.min(y)) <= 1)
            })
            .collect::<HashSet<_>>();

        if empty.len() == 0 {
            break;
        }
        turn = !turn;
        let active_player = if turn { &mut plugin_a } else { &mut plugin_b };
        let board_serialized = serde_json::to_string(&board).expect("Failed to serialize board");
        let input = StrainInput { board, allowed };
        let response = active_player
            .call::<&str, &str>("take_turn", &board_serialized)
            .expect("Failed to call plugin");
        let response: (u32, u32) =
            serde_json::from_str(&response).expect("Failed to parse response");
        if allowed.contains(&&&response) {
            log.push((response, true));
            board.insert(response, Some(turn));
        } else {
            log.push((response, false));
            winner = Some(if turn {
                battle_request.strain_b
            } else {
                battle_request.strain_a
            });
            break;
        }
    }

    let score_a = board
        .iter()
        .filter(|(_, v)| v.is_some())
        .filter(|(_, v)| **v == Some(true))
        .count() as i32;
    let score_b = board
        .iter()
        .filter(|(_, v)| v.is_some())
        .filter(|(_, v)| **v == Some(false))
        .count() as i32;

    // save to db
    let log = log
        .iter()
        .enumerate()
        .map(|(i, ((x, y), allowed))| BattleLog {
            x: *x,
            y: *y,
            last: i == log.len() - 1,
            allowed: *allowed,
        })
        .collect::<Vec<_>>();
    sqlx::query!(
        r#"
        INSERT INTO battles (id, arena_size, strain_a, strain_b, winner, score_a, score_b)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        battle_id,
        BOARD_SIZE as i32,
        battle_request.strain_a,
        battle_request.strain_b,
        winner,
        score_a,
        score_b,
    )
    .execute(&mut **db)
    .await?;
    for log_entry in &log {
        sqlx::query!(
            r#"
            INSERT INTO battle_logs (battle_id, move_x, move_y, last, allowed)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            battle_id,
            log_entry.x as i32,
            log_entry.y as i32,
            log_entry.last,
            log_entry.allowed,
        )
        .execute(&mut **db)
        .await?;
    }

    Ok(Json(BattleResult {
        id: battle_id,
        arena_size: BOARD_SIZE as i32,
        strain_a: battle_request.strain_a,
        strain_b: battle_request.strain_b,
        winner,
        score_a,
        score_b,
        log,
    }))
}

use rocket::data::Limits;
use rocket::data::ToByteUnit;

#[rocket::launch]
async fn rocket() -> _ {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let cors = rocket_cors::CorsOptions::default().to_cors().unwrap();

    let limits = Limits::default().limit("json", 10.mebibytes());

    // set database url in config
    let rocket_config = Config::figment()
        .merge(("limits", limits))
        .merge(("address", "::"))
        .merge(("databases.sqlx.url", database_url))
        .merge(("databases.sqlx.pool_size", 5))
        .merge(("databases.sqlx.connect_timeout", 10));
    rocket::custom(rocket_config)
        .attach(cors)
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
        .mount(
            "/",
            routes![
                users,
                new_user,
                new_strain,
                strains,
                strain,
                delete_strain,
                new_strain_version,
                login,
                logout,
                validate_session,
                strain_version,
                battle
            ],
        )
}

async fn run_migrations(rocket: rocket::Rocket<rocket::Build>) -> rocket::fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!().run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                rocket::error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}
