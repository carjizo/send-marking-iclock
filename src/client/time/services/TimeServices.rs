use crate::client::time::models::TimeModels::{MarkStatusRequest, MarkStatusResponse, CompanyConfigResponse, ConectionStatusRequest, ConectionStatusResponse};
use reqwest::{header::{HeaderMap, HeaderValue, CONTENT_TYPE}};
use crate::StatusCode;
use crate::{CompanyConfiguration, Config};
use crate::log_to_csv;

pub async fn update_mark_status(request_data: MarkStatusRequest) -> Result<(StatusCode, Option<MarkStatusResponse>), Box<dyn std::error::Error>> {

    let env = Config::from_env();

    // let url: &str =  "https://api-time-qa.smartboleta.com/qa//time/iclock/flow-markings/update-mark-status";
    let url =  env.domain_time + "/time/iclock/flow-markings/update-mark-status";

    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", HeaderValue::from_str(&env.api_key)?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let client = reqwest::Client::new();

    log_to_csv("INFO", &format!("url: {:?}", url));

    let res = client
        .put(url)
        .headers(headers)
        .json(&request_data)
        .send()
        .await?;

    let status = res.status();

    if status == StatusCode::OK {
        let json = res.json::<MarkStatusResponse>().await?;
        Ok((status, Some(json)))
    } else {
        Ok((status, None))
    }
}


pub async fn fetch_company_config(id_company: &str) -> Result<CompanyConfiguration, Box<dyn std::error::Error>> {
    let env = Config::from_env();

    let url = format!("{}/time/iclock/configurations/{}", env.domain_time, id_company);
    
    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", HeaderValue::from_str(&env.api_key)?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .headers(headers)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(format!("Request failed with status: {}", res.status()).into());
    }

    let parsed: CompanyConfigResponse = res.json().await?;
    Ok(parsed.response)
}

pub async fn update_conection_status(request_data: ConectionStatusRequest) -> Result<(StatusCode, Option<ConectionStatusResponse>), Box<dyn std::error::Error>> {

    let env = Config::from_env();

    let url =  env.domain_time + "/time/iclock/flow-markings/update-conection-status";

    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", HeaderValue::from_str(&env.api_key)?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let client = reqwest::Client::new();

    log_to_csv("INFO", &format!("url: {:?}", url));

    let res = client
        .put(url)
        .headers(headers)
        .json(&request_data)
        .send()
        .await?;

    let status = res.status();

    if status == StatusCode::OK {
        let json = res.json::<ConectionStatusResponse>().await?;
        Ok((status, Some(json)))
    } else {
        Ok((status, None))
    }
}