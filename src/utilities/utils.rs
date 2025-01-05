use actix_web::{
    body::BoxBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web::Data,
    Error, HttpMessage, HttpRequest, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use std::future::{ready, Ready};

use crate::{
    models::users::get_user,
    utilities::{auth::decode_jwt, errors::api_error},
    AppState,
};

pub struct JwtMiddleware;

impl<S> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddleWareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddleWareService { service }))
    }
}

pub struct JwtMiddleWareService<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for JwtMiddleWareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        println!(
            "Hello from the JwtMiddleWareService. You requested: {}",
            req.path()
        );
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok());

        if let Some(auth_value) = auth_header {
            if auth_value.starts_with("Bearer ") {
                let token = auth_value.trim_start_matches("Bearer ");
                if let Ok(tok) = decode_jwt(token.to_string()) {
                    req.extensions_mut().insert(tok.sub as Uuid);
                    let fut = self.service.call(req);
                    return Box::pin(async move { fut.await });
                }
            }
        } else if req.path() == "/user/register_user" || req.path() == "/user/get_token" {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }
        let response = HttpResponse::Unauthorized().json(json!(
            {
                "status": "Error",
        "message": "Invalid or missing JWT token",
        "detailed_Message" : "Connection Timeout. JWT is invalid"
            }
        ));
        Box::pin(async move {
            let response = req.into_response(response);
            Ok(response)
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct ValidateUserError {
    pub status: String,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct ValidateUserDetails {
    pub uid: i32,
    pub email: String,
}

pub async fn validate_user(
    pool: &Pool<Postgres>,
    req: HttpRequest,
    email: String,
) -> Result<ValidateUserDetails, ValidateUserError> {
    println!("Hello from the vslidate user");
    let uid = if let Some(id) = req.extensions().get::<i32>() {
        id.clone()
    } else {
        return Err(ValidateUserError {
            message: String::from("user id not found"),
            status: String::from("Error"),
        });
    };
    println!("uid = {:?}", uid);
    match get_user(pool, email).await {
        Ok(v) => {
            println!("{:?}", v);
            println!("{}", uid);
            if v.id != uid {
                return Err(ValidateUserError {
                    message: String::from("Not Authorized"),
                    status: String::from("Error"),
                });
            }
            Ok(ValidateUserDetails {
                email: v.email,
                uid: v.id,
            })
        }
        Err(e) => {
            let mut message = "Invalid user details";
            if let sqlx::Error::Database(db_err) = &e {
                println!("Db-error = {:?}", db_err.message());
                message = db_err.message();
            }
            Err(ValidateUserError {
                message: message.to_owned(),
                status: String::from("Error"),
            })
        }
    }
}
