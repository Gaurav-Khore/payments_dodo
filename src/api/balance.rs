use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    models::{
        balance::get_balance,
        transactions::add_transaction,
        users::get_user_by_id,
    },
    utilities::errors::api_error,
    AppState,
};

#[derive(Serialize, Deserialize)]
pub struct AddBalanceReq {
    pub amount: Decimal,
}
pub async fn add_balance(
    data: web::Data<AppState>,
    content: web::Json<AddBalanceReq>,
    req: HttpRequest,
) -> impl Responder {
    println!("Hello from the add balance api");
    let uid = req.extensions().get::<Uuid>().unwrap().clone();
    let pool = data.db.lock().unwrap();

    match get_balance(&pool, uid).await {
        Ok(v) => {
            let new_bal: Decimal = v.balance + content.amount;
            match get_user_by_id(&pool, uid).await {
                Ok(v) => {
                    match add_transaction(
                        &pool,
                        v.user_id,
                        None,
                        new_bal,
                        String::from("Deposit"),
                    )
                    .await
                    {
                        Ok(_) => HttpResponse::Ok().json(json!({
                            "status": "Success",
                            "message": "Balance Added Successfully"
                        })),
                        Err(e) => api_error(e),
                    }
                }
                Err(e) => api_error(e),
            }
        }
        Err(e) => return api_error(e),
    }
}

pub async fn fetch_balance(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let pool = data.db.lock().unwrap();
    println!("{:?}", req.extensions());
    let uid = req.extensions().get::<Uuid>().unwrap().clone();
    println!("uid = {}", uid);
    match get_balance(&pool, uid).await {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(e) => api_error(e),
    }
}

#[cfg(test)]
mod test {
    use std::sync::Mutex;

    use actix_web::{test, web, App};
    use rust_decimal::Decimal;
    use serde_json::{json, Value};

    use crate::{api::users::get_token, config::db::get_db, models::balance::BalanceDetails, utilities::utils::JwtMiddleware, AppState};

    use super::fetch_balance;

    #[test]
    async fn test_fetch_balance() {
        println!("Hello from the test fetch balance");
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
                .route("/balance/fetch_balance", web::get().to(fetch_balance)),
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
        let req = test::TestRequest::get().uri("/balance/fetch_balance").insert_header(("Authorization",format!("Bearer {}",token))).to_request();
        let resp =test::call_service(&app, req).await;
        assert_eq!(resp.status(),actix_web::http::StatusCode::OK);
    }
}