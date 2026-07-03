//! 🚧 En construcción.
//!
//! Cliente base para la API de WebAbility (https://api.webability.info).
//! Seguirá el mismo esquema de autenticación que el cliente Go de referencia
//! (github.com/webability/webability-go): ClientID + Token, firma HMAC-SHA256 en los
//! headers X-WA-Client / X-WA-Timestamp / X-WA-Digest. El Token nunca viaja
//! en el request.

pub struct WaApi {
    pub client_id: String,
    pub token: String,
    pub base_url: String,
}

impl WaApi {
    pub fn new(client_id: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            token: token.into(),
            base_url: "https://api.webability.info".to_string(),
        }
    }
}
