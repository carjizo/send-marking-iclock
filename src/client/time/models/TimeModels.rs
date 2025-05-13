use serde::{Deserialize, Serialize};
use crate::CompanyConfiguration;

#[derive(Serialize, Debug)]
pub struct MarkStatusRequest {
    pub ruc: String,
    pub statusMark: u8,
    pub idCompany: String,
    pub identityNumber: String,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub model: String,
    pub timestamp: String,
}

#[derive(Deserialize, Debug)]
pub struct MarkStatusResponse {
    pub codigoRespuesta: Option<String>,
    pub mensajeRespuesta: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CompanyConfigResponse {
    pub response: CompanyConfiguration,
}

#[derive(Deserialize, Debug)]
pub struct AllCompanysConfigResponse {
    pub response: Vec<CompanyConfiguration>,
}

#[derive(Serialize, Debug)]
pub struct ConectionStatusRequest {
    pub ruc: String,
    pub idCompany: String,
    pub serialNumber: String,
    pub messageError: String,
    pub connectionStatus: bool,
    pub lastConnectionTime: String,
}

#[derive(Deserialize, Debug)]
pub struct ConectionStatusResponse {
    pub codigoRespuesta: Option<String>,
    pub mensajeRespuesta: Option<String>,
}