use reqwest::{header::{HeaderMap, HeaderValue, CONTENT_TYPE}};
use std::collections::HashMap;
use crate::client::iclock::models::IClockModels::{TransactionResponse, TokenAuthResponse, TokenAuthRequest};
use crate::StatusCode;
use crate::log_to_csv;
use crate::Config;
use crate::{Utc,ChronoDuration, Lima};

pub async fn get_transactions(jwt: Option<String>, port: u16, serial_number: String, time_config: i64) -> Result<(StatusCode, Option<TransactionResponse>), reqwest::Error> {
    let env = Config::from_env();
    let ip_server: String = env.ip_server.clone(); 
    
    let url = format!("http://{}:{}/iclock/api/transactions/", ip_server,port);

    let now_lima = Utc::now().with_timezone(&Lima);
    let start_time = now_lima - ChronoDuration::minutes(time_config);
    let end_time = now_lima;
    let start_time_str = start_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let end_time_str = end_time.format("%Y-%m-%d %H:%M:%S").to_string();

    let token_auth = jwt.map(|t| format!("JWT {}", t));
    let mut headers = HeaderMap::new();
    if let Some(token) = token_auth {
        if let Ok(header_value) = HeaderValue::from_str(&token) {
            headers.insert("Authorization", header_value);
        }
    }
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("start_time", start_time_str);
    params.insert("end_time", end_time_str);
    params.insert("page_size", "200".to_string());
    params.insert("terminal_sn", serial_number);

    log_to_csv("INFO", &format!("url: {}, params: {:?}", url, params));
    let res = client
        .get(url)
        .headers(headers)
        .query(&params)
        .send()
        .await?;

    let status = res.status();
    
    if status == StatusCode::OK {
        let json = res.json::<TransactionResponse>().await?;
        Ok((status, Some(json)))
    } else {
        // También podrías leer el body con `.text().await?` para más detalles
        Ok((status, None))
    }
}


pub async fn jwt_api_token_auth(port: u16) -> Result<(StatusCode, Option<TokenAuthResponse>), reqwest::Error> {

    let env = Config::from_env();
    let usser: String = env.usser_biotime.to_string();
    let pass: String = env.password_biotime.to_string(); 
    let ip_server: String = env.ip_server.clone(); 
    let url: String = format!("http://{}:{}/jwt-api-token-auth/", ip_server, port);
    println!("user: {} , pass: {}", usser, pass);
    let request_data = TokenAuthRequest {username: usser, password: pass};
    println!("request_data {:?}", request_data);

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let client = reqwest::Client::new();
    
    log_to_csv("INFO", &format!("url: {}", url));

    let res = client
        .post(url)
        .headers(headers)
        .json(&request_data)
        .send()
        .await?;
    println!("res {:?}", res);
    let status = res.status();
    println!("status {}", status);
    if status == StatusCode::OK {
        let json: TokenAuthResponse = res.json::<TokenAuthResponse>().await?;
        Ok((status, Some(json)))
    } else {
        // También podrías leer el body con `.text().await?` para más detalles
        Ok((status, None))
    }
}