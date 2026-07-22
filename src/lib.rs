//! Cliente base para la API de WebAbility (https://api.webability.info).
//!
//! Firma cada request con HMAC-SHA256 (headers X-WA-Client, X-WA-Timestamp,
//! X-WA-Digest) — mismo esquema que el SDK de Go
//! (github.com/webability/webability-go/wa). El Token nunca viaja en el
//! request: solo se usa localmente para calcular el digest.
//!
//! Dependencias (ver Cargo.toml): `ureq` (HTTP síncrono), `hmac` + `sha2`
//! (firma), `hex` (codificación), `serde` + `serde_json` (JSON).

pub mod dns;
pub mod mail;

use std::fmt;

use hmac::{Hmac, Mac};
use serde::de::DeserializeOwned;
use serde::Serialize;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Host por defecto de la API de WebAbility.
pub const DEFAULT_BASE_URL: &str = "https://api.webability.info";

/// Construye el mensaje canónico a firmar: "{METODO}|{PATH}|{TIMESTAMP}|{CLIENTID}".
/// path debe ser la ruta del request sin query string.
pub fn build_message(method: &str, path: &str, timestamp: &str, client_id: &str) -> String {
    format!("{method}|{path}|{timestamp}|{client_id}")
}

/// Error de la librería: de la API (formato {status, code, message}), de
/// transporte (red) o de (de)serialización.
#[derive(Debug)]
pub enum WaError {
    Api {
        status_code: u16,
        code: Option<i64>,
        message: String,
    },
    Transport(String),
    Decode(String),
}

impl fmt::Display for WaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WaError::Api { code, message, .. } => write!(f, "wa api error {code:?}: {message}"),
            WaError::Transport(msg) => write!(f, "enviando request: {msg}"),
            WaError::Decode(msg) => write!(f, "decodificando respuesta: {msg}"),
        }
    }
}

impl std::error::Error for WaError {}

#[derive(Debug, serde::Deserialize)]
struct ErrorBody {
    #[serde(default)]
    code: Option<i64>,
    #[serde(default)]
    message: Option<String>,
}

/// Respuesta cruda de un request a la API.
pub struct Response {
    pub status_code: u16,
    pub body: Vec<u8>,
}

impl Response {
    /// Decodifica el cuerpo JSON de la respuesta.
    pub fn decode<T: DeserializeOwned>(&self) -> Result<T, WaError> {
        serde_json::from_slice(&self.body).map_err(|e| WaError::Decode(e.to_string()))
    }
}

pub struct WaApi {
    pub client_id: String,
    pub token: String,
    pub base_url: String,
}

impl WaApi {
    /// Crea un objeto WaApi con el ClientID y el Token de la cuenta, usando
    /// el host por defecto.
    pub fn new(client_id: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            token: token.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    /// Crea un objeto WaApi con un host alternativo (útil para pruebas).
    pub fn new_with_url(base_url: impl Into<String>, client_id: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            token: token.into(),
            base_url: base_url.into(),
        }
    }

    /// Retorna hex(HMAC-SHA256(self.token, message)).
    pub fn digest(&self, message: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.token.as_bytes())
            .expect("HMAC-SHA256 acepta claves de cualquier tamaño");
        mac.update(message.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    fn do_request(&self, method: &str, path: &str, body: Option<String>) -> Result<Response, WaError> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();
        let message = build_message(method, path, &timestamp, &self.client_id);
        let digest = self.digest(&message);

        let url = format!("{}{}", self.base_url, path);
        let mut req = ureq::request(method, &url)
            .set("X-WA-Client", &self.client_id)
            .set("X-WA-Timestamp", &timestamp)
            .set("X-WA-Digest", &digest);

        let result = if let Some(payload) = body {
            req = req.set("Content-Type", "application/json");
            req.send_string(&payload)
        } else {
            req.call()
        };

        match result {
            Ok(response) => {
                let status_code = response.status();
                let text = response
                    .into_string()
                    .map_err(|e| WaError::Transport(e.to_string()))?;
                Ok(Response {
                    status_code,
                    body: text.into_bytes(),
                })
            }
            Err(ureq::Error::Status(status_code, response)) => {
                let text = response.into_string().unwrap_or_default();
                let buf = text.into_bytes();
                if let Ok(parsed) = serde_json::from_slice::<ErrorBody>(&buf) {
                    if let Some(message) = parsed.message {
                        return Err(WaError::Api {
                            status_code,
                            code: parsed.code,
                            message,
                        });
                    }
                }
                Err(WaError::Transport(format!("error HTTP {status_code}")))
            }
            Err(ureq::Error::Transport(t)) => Err(WaError::Transport(t.to_string())),
        }
    }

    /// Envía un GET a path.
    pub fn get(&self, path: &str) -> Result<Response, WaError> {
        self.do_request("GET", path, None)
    }

    /// Envía un POST a path con body codificado en JSON.
    pub fn post<T: Serialize>(&self, path: &str, body: &T) -> Result<Response, WaError> {
        let payload = serde_json::to_string(body).map_err(|e| WaError::Decode(e.to_string()))?;
        self.do_request("POST", path, Some(payload))
    }

    /// Envía un PUT a path con body codificado en JSON.
    pub fn put<T: Serialize>(&self, path: &str, body: &T) -> Result<Response, WaError> {
        let payload = serde_json::to_string(body).map_err(|e| WaError::Decode(e.to_string()))?;
        self.do_request("PUT", path, Some(payload))
    }

    /// Envía un DELETE a path.
    pub fn delete(&self, path: &str) -> Result<Response, WaError> {
        self.do_request("DELETE", path, None)
    }
}
