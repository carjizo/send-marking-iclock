use std::env;
use aes::Aes128;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use base64::decode;

type Aes128Cbc = Cbc<Aes128, Pkcs7>;

pub struct Config {
    pub id_company: String,
    pub api_key: String,
    pub iclock_config_path: String,
    pub domain_time: String,
    pub ip_server: String,
    pub usser_biotime: String,
    pub password_biotime: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();

        let id_company: String = env::var("ID_COMPANY").unwrap_or_else(|_| "".to_string());
        let iclock_config_path: String = env::var("ICLOCK_CONFIG_PATH").unwrap_or_else(|_| "".to_string());
        let domain_time: String = env::var("DOMAIN_TIME").unwrap_or_else(|_| "".to_string());
        let ip_server: String = env::var("IP_SERVER").unwrap_or_else(|_| "".to_string());
        let usser_biotime: String = env::var("USSER_BIOTIME").unwrap_or_else(|_| "".to_string());
        let password_biotime: String = env::var("PASSWORD_BIOTIME").unwrap_or_else(|_| "".to_string());
        
        let api_key = match (
            env::var("AES_KEY"),
            env::var("AES_IV"),
            env::var("API_KEY_ENC"),
        ) {
            (Ok(key_b64), Ok(iv_b64), Ok(enc_b64)) => {
                let key = decode(key_b64).expect("Clave mal codificada");
                let iv = decode(iv_b64).expect("IV mal codificado");
                let ciphertext = decode(enc_b64).expect("Texto cifrado mal codificado");

                let cipher = Aes128Cbc::new_from_slices(&key, &iv)
                    .expect("Error creando el descifrador");
                let decrypted_data = cipher.decrypt_vec(&ciphertext)
                    .expect("Error al descifrar");

                String::from_utf8(decrypted_data).expect("API Key descifrada no es UTF-8")
            }
            _ => "".to_string(),
        };

        Config { id_company, api_key, iclock_config_path, domain_time, ip_server, usser_biotime, password_biotime }
    }   
}