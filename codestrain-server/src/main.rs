use extism::*;
use rocket::fairing::AdHoc;
use rocket::response::status::Created;
use rocket::serde::Serialize;
use rocket::serde::{json::Json, Deserialize};
use rocket::{routes, Config, State};
use rocket_db_pools::{sqlx, Connection, Database};
use sqlx::{Pool, Postgres};

use futures::{future::TryFutureExt, stream::TryStreamExt};

#[derive(Database)]
#[database("sqlx")]
struct Db(sqlx::Pool<sqlx::Postgres>);

type CoolerResult<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[rocket::get("/")]
async fn index(mut db: Connection<Db>) -> CoolerResult<Json<Vec<NewUser>>> {
    let users = sqlx::query_as!(
        NewUser,
        r#"
        SELECT id, name, email, password
        FROM users
        ORDER BY id ASC
        "#
    )
    .fetch(&mut **db)
    .try_collect::<Vec<_>>()
    .await?;

    Ok(Json(users))
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct NewUser {
    name: String,
    #[serde(skip_serializing)]
    password: String,
    email: String,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    id: Option<i32>,
}

#[rocket::post("/user", data = "<new_user>")]
async fn new_user(
    mut db: Connection<Db>,
    mut new_user: Json<NewUser>,
) -> CoolerResult<Created<Json<NewUser>>> {
    let results = sqlx::query!(
        r#"
        INSERT INTO users (name, password, email)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        new_user.name,
        new_user.password,
        new_user.email
    )
    .fetch(&mut **db)
    .try_collect::<Vec<_>>()
    .await?;

    new_user.id = Some(results.first().expect("returning results").id);
    Ok(Created::new("/user").body(new_user))
}

#[rocket::launch]
async fn rocket() -> _ {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // set database url in config
    let rocket_config = Config::figment()
        .merge(("databases.sqlx.url", database_url))
        .merge(("databases.sqlx.pool_size", 5))
        .merge(("databases.sqlx.connect_timeout", 10));
    rocket::custom(rocket_config)
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
        .mount("/", routes![index, new_user])
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
