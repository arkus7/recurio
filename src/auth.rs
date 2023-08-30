use axum_login::axum_sessions::async_session::MemoryStore;
use axum_login::PostgresStore;

use axum_login::{
    axum_sessions::SessionLayer as AxumSessionLayer, AuthLayer as AxumAuthLayer,
    RequireAuthorizationLayer,
};
use secrecy::ExposeSecret;
use sqlx::PgPool;

use crate::configuration::AuthSettings;
use crate::domain::{User, UserId, UserRole};
use crate::session_store::PostgresSessionStore;

pub(crate) type AuthContext =
    axum_login::extractors::AuthContext<UserId, User, AuthUserStore, UserRole>;
pub(crate) type SessionStore = PostgresSessionStore;
pub(crate) type RequireAuth = RequireAuthorizationLayer<UserId, User, UserRole>;
pub(crate) type AuthUserStore = PostgresStore<User, UserRole>;
pub(crate) type AuthLayer = AxumAuthLayer<AuthUserStore, UserId, User, UserRole>;
pub(crate) type SessionLayer = AxumSessionLayer<SessionStore>;

pub(crate) async fn setup_auth(
    db_pool: PgPool,
    configuration: &AuthSettings,
) -> (AuthLayer, SessionLayer) {
    let user_store = PostgresStore::new(db_pool.clone());
    let session_store = PostgresSessionStore::new(db_pool.clone(), "sessions");

    session_store
        .migrate()
        .await
        .expect("Failed to migrate session store database");

    let secret = configuration
        .session_secret
        .expose_secret()
        .as_str()
        .as_bytes();
    let auth_layer = AuthLayer::new(user_store, secret);

    let session_layer = SessionLayer::new(session_store, secret).with_cookie_name("recurio.sid");

    (auth_layer, session_layer)
}

// #[async_trait]
// impl SessionStore for DatabaseUserStore {
//     /// Get a session from the storage backend.
//     ///
//     /// The input is expected to be the value of an identifying
//     /// cookie. This will then be parsed by the session middleware
//     /// into a session if possible
//     async fn load_session(
//         &self,
//         cookie_value: String,
//     ) -> Result<Option<Session>, axum_login::axum_sessions::async_session::Error> {
//         debug!("load_session, cookie value: {}", cookie_value);
//         dbg!("session_store", &self.session_store);
//         let id = Session::id_from_cookie_value(&cookie_value)?;
//         debug!("cookie ID: {id}");
//         Ok(self
//             .session_store
//             .read()
//             .await
//             .get(&id)
//             .cloned()
//             .map(|s| {
//                 debug!("before validation, session: {:?}", s);
//                 s
//             })
//             .and_then(Session::validate)
//             .map(|s| {
//                 debug!("after validation, session: {:?}", s);
//                 s
//             }))
//     }
//
//     /// Store a session on the storage backend.
//     ///
//     /// The return value is the value of the cookie to store for the
//     /// user that represents this session
//     async fn store_session(
//         &self,
//         session: Session,
//     ) -> Result<Option<String>, axum_login::axum_sessions::async_session::Error> {
//         debug!("store_session, session value: {:?}", session);
//         dbg!("session_store before store", &self.session_store);
//
//         self.session_store
//             .write()
//             .await
//             .insert(session.id().to_string(), session.clone());
//
//         dbg!("session_store after store", &self.session_store);
//         session.reset_data_changed();
//         Ok(session.into_cookie_value())
//     }
//
//     /// Remove a session from the session store
//     async fn destroy_session(
//         &self,
//         session: Session,
//     ) -> Result<(), axum_login::axum_sessions::async_session::Error> {
//         debug!("destroy_session");
//         dbg!("session to destroy", &session);
//         self.session_store.write().await.remove(session.id());
//         Ok(())
//     }
//
//     /// Empties the entire store, destroying all sessions
//     async fn clear_store(&self) -> Result<(), axum_login::axum_sessions::async_session::Error> {
//         debug!("clear_store");
//         Ok(())
//     }
// }
