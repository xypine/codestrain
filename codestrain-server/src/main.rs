use extism::*;
use rocket::State;
use sqlx::{Pool, Postgres};

#[rocket::get("/")]
async fn index(db: &State<Pool<Postgres>>) -> String {
    let db = (*db).clone();
    let users_count = sqlx::query!("SELECT COUNT(*) FROM users")
        .fetch_one(&db)
        .await
        .unwrap()
        .count
        .unwrap();
    format!("Hello, world! {} users", users_count)
}

#[rocket::launch]
async fn rocket() -> _ {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();
    sqlx::migrate!()
        .run(&db)
        .await
        .expect("Failed to migrate database");
    rocket::build()
        .mount("/", rocket::routes![index])
        .manage(db)
}
