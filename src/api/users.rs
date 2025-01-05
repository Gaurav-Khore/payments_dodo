use crate::{
    models::users::{get_user, get_user_by_id, register_user, update_user},
    utilities::{auth::encode_jwt, errors::api_error},
    AppState,
};
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserRegisterReq {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub async fn user_register(
    data: web::Data<AppState>,
    content: web::Json<UserRegisterReq>,
) -> impl Responder {
    println!("Hello from the user registration api");
    let pool = data.db.lock().unwrap();
    match register_user(
        &pool,
        content.username.clone(),
        content.email.clone(),
        content.password.clone(),
    )
    .await
    {
        Ok(v) => HttpResponse::Ok().json(json!(
            {"status": "Success",
            "message": "User Registration Successfully",
            "detailed_Message" : "N/A"}
        )),
        Err(e) => {
            return api_error(e);
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct UserDetailsReq {
    pub user_id: Uuid,
}

pub async fn get_user_details(
    data: web::Data<AppState>,
    content: web::Json<UserDetailsReq>,
) -> impl Responder {
    println!("Hello from the get user details");

    let pool = data.db.lock().unwrap();

    match get_user_by_id(&pool, content.user_id.clone()).await {
        Ok(v) => {
            println!("Get User Successfully");
            return HttpResponse::Ok().json(json!({
                "id":v.id,
                "username":v.username,
                "email":v.email,
                "user_id":v.user_id,
                "created_at":v.created_at,
                "updated_at":v.updated_at
            }));
        }
        Err(e) => {
            return api_error(e);
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserUpdateReq {
    pub username: String,
}

pub async fn user_update(
    data: web::Data<AppState>,
    content: web::Json<UserUpdateReq>,
    req: HttpRequest,
) -> impl Responder {
    println!("Hello from the user_update");

    let pool = data.db.lock().unwrap();
    let id = req.extensions().get::<Uuid>().unwrap().clone();
    match update_user(&pool, id, content.username.clone()).await {
        Ok(_) => {
            println!("UserName update successully");
            return HttpResponse::Ok().json(json!(
                {
                    "status":"Success",
                    "message": "User Updated Successfully"
                }
            ));
        }
        Err(e) => {
            return api_error(e);
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetTokenReq {
    pub email: String,
    pub password: String,
}

pub async fn get_token(
    data: web::Data<AppState>,
    content: web::Json<GetTokenReq>,
) -> impl Responder {
    println!("Hello from the Sign in");
    let pool = data.db.lock().unwrap();
    match get_user(&pool, content.email.clone()).await {
        Ok(v) => {
            if v.password == content.password {
                let token = match encode_jwt(v.user_id as Uuid) {
                    Ok(v) => v,
                    Err(e) => {
                        return HttpResponse::Unauthorized().json(json!(
                            {
                                "status": "Error",
                                "message":"Invalid Credentials"
                            }
                        ))
                    }
                };
                return HttpResponse::Ok().json(json!(
                    {
                        "status": "Success",
                        "message": "Successfully logged in",
                        "token":token,
                        "user_id":v.user_id
                    }
                ));
            } else {
                return HttpResponse::Unauthorized().json(json!(
                    {
                        "status": "Error",
                        "message":"Invalid Credentials"
                    }
                ));
            }
        }
        Err(e) => {
            return HttpResponse::Unauthorized().json(json!({
                "status": "Error",
                "message":"Invalid Credentials"
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use actix_web::{test, web, App};
    use serde_json::{json, Value};

    use crate::{
        api::users::{get_token, get_user_details, user_register},
        config::db::get_db,
        utilities::utils::JwtMiddleware,
        AppState,
    };

    use super::user_update;


    #[test]
    async fn test_user_register() {
        //get the pool connection
        let pool = match get_db().await {
            Ok(v) => v,
            Err(e) => panic!("Error at pool connection = {:?}", e),
        };

        let appdata = web::Data::new(AppState {
            db: Mutex::new(pool),
        });

        let app = test::init_service(
            App::new()
                .app_data(appdata)
                .route("/user/register_user", web::post().to(user_register)),
        )
        .await;

        let req_body = json!({
            "username":"test",
            "email":"test@test.com",
            "password":"test"
        });

        let req = test::TestRequest::post()
            .uri("/user/register_user")
            .set_json(&req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        println!("status of the user_register");
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        // let resp_body: Value = test::read_body_json(resp).await;
        // println!("status of the user_register response body");
        // assert_eq!(
        //     resp_body,
        //     json!(
        //         {
        //         "status": "Success",
        //         "message": "User Registration Successfully",
        //         "detailed_Message" : "N/A"
        //         }
        //     )
        // );
    }

    #[test]
    async fn test_get_token() {
        println!("Hello from the test_get_token");

        let pool = match get_db().await {
            Ok(v) => v,
            Err(e) => panic!("Error at pool connection: {}", e),
        };

        let appdata = web::Data::new(AppState {
            db: Mutex::new(pool),
        });

        let app = test::init_service(
            App::new()
                .app_data(appdata)
                .route("/user/get_token", web::get().to(get_token)),
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

        println!("Status of the get_token");
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let resp_body: Value = test::read_body_json(resp).await;

        let message = resp_body.get("message").unwrap();
        assert_eq!("Successfully logged in".to_string(), *message);
    }

    #[test]
    async fn test_update_user() {
        println!("Hello from the test_update_user");

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
                .route("/user/update_user", web::post().to(user_update)),
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
        println!("token = {}", token);
        let req_body = json!({
            "username":"test_updated"
        });

        let req = test::TestRequest::post()
            .uri("/user/update_user")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;

        let resp_body: Value = test::read_body_json(resp).await;

        assert_eq!(
            resp_body,
            json!(
                {
                    "status":"Success",
                    "message": "User Updated Successfully"
                }
            )
        );
    }

    #[test]
    async fn test_get_user() {
        println!("Hello from the test get user");

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
                .route("/user/get_user", web::get().to(get_user_details)),
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
            "email":"test@test.com"
        });

        let req = test::TestRequest::get()
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .uri("/user/get_user")
            .set_json(req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }
}
