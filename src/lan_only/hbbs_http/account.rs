use serde_derive::Serialize;

const NOT_SUPPORTED: &str = "Account auth is not supported in lan-only builds";

/// Never constructed in lan-only builds: `AuthResult::auth_body` is always
/// `None`, but the type has to keep the name the Flutter bridge deserializes.
#[derive(Serialize, Default)]
pub struct AuthBody {}

#[derive(Serialize)]
pub struct AuthResult {
    pub state_msg: String,
    pub failed_msg: String,
    pub url: Option<String>,
    pub auth_body: Option<AuthBody>,
}

pub struct OidcSession;

impl OidcSession {
    pub fn account_auth(
        _api_server: String,
        _op: String,
        _id: String,
        _uuid: String,
        _remember_me: bool,
    ) {
    }

    pub fn auth_cancel() {}

    pub fn get_result() -> AuthResult {
        AuthResult {
            state_msg: NOT_SUPPORTED.to_owned(),
            failed_msg: NOT_SUPPORTED.to_owned(),
            url: None,
            auth_body: None,
        }
    }
}
