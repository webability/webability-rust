//! Módulo DNS: zonas y registros del cliente. Envuelve /v1/dns/*.

use serde::{Deserialize, Serialize};

use crate::{WaApi, WaError};

/// Zona DNS del cliente.
#[derive(Debug, Clone, Deserialize)]
pub struct Zone {
    pub key: i64,
    pub name: String,
    pub status: i64,
    pub primaryns: String,
    pub adminemail: String,
    pub serial: i64,
    pub refresh: i64,
    pub retry: i64,
    pub expire: i64,
    pub minimum: i64,
    pub defaultttl: i64,
    pub dnssec: i64,
    #[serde(default)]
    pub creationdate: String,
}

/// Registro DNS de una zona del cliente.
#[derive(Debug, Clone, Deserialize)]
pub struct Record {
    pub key: i64,
    pub zone: i64,
    pub name: String,
    pub rrtype: i64,
    pub rrtypename: String,
    pub ttl: i64,
    pub status: i64,
    pub priority: i64,
    pub weight: i64,
    pub port: i64,
    #[serde(default)]
    pub tag: String,
    pub data: String,
}

/// Respuesta de list_zones.
#[derive(Debug, Deserialize)]
pub struct ListZonesResult {
    pub status: String,
    pub zones: Vec<Zone>,
    pub count: i64,
}

/// Respuesta de get_zone.
#[derive(Debug, Deserialize)]
pub struct GetZoneResult {
    pub status: String,
    pub zone: Zone,
    pub records: Vec<Record>,
    pub ns: Vec<String>,
}

/// Respuesta de add_zone.
#[derive(Debug, Deserialize)]
pub struct AddZoneResult {
    pub status: String,
    pub key: i64,
    pub name: String,
}

/// Campos para crear un registro nuevo con add_record.
#[derive(Debug, Default, Serialize)]
pub struct RecordInput {
    pub name: String,
    pub rrtype: String,
    pub ttl: i64,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

/// Respuesta de add_record.
#[derive(Debug, Deserialize)]
pub struct AddRecordResult {
    pub status: String,
    pub key: i64,
    pub zone: i64,
}

/// Campos opcionales para modificar un registro existente con update_record.
/// Solo los campos `Some(..)` se envían y se modifican.
#[derive(Debug, Default, Serialize)]
pub struct RecordUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i64>,
}

/// Respuesta de update_record / delete_record.
#[derive(Debug, Deserialize)]
pub struct KeyResult {
    pub status: String,
    pub key: i64,
}

/// Respuesta de delete_zone.
#[derive(Debug, Deserialize)]
pub struct DeleteZoneResult {
    pub status: String,
    pub key: i64,
    pub name: String,
}

/// Enlaza un WaApi para hacer las llamadas al servicio DNS de la API.
pub struct Dns<'a> {
    api: &'a WaApi,
}

impl<'a> Dns<'a> {
    pub fn new(api: &'a WaApi) -> Self {
        Self { api }
    }

    /// Lista las zonas (dominios) del cliente. GET /v1/dns/zone
    pub fn list_zones(&self) -> Result<ListZonesResult, WaError> {
        self.api.get("/v1/dns/zone")?.decode()
    }

    /// Obtiene una zona (por clave numérica o por nombre de dominio) junto
    /// con sus registros. GET /v1/dns/zone/{key|domain}
    pub fn get_zone(&self, key_or_domain: &str) -> Result<GetZoneResult, WaError> {
        let path = format!("/v1/dns/zone/{}", urlencode(key_or_domain));
        self.api.get(&path)?.decode()
    }

    /// Crea una nueva zona. POST /v1/dns/zone
    pub fn add_zone(&self, name: &str) -> Result<AddZoneResult, WaError> {
        let body = serde_json::json!({ "name": name });
        self.api.post("/v1/dns/zone", &body)?.decode()
    }

    /// Agrega un registro a una zona. POST /v1/dns/zone/{key}/record
    pub fn add_record(&self, zone_key: i64, record: &RecordInput) -> Result<AddRecordResult, WaError> {
        let path = format!("/v1/dns/zone/{zone_key}/record");
        self.api.post(&path, record)?.decode()
    }

    /// Modifica un registro existente. PUT /v1/dns/record/{key}
    pub fn update_record(&self, record_key: i64, fields: &RecordUpdate) -> Result<KeyResult, WaError> {
        let path = format!("/v1/dns/record/{record_key}");
        self.api.put(&path, fields)?.decode()
    }

    /// Elimina un registro. DELETE /v1/dns/record/{key}
    pub fn delete_record(&self, record_key: i64) -> Result<KeyResult, WaError> {
        let path = format!("/v1/dns/record/{record_key}");
        self.api.delete(&path)?.decode()
    }

    /// Elimina una zona y todos sus registros. DELETE /v1/dns/zone/{key}
    pub fn delete_zone(&self, zone_key: i64) -> Result<DeleteZoneResult, WaError> {
        let path = format!("/v1/dns/zone/{zone_key}");
        self.api.delete(&path)?.decode()
    }
}

/// Escapa un segmento de path (percent-encoding mínimo, suficiente para
/// dominios y claves numéricas: no requiere una dependencia externa de URL).
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
