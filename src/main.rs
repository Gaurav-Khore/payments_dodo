use std::sync::Mutex;

use actix_web::{web, App, HttpServer};
use api::{
    balance::fetch_balance,
    transactions::{fetch_transaction, list_transactions, transaction},
    users::{get_token, get_user_details, user_register, user_update},
};
use config::db::get_db;
use sqlx::{Pool, Postgres};
use utilities::utils::JwtMiddleware;

pub mod config {
    pub mod db;
}
pub mod api {
    pub mod balance;
    pub mod transactions;
    pub mod users;
}

pub mod models {
    pub mod balance;
    pub mod transactions;
    pub mod users;
}

pub mod utilities {
    pub mod auth;
    pub mod errors;
    pub mod utils;
}

pub struct AppState {
    db: Mutex<Pool<Postgres>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    //get the pool connection
    let pool = match get_db().await {
        Ok(v) => v,
        Err(e) => panic!("Error at pool connection = {:?}", e),
    };

    let appdata = web::Data::new(AppState {
        db: Mutex::new(pool),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(JwtMiddleware)
            .app_data(appdata.clone())
            .service(
                web::scope("/user")
                    .route("/register_user", web::post().to(user_register))
                    .route("/get_token", web::get().to(get_token))
                    .route("/get_user", web::get().to(get_user_details))
                    .route("/update_user", web::post().to(user_update)),
            )
            .service(web::scope("/balance").route("/fetch_balance", web::get().to(fetch_balance)))
            .service(
                web::scope("/transaction")
                    .route("/operations", web::post().to(transaction))
                    .route("/fetch_transaction", web::get().to(fetch_transaction))
                    .route("/list_trans", web::get().to(list_transactions)),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
