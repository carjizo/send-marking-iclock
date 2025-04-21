use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct TransactionResponse {
    pub count: u32,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub msg: String,
    pub code: i32,
    pub data: Vec<DataItem>,
}

#[derive(Debug, Deserialize)]
pub struct DataItem {
    pub id: u32,
    pub emp: Option<u32>,
    pub emp_code: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub department: Option<String>,
    pub position: Option<String>,
    pub punch_time: String,
    pub punch_state: String,
    pub punch_state_display: String,
    pub verify_type: u8,
    pub verify_type_display: Option<String>,
    pub work_code: Option<String>,
    pub gps_location: Option<String>,
    pub area_alias: Option<String>,
    pub terminal_sn: String,
    pub temperature: Option<f32>,
    pub is_mask: Option<String>,
    pub terminal_alias: Option<String>,
    pub upload_time: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenAuthResponse {
    pub token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TokenAuthRequest {
    pub username: String,
    pub password: String
}