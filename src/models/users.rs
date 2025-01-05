use chrono::{NaiveDateTime, Utc};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

use crate::models::balance::add_balance_db;

pub async fn register_user(
    pool: &Pool<Postgres>,
    username: String,
    email: String,
    passwd: String,
) -> Result<Uuid, sqlx::Error> {
    println!("Hello from the register user");
    let qry = "
        INSERT INTO USERS (user_id,username,email,password,updated_at) VALUES ($1,$2,$3,$4,$5)
    ";

    let uuid = Uuid::new_v4();
    let updated_at = Utc::now();
    match sqlx::query(&qry)
    .bind(&uuid)
        .bind(&username)
        .bind(&email)
        .bind(&passwd)
        .bind(updated_at)
        .execute(pool)
        .await
    {
        Ok(v) => match add_balance_db(pool, uuid, dec!(0.0)).await {
            Ok(_) => Ok(uuid),
            Err(e) => return Err(e),
        },
        Err(e) => {
            println!("Error at Register User : {:?}", e);
            return Err(e);
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct UserInfo {
    pub id: i32,
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub password: String,
    pub created_at:NaiveDateTime ,
    pub updated_at: NaiveDateTime
}

pub async fn get_user(pool: &Pool<Postgres>, email: String) -> Result<UserInfo, sqlx::Error> {
    println!("Hello from the get_user");

    let qry = format!("SELECT * FROM USERS where email like '{}';", email);

    match sqlx::query(&qry).fetch_one(pool).await {
        Ok(v) => {
            println!("Data = {:?}", v);
            return Ok(UserInfo {
                id: v.get(0),
                user_id: v.get("user_id"),
                email: v.get("email"),
                username: v.get("username"),
                password: v.get("password"),
                created_at: v.get("created_at"),
                updated_at: v.get("updated_at")
            });
        }
        Err(e) => {
            println!("Error at get user : {:?}", e);
            return Err(e);
        }
    }
}

pub async fn update_user(
    pool: &Pool<Postgres>,
    id: Uuid,
    username: String,
) -> Result<(), sqlx::Error> {
    println!("Hello from the update user");

    let qry = "UPDATE users SET username=$1 where user_id = $2;";

    match sqlx::query(qry).bind(username).bind(id).execute(pool).await {
        Ok(v) => {
            println!("Data = {:?}", v);
            Ok(())
        }
        Err(e) => {
            println!("Error at update_user : {:?}", e);
            return Err(e);
        }
    }
}

pub async fn get_user_by_id(pool: &Pool<Postgres>, uuid: Uuid) -> Result<UserInfo, sqlx::Error> {
    println!("Hello from the get_user_by_id");

    let qry = "SELECT * from users where user_id = $1";

    match sqlx::query(qry).bind(uuid).fetch_one(pool).await {
        Ok(v) => {
            return Ok(UserInfo {
                id: v.get(0),
                user_id: v.get("user_id"),
                email: v.get("email"),
                username: v.get("username"),
                password: v.get("password"),
                updated_at: v.get("updated_at"),
                created_at: v.get("created_at")
            });
        }
        Err(e) => {
            return Err(e);
        }
    }
}
