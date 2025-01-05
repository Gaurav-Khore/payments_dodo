use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

use crate::models::users::{get_user, get_user_by_id};

pub async fn add_balance_db(
    pool: &Pool<Postgres>,
    uuid: Uuid,
    amount: Decimal,
) -> Result<(), sqlx::Error> {
    println!("Hello from the add_balance db");

    let update_at = Utc::now();

    let qry = "INSERT INTO account_balance (account_id,user_id,balance,updated_at) VALUES ($1,$2,$3,$4)";

    match get_user_by_id(&pool, uuid).await {
        Ok(v) => {
            let mut amt: Decimal = amount;
            match get_balance(&pool, v.user_id).await {
                Ok(v) => {
                    amt = v.balance + amount;
                }
                Err(e) => {
                    println!("Error at get_baalnce = {}", e);
                }
            };
            let accout_id = Uuid::new_v4();
            match sqlx::query(&qry)
            .bind(&accout_id)
                .bind(v.user_id)
                .bind(amt)
                .bind(update_at)
                .execute(pool)
                .await
            {
                Ok(v) => Ok(()),
                Err(e) => {
                    println!("Error at add_balance_dv : {:?}", e);
                    return Err(e);
                }
            }
        }
        Err(e) => {
            println!("Error at add_balance_db : {:?}", e);
            return Err(e);
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[derive(PartialEq)]
#[derive(Debug)]
pub struct BalanceDetails {
    pub user_id: Uuid,
    pub balance: Decimal,
}

pub async fn get_balance(pool: &Pool<Postgres>, uid: Uuid) -> Result<BalanceDetails, sqlx::Error> {
    println!("Hello from the get_balance");

    let qry = "SELECT * FROM account_balance where user_id = $1";

    match sqlx::query(&qry).bind(uid).fetch_one(pool).await {
        Ok(v) => Ok(BalanceDetails {
            balance: v.get("balance"),
            user_id: v.get("user_id"),
        }),
        Err(e) => {
            println!("Error at get_balance");
            return Err(e);
        }
    }
}

pub async fn update_balance(
    pool: &Pool<Postgres>,
    uid: Uuid,
    new_bal: Decimal,
) -> Result<(), sqlx::Error> {
    println!("Hello from the update balance");

    let qry = "UPDATE account_balance SET balance=$1 where user_id = $2";

    match sqlx::query(qry).bind(new_bal).bind(uid).execute(pool).await {
        Ok(_) => {
            println!("Balance updated successfully");
            Ok(())
        }
        Err(e) => {
            println!("Error at update balance: {:?}", e);
            return Err(e);
        }
    }
}
