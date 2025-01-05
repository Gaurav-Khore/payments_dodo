use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    models::{
        transactions::{add_transaction, get_transaction, list_all_transactions},
        users::get_user_by_id,
    },
    utilities::errors::api_error,
    AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionDataReq {
    pub receiver: Option<Uuid>,
    pub amount: Decimal,
    pub transaction_type: String,
}

pub async fn transaction(
    data: web::Data<AppState>,
    content: web::Json<TransactionDataReq>,
    req: HttpRequest,
) -> impl Responder {
    println!("Hello from the transaction");

    let pool = data.db.lock().unwrap();
    let id = req.extensions().get::<Uuid>().unwrap().clone();
    match get_user_by_id(&pool, id).await {
        Ok(v) => {
            match add_transaction(
                &pool,
                v.user_id.clone(),
                content.receiver.clone(),
                content.amount,
                content.transaction_type.clone(),
            )
            .await
            {
                Ok(_) => {
                    println!("Transaction added successfully");
                    return HttpResponse::Ok().json(json!({
                        "status": "Success",
                        "message":"Transaction added successfully"
                    }));
                }
                Err(e) => return api_error(e),
            }
        }
        Err(e) => api_error(e),
    }
}

pub async fn list_transactions(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    println!("Hello from the list transactions");
    let pool = data.db.lock().unwrap();

    let uid = req.extensions().get::<Uuid>().unwrap().clone();

    match list_all_transactions(&pool, uid).await {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(e) => api_error(e),
    }
}

#[derive(Serialize, Deserialize)]
pub struct FetchTransactionReq {
    pub transaction_id: Uuid,
}

pub async fn fetch_transaction(
    data: web::Data<AppState>,
    content: web::Json<FetchTransactionReq>,
    req: HttpRequest,
) -> impl Responder {
    println!("Hello from the fetch_transaction");

    let pool = data.db.lock().unwrap();
    let id = req.extensions().get::<Uuid>().unwrap().clone();
    match get_transaction(&pool, content.transaction_id).await {
        Ok(v) => {
            if v.sender != id {
                return HttpResponse::Unauthorized().json(json!({
                    "status": "Error",
                    "message" : "Unauthorized to access the trasaction"
                }));
            }
            HttpResponse::Ok().json(v)
        }
        Err(e) => api_error(e),
    }
}

#[cfg(test)]
mod tests {

    use std::sync::Mutex;

    use actix_web::{test, web, App};
    use rust_decimal::Decimal;
    use serde_json::{json, Value};

    use crate::{api::users::{get_token, user_register}, config::db::get_db, models::transactions::TransactionDetails, utilities::utils::JwtMiddleware, AppState};

    use super::{fetch_transaction, list_transactions, transaction};

    #[test]
    async fn test_transaction_deposit() {
        println!("Hello from the test transaction depost");

        let pool = match get_db().await {
            Ok(v) => v,
            Err(e) => panic!("Error at pool connection: {}", e),
        };

        let appdata = web::Data::new(AppState {
            db: Mutex::new(pool),
        });

        let app = test::init_service(
            App::new()
                .wrap(JwtMiddleware)
                .app_data(appdata)
                .route("/user/get_token", web::get().to(get_token))
                .route("/transaction/operations", web::post().to(transaction)),
        )
        .await;

        let req_body = json!({
            "email":"test@test.com",
            "password":"test"
        });

        let req = test::TestRequest::get()
            .uri("/user/get_token")
            .set_json(req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;

        let resp_body: Value = test::read_body_json(resp).await;

        let token = resp_body.get("token").unwrap().to_string().clone();
        let token = token.trim_start_matches("\"").trim_end_matches("\"");

        let req_body = json!({
            "amount": Decimal::new(1000,1),
            "transaction_type": "deposit"
        });

        let req = test::TestRequest::post().insert_header(("Authorization",format!("Bearer {}",token))).uri("/transaction/operations").set_json(req_body).to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(),actix_web::http::StatusCode::OK);

        let resp_body :Value = test::read_body_json(resp).await;
        assert_eq!(json!({
            "status": "Success",
            "message":"Transaction added successfully"
        }) , resp_body);
    }

    #[test]
    async fn test_transaction_withdrawl() {
        println!("Hello from the test transaction withdrawl");

        let pool = match get_db().await {
            Ok(v) => v,
            Err(e) => panic!("Error at pool connection: {}", e),
        };

        let appdata = web::Data::new(AppState {
            db: Mutex::new(pool),
        });

        let app = test::init_service(
            App::new()
                .wrap(JwtMiddleware)
                .app_data(appdata)
                .route("/user/get_token", web::get().to(get_token))
                .route("/transaction/operations", web::post().to(transaction)),
        )
        .await;

        let req_body = json!({
            "email":"test@test.com",
            "password":"test"
        });

        let req = test::TestRequest::get()
            .uri("/user/get_token")
            .set_json(req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;

        let resp_body: Value = test::read_body_json(resp).await;

        let token = resp_body.get("token").unwrap().to_string().clone();
        let token = token.trim_start_matches("\"").trim_end_matches("\"");

        let req_body = json!({
            "amount": Decimal::new(100,1),
            "transaction_type": "withdrawl"
        });

        let req = test::TestRequest::post().insert_header(("Authorization",format!("Bearer {}",token))).uri("/transaction/operations").set_json(req_body).to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(),actix_web::http::StatusCode::OK);

        let resp_body :Value = test::read_body_json(resp).await;
        assert_eq!(json!({
            "status": "Success",
            "message":"Transaction added successfully"
        }) , resp_body);
    }

    #[test]
    async fn test_transaction_transfer() {
        println!("Hello from the test transaction withdrawl");

        let pool = match get_db().await {
            Ok(v) => v,
            Err(e) => panic!("Error at pool connection: {}", e),
        };

        let appdata = web::Data::new(AppState {
            db: Mutex::new(pool),
        });

        let app = test::init_service(
            App::new()
                .wrap(JwtMiddleware)
                .app_data(appdata)
                .route("/user/register_user",web::post().to(user_register))
                .route("/user/get_token", web::get().to(get_token))
                .route("/transaction/operations", web::post().to(transaction)),
        )
        .await;

        //sender details
        let req_body = json!({
            "email":"test@test.com",
            "password":"test"
        });

        let req = test::TestRequest::get()
            .uri("/user/get_token")
            .set_json(req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;

        let resp_body: Value = test::read_body_json(resp).await;

        let token = resp_body.get("token").unwrap().to_string().clone();
        let token = token.trim_start_matches("\"").trim_end_matches("\"");

        //receiver details
        let req_body = json!({
            "username":"test_receiver",
            "email":"test1@test.com",
            "password":"test"
        });

        let req = test::TestRequest::post()
            .uri("/user/register_user")
            .set_json(&req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let resp_body:Value = test::read_body_json(resp).await;

        let recv_id = resp_body.get("user_id").unwrap();




        let req_body = json!({
            "receiver":recv_id,
            "amount": Decimal::new(100,1),
            "transaction_type": "transfer"
        });

        let req = test::TestRequest::post().insert_header(("Authorization",format!("Bearer {}",token))).uri("/transaction/operations").set_json(req_body).to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(),actix_web::http::StatusCode::OK);

        let resp_body :Value = test::read_body_json(resp).await;
        assert_eq!(json!({
            "status": "Success",
            "message":"Transaction added successfully"
        }) , resp_body);
    }

    #[test]
    async fn test_list_transaction() {
        println!("Hello from the test list transaction");

        let pool = match get_db().await {
            Ok(v) => v,
            Err(e) => panic!("Error at pool connection: {}", e),
        };

        let appdata = web::Data::new(AppState {
            db: Mutex::new(pool),
        });

        let app = test::init_service(
            App::new()
                .wrap(JwtMiddleware)
                .app_data(appdata)
                .route("/user/get_token", web::get().to(get_token))
                .route("/transaction/list_trans", web::get().to(list_transactions)),
        )
        .await;

        //sender details
        let req_body = json!({
            "email":"test@test.com",
            "password":"test"
        });

        let req = test::TestRequest::get()
            .uri("/user/get_token")
            .set_json(req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;

        let resp_body: Value = test::read_body_json(resp).await;

        let token = resp_body.get("token").unwrap().to_string().clone();
        let token = token.trim_start_matches("\"").trim_end_matches("\"");

        let req = test::TestRequest::get()
        .insert_header(("Authorization",format!("Bearer {}",token))).uri("/transaction/list_trans").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(),actix_web::http::StatusCode::OK);

    }

    #[test]
    async fn test_fetch_transaction() {
        println!("Hello from the test fetch transactions");

        let pool = match get_db().await {
            Ok(v) => v,
            Err(e) => panic!("Error at pool connection: {}", e),
        };

        let appdata = web::Data::new(AppState {
            db: Mutex::new(pool),
        });

        let app = test::init_service(
            App::new()
                .wrap(JwtMiddleware)
                .app_data(appdata)
                .route("/user/get_token", web::get().to(get_token))
                .route("/transaction/list_trans",web::get().to(list_transactions))
                .route("/transaction/fetch_transaction", web::get().to(fetch_transaction)),
        )
        .await;

        let req_body = json!({
            "email":"test@test.com",
            "password":"test"
        });

        let req = test::TestRequest::get()
            .uri("/user/get_token")
            .set_json(req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;

        let resp_body: Value = test::read_body_json(resp).await;

        let token = resp_body.get("token").unwrap().to_string().clone();
        let token = token.trim_start_matches("\"").trim_end_matches("\"");

        let req = test::TestRequest::get().uri("/transaction/list_trans").insert_header(("Authorization",format!("Bearer {}",token))).to_request();

        let resp = test::call_service(&app, req).await;

        let resp_body :Vec<TransactionDetails>  = test::read_body_json(resp).await;
        let data = &resp_body[0];
        let req_body = json!({
            "transaction_id":data.transaction_id
        });

        let req = test::TestRequest::get().uri("/transaction/fetch_transaction").insert_header(("Authorization",format!("Bearer {}",token))).set_json(req_body).to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(),actix_web::http::StatusCode::OK);
    }
}
