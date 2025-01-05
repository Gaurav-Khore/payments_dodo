use sqlx::{postgres::PgPoolOptions, Error, Pool, Postgres};

//function to retrive the database connection
pub async fn get_db() -> Result<Pool<Postgres>, sqlx::Error> {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    match PgPoolOptions::new().connect(&url).await {
        Ok(v) => {
            match db_config(&v).await {
                Ok(_) => {
                    return Ok(v);
                }
                Err(e) => {
                    return Err(e);
                }
            };
        }
        Err(e) => {
            println!("Error on connecting the postgress server");
            return Err(e);
        }
    };
}

//function to create the database
pub async fn db_config(pool: &Pool<Postgres>) -> Result<(), Error> {
    println!("Hello from the db_config");

    //users
    let user_qry = "
        CREATE TABLE IF NOT EXISTS users(
            id SERIAL UNIQUE NOT NULL,
            user_id UUID UNIQUE NOT NULL PRIMARY KEY ,
            username VARCHAR(255) NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL,
            password VARCHAR(255) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP
        );
    ";

    match sqlx::query(&user_qry).execute(pool).await {
        Ok(_) => (),
        Err(e) => {
            println!("Error at user table creation : {:?}", e);
            return Err(e);
        }
    };

    //transactions
    //transactions type -> trnasfer, deposit, withdrawl
    //status -> pending, completed, failed
    let trans_qry = "
        CREATE TABLE IF NOT EXISTS transactions (
            id SERIAL PRIMARY KEY,
            transaction_id UUID UNIQUE NOT NULL,
            sender_id UUID REFERENCES users(user_id) ON DELETE CASCADE,
            receiver_id UUID REFERENCES users(user_id) ON DELETE CASCADE,
            amount DECIMAL(10,2) NOT NULL,
            transaction_type VARCHAR(50),
            status VARCHAR(20) DEFAULT 'pending',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP
        );
    ";

    match sqlx::query(&trans_qry).execute(pool).await {
        Ok(_) => (),
        Err(e) => {
            println!("Error at transaction table creation : {:?}", e);
            return Err(e);
        }
    };
    //account_balance
    let bal_qry = "
        CREATE TABLE IF NOT EXISTS account_balance (
            id SERIAL PRIMARY KEY,
            account_id UUID UNIQUE NOT NULL,
            user_id UUID REFERENCES users(user_id) ON DELETE CASCADE,
            balance DECIMAL(15,12) NOT NULL DEFAULT 0.00,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP
        );
    ";

    match sqlx::query(&bal_qry).execute(pool).await {
        Ok(_) => (),
        Err(e) => {
            println!("Error at account_balance creation : {:?}", e);
            return Err(e);
        }
    };
    Ok(())
}
