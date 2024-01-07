use codestrain_common::*;

use extism::*;
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

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

#[derive(Database)]
#[database("sqlx")]
struct Db(sqlx::Pool<sqlx::Postgres>);

type CoolerResult<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct User {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    id: Option<Uuid>,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    admin: Option<bool>,
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
    let admin = new_user.email == "elias@ruta.fi";

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(new_user.password.as_bytes(), &salt)
        .map_err(|_| NewUserError::InternalError)?
        .to_string();

    let r = sqlx::query!(
        r#"
        INSERT INTO users (name, password, email, admin)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        new_user.name,
        password_hash,
        new_user.email,
        admin
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
        SELECT id, admin, name, email, password, created_at, updated_at
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
    #[error("Internal error")]
    LoadingPasswordHashFailed,
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
    // Get user from db
    let r = sqlx::query!(
        r#"
        SELECT id, name, email, password, created_at, updated_at
        FROM users
        WHERE email = $1
        "#,
        credentials.email
    )
    .fetch_optional(&mut **db)
    .await
    .map_err(LoginError::DbError)?;

    match r {
        Some(user) => {
            // Validate user credentials
            let parsed_hash = PasswordHash::new(&user.password)
                .map_err(|_| LoginError::LoadingPasswordHashFailed)?;
            Argon2::default()
                .verify_password(credentials.password.as_bytes(), &parsed_hash)
                .map_err(|_| LoginError::InvalidCredentials)?;

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

use serde_with::base64::Base64;
use serde_with::As;
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct Strain {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    id: Option<Uuid>,
    name: String,
    description: Option<String>,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    creator_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "As::<Option<Base64>>")]
    wasm: Option<Vec<u8>>,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    wasm_hash: Option<String>,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    created_at: Option<PrimitiveDateTime>,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    updated_at: Option<PrimitiveDateTime>,
}

#[rocket::get("/strain?<creator_id>")]
async fn strains(
    mut db: Connection<Db>,
    creator_id: Option<String>,
) -> CoolerResult<Json<Vec<Strain>>> {
    let creator_id = creator_id.map(|s| Uuid::parse_str(&s).ok()).flatten();
    let strains = sqlx::query_as!(
        Strain,
        r#"
        SELECT id, name, description, creator_id, NULL AS code, NULL::bytea AS wasm, created_at, updated_at, wasm_hash
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
        SELECT users.id, users.admin, users.name, users.email, users.password, users.created_at, users.updated_at
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

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum NewStrainError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Constraint violated")]
    ConstraintViolated,
    #[error("Internal error")]
    InternalError,
}
impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for NewStrainError {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        println!("{:?}", self);

        match self {
            Self::InvalidCredentials => Status::Unauthorized,
            Self::ConstraintViolated => Status::Conflict,
            _ => Status::InternalServerError,
        }
        .respond_to(req)
    }
}
use sha2::Digest;
#[rocket::post("/strain", data = "<new_strain>")]
async fn new_strain(
    mut db: Connection<Db>,
    mut new_strain: Json<Strain>,
    auth: Credential<OAuth>,
) -> Result<Created<Json<Strain>>, NewStrainError> {
    let user = get_user_from_token(&mut db, &auth.token)
        .await
        .ok_or(NewStrainError::InvalidCredentials)?;
    new_strain.creator_id = Some(user.id.unwrap());
    new_strain.wasm_hash = Some(
        sha2::Sha256::digest(&new_strain.wasm.as_ref().unwrap())
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>(),
    );
    let r = sqlx::query!(
        r#"
        INSERT INTO strains (name, description, creator_id, code, wasm, wasm_hash)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
        new_strain.name,
        new_strain.description,
        new_strain.creator_id,
        new_strain.code,
        new_strain.wasm,
        new_strain.wasm_hash
    )
    .fetch_optional(&mut **db)
    .await;

    match r {
        Ok(Some(row)) => {
            // Set the new user's id from the returned value and return as before
            new_strain.id = Some(row.id);
            // clear code and wasm to save bandwidth
            new_strain.code = None;
            new_strain.wasm = None;
            return Ok(Created::new("/strain").body(new_strain));
        }
        Err(sqlx::Error::Database(e)) => {
            println!("{:?}", e);
            if e.constraint().is_some() {
                // The constraint was violated, so we know the user already exists
                return Err(NewStrainError::ConstraintViolated);
            }
        }
        _ => {}
    }
    Err(NewStrainError::InternalError)
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct StrainWithoutWasm {
    id: Uuid,
    name: String,
    description: Option<String>,
    code: String,
    wasm_hash: String,
    wasm_size: Option<i32>,
    creator_id: Uuid,
    created_at: PrimitiveDateTime,
    updated_at: PrimitiveDateTime,
}

#[rocket::get("/strain/<strain_id_str>")]
async fn strain(
    mut db: Connection<Db>,
    strain_id_str: String,
) -> CoolerResult<Json<StrainWithoutWasm>> {
    let strain_id = Uuid::parse_str(&strain_id_str.to_string()).expect("Invalid strain id");
    let strain = sqlx::query_as!(
        StrainWithoutWasm,
        r#"
        SELECT id, name, creator_id, created_at, updated_at, description, code, octet_length(wasm) AS wasm_size, wasm_hash
        FROM strains
        WHERE id = $1
        "#,
        strain_id
    )
    .fetch_one(&mut **db)
    .await?;

    Ok(Json(strain))
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
    player: bool,
    x: i32,
    y: i32,
    allowed: bool,
}

async fn get_latest_strain_version(
    db: &mut Connection<Db>,
    strain_id: Uuid,
) -> CoolerResult<Strain> {
    let strain = sqlx::query_as!(
        Strain,
        r#"
        SELECT id, name, description, creator_id, code, wasm, created_at, updated_at, wasm_hash
        FROM strains
        WHERE id = $1
        "#,
        strain_id
    )
    .fetch_one(&mut ***db)
    .await?;

    Ok(strain)
}

async fn process_turn(
    battle_id: Uuid,
    board: &Vec<((i32, i32), Option<bool>)>,
    plugin: &mut Plugin,
) -> Option<(i32, i32)> {
    let empty = board
        .iter()
        .filter(|(_, v)| v.is_none())
        .map(|(k, _)| *k)
        .collect::<Vec<_>>();
    let occupied = board
        .iter()
        .filter(|(_, v)| v.is_some())
        .map(|(k, v)| (*k, *v))
        .collect::<Vec<_>>();
    let friendly = occupied
        .iter()
        .filter(|(_, v)| *v == Some(true))
        .map(|(k, _)| *k)
        .collect::<Vec<_>>();
    let _enemy = occupied
        .iter()
        .filter(|(_, v)| *v == Some(false))
        .map(|(k, _)| *k)
        .collect::<Vec<_>>();
    // Only allow moves that are directly adjacent to an occupied square (with diagonals) and are the same color
    let allowed = empty
        .iter()
        .filter(|(x, y)| {
            friendly.iter().any(|(ox, oy)| {
                (x == ox && (y - oy).abs() == 1) || (y == oy && (x - ox).abs() == 1)
            })
        })
        .map(|(x, y)| (*x, *y))
        .collect::<Vec<_>>();
    /*
    println!(
        "battle{battle_id}| Turn: {}, Allowed moves: {:?}, Occupied: {:?}",
        if turn { "A" } else { "B" },
        allowed,
        occupied
    ); */
    if allowed.len() == 0 {
        return None;
    }

    let input = StrainInput {
        board: board.clone(),
        allowed: allowed.clone(),
    };
    let extism::convert::Json(response) = plugin
        .call::<extism::convert::Json<StrainInput>, extism::convert::Json<StrainOutput>>(
            "take_turn",
            extism::convert::Json(input),
        )
        .expect("Failed to call plugin");
    if !allowed.contains(&response) {
        println!("battle{battle_id}| Invalid move: {:?}", response);
        return None;
    }
    Some(response)
}

#[rocket::post("/battle", data = "<battle_request>")]
async fn battle(
    mut db: Connection<Db>,
    battle_request: Json<BattleRequest>,
    auth: Credential<OAuth>,
) -> CoolerResult<Json<BattleResult>> {
    const BOARD_SIZE: usize = 24;
    let user = get_user_from_token(&mut db, &auth.token)
        .await
        .ok_or(LoginError::InvalidCredentials)
        .expect("Invalid credentials");
    if !user.admin.unwrap() {
        panic!("User is not admin");
    }
    let battle_id = Uuid::new_v4();
    println!(
        "battle{battle_id}| user {} requested a battle between {} and {}",
        user.id.unwrap(),
        battle_request.strain_a,
        battle_request.strain_b
    );
    // check for existing result
    let row = sqlx::query!(
        r#"
        SELECT id, arena_size, strain_a, strain_b, winner, score_a, score_b
        FROM battles
        WHERE strain_a = $1 AND strain_b = $2
        "#,
        battle_request.strain_a,
        battle_request.strain_b
    )
    .fetch_optional(&mut **db)
    .await?;
    if let Some(old) = row {
        println!("battle{battle_id}| Battle already exists, deleting the old one");
        sqlx::query!(
            r#"
            DELETE FROM battles
            WHERE id = $1
            "#,
            old.id
        )
        .execute(&mut **db)
        .await?;
        sqlx::query!(
            r#"
            DELETE FROM battle_logs
            WHERE battle_id = $1
            "#,
            old.id
        )
        .execute(&mut **db)
        .await?;
    }

    /*
    if let Some(row) = row {
        println!("battle{battle_id}| Battle already exists");
        let log = sqlx::query_as!(
            BattleLog,
            r#"
            SELECT move_x AS x, move_y AS y, last, allowed
            FROM battle_logs
            WHERE battle_id = $1
            ORDER BY turn ASC
            "#,
            battle_id
        )
        .fetch(&mut **db)
        .try_collect::<Vec<_>>()
        .await?;
        return Ok(Json(BattleResult {
            id: row.id,
            arena_size: row.arena_size,
            strain_a: row.strain_a,
            strain_b: row.strain_b,
            winner: row.winner,
            score_a: row.score_a,
            score_b: row.score_b,
            log,
        }));
    } */
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
    let mut board = vec![];
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            let value = if x == 0 && y == 0 {
                Some(true)
            } else if x == BOARD_SIZE - 1 && y == BOARD_SIZE - 1 {
                Some(false)
            } else {
                None
            };
            board.push(((x as i32, y as i32), value));
        }
    }

    let mut log = vec![];

    // true = player a
    // false = player b
    let mut turn = false;
    let mut skips = 0;
    const MOVES_PER_TURN: usize = 5;
    'outer: loop {
        turn = !turn;
        for _ in 0..MOVES_PER_TURN {
            // rotate the board so that the active player is always at the top left
            // this way the plugin doesn't have to worry about the board rotation
            // also change the mappings so that friendly cells are always true and enemy cells are always false
            let board_copy = if turn {
                board.clone()
            } else {
                board
                    .iter()
                    .map(|((x, y), v)| {
                        (
                            ((BOARD_SIZE as i32 - 1) - y, (BOARD_SIZE as i32 - 1) - x),
                            v.map(|v| if turn { v } else { !v }),
                        )
                    })
                    .collect::<Vec<_>>()
            };
            // (0, 0) should always be true
            if !board_copy
                .iter()
                .any(|((x, y), v)| *x == 0 && *y == 0 && *v == Some(true))
            {
                // something extremely weird happened
                println!(
                    "battle{battle_id}| Something weird happened ((0,0) != true), ending game"
                );
                println!(
                    "battle{battle_id}| Board: {:?}",
                    board_copy
                        .iter()
                        .filter(|(_, v)| v.is_some())
                        .map(|((x, y), v)| (*x, *y, v.unwrap()))
                        .collect::<Vec<_>>()
                );
                break;
            }

            let active_player = if turn { &mut plugin_a } else { &mut plugin_b };
            let response = process_turn(battle_id, &board_copy, active_player).await;

            if let Some(response) = response {
                // rotate the response so that it matches the original board
                let corrected_response: StrainOutput = if !turn {
                    (
                        (BOARD_SIZE as i32 - 1) - response.1,
                        (BOARD_SIZE as i32 - 1) - response.0,
                    )
                } else {
                    response
                };
                println!(
                    "battle{battle_id}| Player {} made the move {:?} -> {:?}",
                    if turn { "A" } else { "B" },
                    response,
                    corrected_response
                );
                log.push((corrected_response, turn, true));
                board
                    .iter_mut()
                    .filter(|(k, _)| *k == corrected_response)
                    .for_each(|(_, v)| *v = Some(turn));
                skips = 0;
            } else {
                println!(
                    "battle{battle_id}| No moves available, skipping turn ({} skips in total)",
                    skips
                );
                skips += 1;
                if skips >= 2 * MOVES_PER_TURN {
                    println!("battle{battle_id}| Both players skipped, ending game");
                    break 'outer;
                }
                continue;
            }
        }
    }

    let score_a = board.iter().filter(|(_, v)| *v == Some(true)).count() as i32;
    let score_b = board.iter().filter(|(_, v)| *v == Some(false)).count() as i32;
    let winner = if score_a > score_b {
        Some(strain_a.id.unwrap())
    } else if score_b > score_a {
        Some(strain_b.id.unwrap())
    } else {
        None
    };

    // save to db
    let log = log
        .iter()
        .map(|((x, y), player, allowed)| BattleLog {
            player: *player,
            x: *x as i32,
            y: *y as i32,
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
        strain_a.id,
        strain_b.id,
        winner,
        score_a,
        score_b,
    )
    .execute(&mut **db)
    .await?;
    for (turn, log_entry) in log.iter().enumerate() {
        sqlx::query!(
            r#"
            INSERT INTO battle_logs (battle_id, turn, move_x, move_y, player, allowed)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            battle_id,
            turn as i32,
            log_entry.x,
            log_entry.y,
            log_entry.player,
            log_entry.allowed,
        )
        .execute(&mut **db)
        .await?;
    }

    Ok(Json(BattleResult {
        id: battle_id,
        arena_size: BOARD_SIZE as i32,
        strain_a: strain_a.id.unwrap(),
        strain_b: strain_b.id.unwrap(),
        winner,
        score_a,
        score_b,
        log,
    }))
}

#[rocket::get("/battle/<battle_id_str>")]
async fn get_battle(
    mut db: Connection<Db>,
    battle_id_str: String,
) -> CoolerResult<Json<BattleResult>> {
    let battle_id = Uuid::parse_str(&battle_id_str.to_string()).expect("Invalid battle id");
    let row = sqlx::query!(
        r#"
        SELECT id, arena_size, strain_a, strain_b, winner, score_a, score_b
        FROM battles
        WHERE id = $1
        "#,
        battle_id
    )
    .fetch_one(&mut **db)
    .await?;
    let log = sqlx::query_as!(
        BattleLog,
        r#"
        SELECT move_x AS x, move_y AS y, allowed, player
        FROM battle_logs
        WHERE battle_id = $1
        ORDER BY turn ASC
        "#,
        battle_id
    )
    .fetch(&mut **db)
    .try_collect::<Vec<_>>()
    .await?;

    Ok(Json(BattleResult {
        id: row.id,
        arena_size: row.arena_size,
        strain_a: row.strain_a,
        strain_b: row.strain_b,
        winner: row.winner,
        score_a: row.score_a,
        score_b: row.score_b,
        log,
    }))
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "rocket::serde")]
struct BattleIndex {
    id: Uuid,
    arena_size: i32,
    strain_a: Uuid,
    strain_b: Uuid,
    winner: Option<Uuid>,
    score_a: i32,
    score_b: i32,
}

#[rocket::get("/battle")]
async fn battles(mut db: Connection<Db>) -> CoolerResult<Json<Vec<BattleIndex>>> {
    let battles = sqlx::query_as!(
        BattleIndex,
        r#"
        SELECT battles.id, arena_size, strain_a, strain_b, winner, score_a, score_b
        FROM battles
        ORDER BY created_at DESC
        "#
    )
    .fetch(&mut **db)
    .try_collect::<Vec<_>>()
    .await?;

    Ok(Json(battles))
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
                login,
                logout,
                validate_session,
                battle,
                get_battle,
                battles
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
