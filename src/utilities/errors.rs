use actix_web::HttpResponse;
use serde_json::json;

pub fn api_error(error: sqlx::Error) -> HttpResponse {
    println!("Api Error");
    let mut message = "Invalid Request".to_string();

    match error {
        sqlx::Error::Database(db_err) => {
            println!("Db-error = {:?}", db_err.message());
            message = db_err.message().to_string();
        }
        sqlx::Error::Encode(db_err) => {
            println!("Error = {:?}", db_err);
            message = "Insufficient Balance".to_string();
        }
        sqlx::Error::RowNotFound => {
            println!("Not Found Error");
            message = "Requested item not found".to_string();
        }
        _ => {
            println!("Other error");
        }
    };
    HttpResponse::BadRequest().json(json!({
        "status": "Error",
        "message": message
    }))
}
