mod client;
mod config;

use client::iclock;
use reqwest::StatusCode;
use std::time::Duration;
use tokio::time::sleep;
use std::fs::OpenOptions;
use futures::future::join_all;
use chrono::{Duration as ChronoDuration, Utc, NaiveDateTime};
use chrono_tz::America::Lima;

use client::iclock::services::IclockServices::{get_transactions, jwt_api_token_auth};
use client::time::services::TimeServices::{update_mark_status, fetch_companys_configs, fetch_company_config, update_conection_status};
use client::time::models::TimeModels::{MarkStatusRequest, ConectionStatusRequest};
use config::config::Config;
use config::company_config::{CompanyConfiguration,Iclock};


#[tokio::main]
async fn main() {
    println!("Aplicación Iniciada");
    log_to_csv("INFO", &"Aplicación Iniciada".to_string());
    println!("Obteniendo variables de entorno");
    log_to_csv("INFO", &"Obteniendo variables de entorno".to_string());
    
    let env = Config::from_env();
    let ids_companys: Vec<String> = env.ids_companys.clone();
    if ids_companys.len() < 1 {
        return;
    }
    
    if env.usser_biotime.is_empty() || env.password_biotime.is_empty() {
        println!("No se configuró usuario y/o contraseña");
        log_to_csv("ERROR", &"No se configuró usuario y/o contraseña".to_string());
        return;
    }
    
    if env.ip_server.is_empty(){
        println!("No se configuró ip_server");
        log_to_csv("ERROR", &"No se configuró ip_server".to_string());
        return;
    }
    let domain_time: String = env.domain_time.clone();
    let iclock_config_path: String = env.iclock_config_path.clone();

    if !check_internet_connection().await {
        println!("Error de conexión a internet");
        log_to_csv("ERROR", &"Error de conexión a internet".to_string());
        return;
    }
    
    println!("Variables de entorno: IDS_COMPANYS: {:?}, DOMAIN_TIME: {}, ICLOCK_CONFIG_PATH: {}", ids_companys, domain_time, iclock_config_path);
    log_to_csv("INFO", &format!("Variables de entorno: IDS_COMPANYS: {:?}, DOMAIN_TIME: {}, ICLOCK_CONFIG_PATH: {}", ids_companys, domain_time, iclock_config_path));
    
    let companies = match fetch_companys_configs(ids_companys).await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error al obtener configuraciones de empresa: {}", e);
            log_to_csv("ERROR", &format!("Error al obtener configuraciones de empresa: {}", e));
            return;
        }
    };
    
    if let Err(e) = CompanyConfiguration::save_to_file(&companies, &iclock_config_path) {
        eprintln!("Error al guardar configuración: {}", e);
        log_to_csv("ERROR", &format!("Error al guardar configuración: {}", e));
        return;
    }

    println!("Escribiendo archivo iclock_configuration.json");
    log_to_csv("INFO", &"Escribiendo archivo iclock_configuration.json".to_string());

    for company in companies {
        println!("Configuración para empresa: {:?}", company.razonSocial);
        log_to_csv("INFO", &format!("Configuración para empresa: {}", company.razonSocial));
        
        let mut ruc_company: String = "".to_string(); 
        let mut idcompany: String = "".to_string(); 
        let mut iclocks: Vec<Iclock> = vec![];
        idcompany = company.idCompany;
        iclocks = company.iclocks;
        ruc_company = company.ruc;

        if iclocks.is_empty() {
            println!("No se configuraron los parameters en DynamoDB");
            log_to_csv("ERROR", &"No se configuraron los parameters en DynamoDB".to_string());
            return;
        }

        let handles = iclocks
        .into_iter()
        .filter(|iclock| iclock.status == true)
        .map(|iclock| {
            let id_company: String = idcompany.clone();
            let ruc_company: String = ruc_company.clone();

            tokio::spawn(async move {
                handle_port_loop(iclock.port, iclock.serialNumber, id_company, ruc_company).await;
            })
        });

        join_all(handles).await;
    }

}


async fn handle_port_loop(port: u16, serial_number: String, id_company: String, ruc_company: String) {
    let mut jwt: Option<String> = None;
    let mut time_config: i64 = 1;

    loop {
        let now_lima = Utc::now().with_timezone(&Lima);
        let time_str = now_lima.format("%Y-%m-%d %H:%M:%S").to_string();
        let company: CompanyConfiguration = match fetch_company_config(&id_company).await {
            Ok(data) => data,
            Err(e) => {
                println!("Error al obtener configuración de empresa: {}", e);
                log_to_csv("ERROR", &format!("Error al obtener configuración de empresa: {}", e));
                continue;
            }
        };

        let mut interval_config: u64 = company.timeConfig;
        let iclocks: Vec<Iclock> = company.iclocks;
        let mut lastConnectionTime: Option<String> = Some("".to_string());
        for iclock in iclocks {
            if iclock.serialNumber == serial_number {
                lastConnectionTime = iclock.lastConnectionTime;
            }
        }
        
        if let Some(ref last_str) = lastConnectionTime {
            if !last_str.is_empty() {
                let fmt = "%Y-%m-%d %H:%M:%S";
                let now = NaiveDateTime::parse_from_str(&time_str, fmt).unwrap();
                let last = NaiveDateTime::parse_from_str(last_str, fmt).unwrap();
        
                let diff: chrono::TimeDelta = now - last;
                let minutes = diff.num_minutes();
                if minutes > ((interval_config/60) + 1).try_into().unwrap()  {
                    time_config = minutes;
                }
            }
        }

        match get_transactions(jwt.clone(), port, serial_number.clone(), time_config).await {
            Ok((StatusCode::OK, Some(response))) => {
                for item in response.data {    
                    let terminal_sn: String = item.terminal_sn;
                    let data = MarkStatusRequest {
                        ruc: ruc_company.clone(),
                        statusMark: item.punch_state.parse::<u8>().unwrap_or(0),
                        idCompany: id_company.clone(),
                        identityNumber: item.emp_code,
                        address: "".to_string(),
                        latitude: 0.0,
                        longitude: 0.0,
                        model: terminal_sn.clone(),
                        timestamp: item.punch_time,
                    };

                    log_to_csv("INFO", &format!("Puerto {}: {:?}", port, data));
                    
                    match update_mark_status(data).await {
                        Ok((StatusCode::OK, Some(response))) => {
                            println!("Puerto {}: Código: {:?}, Mensaje: {:?}", port, response.codigoRespuesta, response.mensajeRespuesta);
                        }
                        Ok((status, _)) => {
                            println!("Puerto {}: Respuesta inesperada: {}", port, status);
                        }
                        Err(e) => {
                            println!("Puerto {}: Error al actualizar estado: {}", port, e);
                        }
                    }
                }
                
                let data_conection = ConectionStatusRequest {
                    ruc: ruc_company.clone(),
                    idCompany: id_company.clone(),
                    messageError: "".to_string(),
                    serialNumber: serial_number.clone(),
                    connectionStatus: true,
                    lastConnectionTime: time_str
                };

                match update_conection_status(data_conection).await {
                    Ok((StatusCode::OK, Some(response))) => {
                        println!("Código: {:?}, Mensaje: {:?}", response.codigoRespuesta, response.mensajeRespuesta);
                    }
                    Ok((status, _)) => {
                        println!("Respuesta inesperada: {}", status);
                    }
                    Err(e) => {
                        println!("Error al actualizar estado: {}", e);
                    }
                }
                time_config = 1;
            }
            Ok((StatusCode::UNAUTHORIZED, _)) => {
                log_to_csv("INFO", &format!("Puerto {}: Token expirado, renovando JWT", port));
                match jwt_api_token_auth(port).await {
                    Ok((StatusCode::OK, Some(response))) => {
                        jwt = response.token;
                    }
                    Err(e) => {
                        println!("Puerto {}: Error obteniendo nuevo JWT: {}", port, e);
                    }
                    _ => {}
                }
            }
            Ok((status, _)) => {
                println!("Puerto {}: Código inesperado: {}", port, status);
            }
            Err(e) => {
                let data_conection = ConectionStatusRequest {
                    ruc: ruc_company.clone(),
                    idCompany: id_company.clone(),
                    messageError: e.to_string(),
                    serialNumber: serial_number.clone(),
                    connectionStatus: false,
                    lastConnectionTime: "".to_string(),
                };

                match update_conection_status(data_conection).await {
                    Ok((StatusCode::OK, Some(response))) => {
                        println!("Código: {:?}, Mensaje: {:?}", response.codigoRespuesta, response.mensajeRespuesta);
                    }
                    Ok((status, _)) => {
                        println!("Respuesta inesperada: {}", status);
                    }
                    Err(e) => {
                        println!("Error al actualizar estado: {}", e);
                    }
                }
                println!("Puerto {}: Error en la solicitud: {}", port, e);
                time_config = 2;
            }
        }

        sleep(Duration::from_secs(interval_config)).await;
        log_to_csv("INFO", &format!("Puerto {}: Esperando {}s para la siguiente petición", port, interval_config));
    }
}


fn log_to_csv(level: &str, message: &String) {
    let now = chrono::Utc::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let filename = format!("logs-{}.csv", date_str);

    if let Ok(file) = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)
    {
        let mut writer = csv::Writer::from_writer(file);
        let timestamp = now.to_rfc3339();

        if writer
            .write_record(&[timestamp, level.to_string(), message.to_string()])
            .is_ok()
        {
            let _ = writer.flush();
        }
    } else {
        eprintln!("No se pudo abrir el archivo de logs.");
    }
}

async fn check_internet_connection() -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();

    match client.get("https://www.google.com").send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}