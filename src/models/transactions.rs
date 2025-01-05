use chrono::{DateTime, NaiveDateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{pool, Pool, Postgres, Row};
use uuid::Uuid;

use crate::models::{
    balance::{get_balance, update_balance},
    users::{get_user, get_user_by_id},
};

pub async fn add_transaction(
    pool: &Pool<Postgres>,
    sender: Uuid,
    receiver: Option<Uuid>,
    amount: Decimal,
    transaction_type: String,
) -> Result<(), sqlx::Error> {
    println!("Hello from the add transactions");
    println!("receiverr = {:?}", receiver);
    let sender_details = match get_user_by_id(pool, sender.clone()).await {
        Ok(v) => match get_balance(pool, v.user_id).await {
            Ok(v) => v,
            Err(e) => return Err(e),
        },
        Err(e) => {
            return Err(e);
        }
    };

    let mut receiver_details = sender_details;

    let mut send_update_balance = sender_details.balance - amount;
    let mut recev_update_balance = receiver_details.balance + amount;
    let mut flag = true;

    match transaction_type.to_uppercase().as_str() {
        "WITHDRAWL" => {
            if receiver.is_some() {
                return Err(sqlx::Error::Encode(
                    String::from("Cannot be done for different account").into(),
                ));
            }

            if sender_details.balance < amount {
                return Err(sqlx::Error::Encode(
                    String::from("Insufficient Balance").into(),
                ));
            }
            receiver_details = sender_details;

            flag = false;
        }
        "DEPOSIT" => {
            if receiver.is_some() {
                return Err(sqlx::Error::Encode(
                    String::from("Cannot be done for different account").into(),
                ));
            }
            receiver_details = sender_details;
            send_update_balance = recev_update_balance;
            flag = false;
        }
        "TRANSFER" => {
            if receiver.is_none() || sender == receiver.clone().unwrap() {
                return Err(sqlx::Error::Encode(
                    String::from("Cannot be done for same user").into(),
                ));
            }
            if sender_details.balance < amount {
                return Err(sqlx::Error::Encode(
                    String::from("Insufficient Balance").into(),
                ));
            }
            receiver_details = match get_user_by_id(pool, receiver.unwrap().clone()).await {
                Ok(v) => match get_balance(pool, v.user_id).await {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                },
                Err(e) => {
                    return Err(e);
                }
            };
            recev_update_balance = receiver_details.balance + amount;
        }
        _ => {
            return Err(sqlx::Error::TypeNotFound {
                type_name: transaction_type,
            })
        }
    }
    let transaction_id = Uuid::new_v4();
    let update_at = Utc::now();
    let qry = "INSERT INTO transactions(transaction_id,sender_id,receiver_id,amount,transaction_type,status,updated_at) Values ($1,$2,$3,$4,$5,$6,$7);";
    match sqlx::query(qry)
        .bind(transaction_id)
        .bind(sender_details.user_id)
        .bind(receiver_details.user_id)
        .bind(amount)
        .bind(transaction_type.to_lowercase())
        .bind("pending")
        .bind(update_at)
        .execute(pool)
        .await
    {
        Ok(_) => {
            //sender balance update
            match update_balance(pool, sender_details.user_id, send_update_balance).await {
                Ok(_) => {
                    if flag {
                        //receiver balance update
                        match update_balance(pool, receiver_details.user_id, recev_update_balance)
                            .await
                        {
                            Ok(_) => {
                                match update_transaction_status(
                                    pool,
                                    String::from("completed"),
                                    transaction_id,
                                )
                                .await
                                {
                                    Ok(_) => Ok(()),
                                    Err(e) => return Err(e),
                                }
                            }
                            Err(e) => return Err(e),
                        }
                    } else {
                        match update_transaction_status(pool, String::from("completed"), transaction_id).await
                        {
                            Ok(_) => Ok(()),
                            Err(e) => return Err(e),
                        }
                    }
                }
                Err(e) => {
                    match update_transaction_status(pool, String::from("failed"), transaction_id).await {
                        Ok(_) => (),
                        Err(e) => println!("Error : {:?}", e),
                    };
                    return Err(e);
                }
            }
        }
        Err(e) => return Err(e),
    }
}

pub async fn update_transaction_status(
    pool: &Pool<Postgres>,
    status: String,
    uuid: Uuid,
) -> Result<(), sqlx::Error> {
    println!("Hello from the update transaction status");
    // let qry = "UPDATE transactions SET status=$1 where sender_id=$2 and receiver_id=$3 and amount=$4 and transaction_type like '$5' and status like 'pending' and updated_at = TO_TIMESTAMP($6, 'YYYY-MM-DD HH24:MI:SS.US');";
    let qry = "UPDATE transactions SET status=$1 where transaction_id=$2;";

    match sqlx::query(&qry)
        .bind(status.to_lowercase())
        .bind(uuid)
        .execute(pool)
        .await
    {
        Ok(_) => {
            println!("Transaction status updated successfully");
            Ok(())
        }
        Err(e) => return Err(e),
    }
}

#[derive(Debug, Serialize, Deserialize)]

pub struct TransactionDetails {
    pub transaction_id: Uuid,
    pub sender: Uuid,
    pub receiver: Uuid,
    pub amount: Decimal,
    pub transaction_type: String,
    pub status: String,
    pub created_at:NaiveDateTime,
    pub updated_at: NaiveDateTime
}
pub async fn list_all_transactions(
    pool: &Pool<Postgres>,
    id: Uuid,
) -> Result<Vec<TransactionDetails>, sqlx::Error> {
    println!("Hello from the list_transactions");

    let qry = "SELECT distinct * FROM transactions where sender_id=$1 or receiver_id=$1;";
    let mut res = vec![];
    match sqlx::query(qry).bind(id).fetch_all(pool).await {
        Ok(v) => {
            for i in v {
                res.push(TransactionDetails {
                    transaction_id: i.get("transaction_id"),
                    sender: i.get("sender_id"),
                    receiver: i.get("receiver_id"),
                    amount: i.get("amount"),
                    status: i.get("status"),
                    transaction_type: i.get("transaction_type"),
                    created_at: i.get("created_at"),
                    updated_at: i.get("updated_at")
                });
            }
            return Ok(res);
        }
        Err(e) => return Err(e),
    }
}

pub async fn get_transaction(
    pool: &Pool<Postgres>,
    uuid: Uuid,
) -> Result<TransactionDetails, sqlx::Error> {
    println!("Hello from the get_transactions");

    let qry = "Select * from transactions where transaction_id = $1;";

    match sqlx::query(qry).bind(uuid).fetch_one(pool).await {
        Ok(v) => Ok(TransactionDetails {
            transaction_id: v.get("transaction_id"),
            sender: v.get("sender_id"),
            receiver: v.get("receiver_id"),
            amount: v.get("amount"),
            status: v.get("status"),
            transaction_type: v.get("transaction_type"),
            created_at: v.get("created_at"),
            updated_at: v.get("updated_at")
        }),
        Err(e) => {
            return Err(e);
        }
    }
}
