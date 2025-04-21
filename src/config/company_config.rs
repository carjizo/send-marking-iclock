use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};

#[derive(Serialize, Deserialize, Debug)]
pub struct Iclock {
    pub ip: String,
    pub port: u16,
    pub serialNumber: String,
    pub nameDispo: Option<String>,
    pub messageError: Option<String>,
    pub status: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompanyConfiguration {
    pub idCompany: String,
    pub ruc: String,
    pub razonSocial: String,
    pub iclocks: Vec<Iclock>,
    pub status: bool,
}

impl CompanyConfiguration {
    pub fn save_to_file<P: AsRef<Path>>(config: &CompanyConfiguration, path: P) -> std::io::Result<()> {
        let mut file = std::fs::File::create(&path)?;
        file.write_all(b"{}")?;
        drop(file);

        let json = serde_json::to_string_pretty(&config)?;
        std::fs::write(&path, json)?;

        let content = std::fs::read_to_string(&path)?;
        if content.trim().is_empty() || content.trim() == "{}" {
            println!("Archivo JSON sigue vacío tras intentar guardar. Abortando.");
            return Ok(());
        }

        println!("Archivo JSON guardado correctamente.");
        Ok(())
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read_to_string(path)?;
        let config: CompanyConfiguration = serde_json::from_str(&data)?;
        Ok(config)
    }
}
