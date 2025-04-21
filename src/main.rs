mod client;
mod config;

use reqwest::StatusCode;
use std::time::Duration;
use tokio::time::sleep;
use std::fs::OpenOptions;
use futures::future::join_all;

use client::iclock::services::IclockServices::{get_transactions, jwt_api_token_auth};
use client::time::services::TimeServices::{update_mark_status, fetch_company_config};
use client::time::models::TimeModels::MarkStatusRequest;
use config::config::Config;
use config::company_config::{CompanyConfiguration,Iclock};


#[tokio::main]
async fn main() {
    println!("Aplicación Iniciada");
    log_to_csv("INFO", &"Aplicación Iniciada".to_string());
    println!("Obteniendo variables de entorno");
    log_to_csv("INFO", &"Obteniendo variables de entorno".to_string());
    
    let env = Config::from_env();
    let id_company: String = env.id_company.clone();
    if id_company.is_empty() {
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

    println!("Variables de entorno: ID_COMPANY: {}, DOMAIN_TIME: {}, ICLOCK_CONFIG_PATH: {}", id_company, domain_time, iclock_config_path);
    log_to_csv("INFO", &format!("Variables de entorno: ID_COMPANY: {}, DOMAIN_TIME: {}, ICLOCK_CONFIG_PATH: {}", id_company, domain_time, iclock_config_path));
    
    let company: CompanyConfiguration = match fetch_company_config(&id_company).await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error al obtener configuración de empresa: {}", e);
            log_to_csv("ERROR", &format!("Error al obtener configuración de empresa: {}", e));
            return;
        }
    };

    if let Err(e) = CompanyConfiguration::save_to_file(&company, &iclock_config_path) {
        eprintln!("Error al guardar configuración: {}", e);
        log_to_csv("ERROR", &format!("Error al guardar configuración: {}", e));
        return;
    }

    println!("Escribiendo archivo iclock_configuration.json");
    log_to_csv("INFO", &"Escribiendo archivo iclock_configuration.json".to_string());
    let mut iclocks: Vec<Iclock> = vec![];
    match CompanyConfiguration::from_file(&iclock_config_path) {
        Ok(config) => {
            println!("Empresa: {}", config.razonSocial);
            log_to_csv("INFO", &format!("Contenido archivo: {:?}", config));
            iclocks = config.iclocks;
        }
        Err(e) => {
            eprintln!("Error al leer archivo JSON: {}", e);
            log_to_csv("ERROR", &format!("Error al leer archivo JSON: {}", e));
        }
    }

    if iclocks.is_empty() {
        println!("No se configuraron los parameters en DynamoDB");
        log_to_csv("ERROR", &"No se configuraron los parameters en DynamoDB".to_string());
        return;
    }

    let handles = iclocks.into_iter().map(|iclock| {
        tokio::spawn(async move {
            handle_port_loop(iclock.port, iclock.serialNumber).await;
        })
    });

    join_all(handles).await;
}


async fn handle_port_loop(port: u16, serial_number: String) {
    let mut jwt = None;
    let mut time_config: i64 = 1;

    loop {
        match get_transactions(jwt.clone(), port, serial_number.clone(), time_config).await {
            Ok((StatusCode::OK, Some(response))) => {
                for item in response.data {
                    let data = MarkStatusRequest {
                        ruc: "20575479820".to_string(),
                        statusMark: item.punch_state.parse::<u8>().unwrap_or(0),
                        idCompany: "b12564a0-95f7-11ea-9e4c-afcadc746e5b".to_string(),
                        identityNumber: item.emp_code,
                        address: "RC74+P52, 20002, Perú".to_string(),
                        latitude: -5.185049,
                        longitude: -80.5942013,
                        model: item.terminal_sn,
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
                    time_config = 1;
                }
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
                println!("Puerto {}: Error en la solicitud: {}", port, e);
                time_config = 10;
            }
        }

        sleep(Duration::from_secs(12)).await;
        log_to_csv("INFO", &format!("Puerto {}: Esperando 30s para la siguiente petición", port));
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