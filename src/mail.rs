//! 🚧 Stub — calca el contrato del SDK de Go (github.com/webability/webability-go/mail).
//!
//! La capa de transporte (WaApi::get/post/put/delete) ya está implementada;
//! falta conectar send()/status() a ella (ver dns.rs para el patrón a seguir).

use std::collections::HashMap;

use crate::{WaApi, WaError};

pub struct Address {
    pub email: String,
    pub name: String,
}

pub struct Recipient {
    pub email: String,
    pub name: String,
    pub vars: HashMap<String, String>,
}

/// Campos para POST /v1/mail/send.
pub struct SendRequest {
    pub from: Address,
    pub to: Recipient,
    /// Si viene (no vacío), es el id de una plantilla ya registrada y activa
    /// en templates_template bajo la cuenta que autentica el request — el
    /// servidor arma el correo con esa plantilla en vez de subject/html/text
    /// (que se ignoran si template viene). La personalización usa las vars
    /// de `to`, sin ningún prefijo en los nombres — dentro del contenido de
    /// la plantilla (Consola → Correos → Plantillas) se acceden exactamente
    /// igual que en el envío ad-hoc: {{clave}} directo. La plantilla solo ve
    /// las vars, nunca el resto del mensaje (to, from, subject, etc.) — si
    /// necesitas imprimir alguno de esos datos dentro del cuerpo, agrégalo
    /// también a vars. El servidor valida que la plantilla exista y esté
    /// activa ANTES de encolar el correo: si no, send() devuelve Err con el
    /// error de la API (códigos 3025/3026), no un envío "pending" fallido.
    pub template: String,
    pub subject: String,
    pub html: String,
    pub text: String,
    pub tags: Vec<String>,
    pub track_opens: bool,
    pub track_clicks: bool,
    /// Si es true, el servidor espera (hasta ~20s) el resultado real del
    /// envío antes de responder, en vez de responder de inmediato con
    /// queue_status="pending". Ver Mail::send().
    pub wait_send: bool,
}

/// Estados posibles de queue_status en SendResult y StatusResult.
pub mod queue_status {
    pub const PENDING: &str = "pending";
    pub const PROCESSING: &str = "processing";
    pub const SENT: &str = "sent";
    pub const ERROR: &str = "error";
}

/// Respuesta de Mail::send().
pub struct SendResult {
    pub status: String,
    pub queue_key: i64,
    pub queue_status: String,
    pub error_detail: String,
    pub to: String,
}

/// Respuesta de Mail::status().
pub struct StatusResult {
    pub status: String,
    pub queue_key: i64,
    pub queue_status: String,
    pub error_detail: String,
}

pub struct Mail<'a> {
    api: &'a WaApi,
}

impl<'a> Mail<'a> {
    pub fn new(api: &'a WaApi) -> Self {
        Self { api }
    }

    /// Envía un correo a un solo destinatario. POST /v1/mail/send
    ///
    /// 🚧 Pendiente de implementar (ver mail.go para el contrato de referencia).
    pub fn send(&self, _req: SendRequest) -> Result<SendResult, WaError> {
        let _ = &self.api;
        Err(WaError::Decode("Mail::send() aún no está implementado en el SDK de Rust".to_string()))
    }

    /// Consulta el estatus real de un envío hecho con send().
    /// GET /v1/mail/status/{queue_key}
    ///
    /// 🚧 Pendiente de implementar (ver mail.go para el contrato de referencia).
    pub fn status(&self, _queue_key: i64) -> Result<StatusResult, WaError> {
        let _ = &self.api;
        Err(WaError::Decode("Mail::status() aún no está implementado en el SDK de Rust".to_string()))
    }
}
