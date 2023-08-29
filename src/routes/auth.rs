use std::str::FromStr;

use axum::Json;
use hyper::StatusCode;

use crate::{
    auth::AuthContext,
    domain::{User, UserRole},
};

#[derive(Debug, serde::Deserialize)]
pub struct LoginInput {
    pub login: String,
    pub password: String,
}

pub(crate) async fn login_handler(
    mut auth: AuthContext,
    Json(input): Json<LoginInput>,
) -> StatusCode {
    let user = User {
        id: uuid::Uuid::from_str("9d2b1174-8f9f-4a12-b136-fb091b61843a").unwrap(),
        login: input.login,
        password_hash: input.password,
        role: UserRole::Admin,
    };
    auth.login(&user).await.unwrap();

    dbg!(&auth.current_user);

    StatusCode::OK
}
