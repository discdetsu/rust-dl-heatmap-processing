use std::collections::HashMap;
use std::env;

#[derive(Debug)]
pub struct OrchestrateConfig {
    pub service_host: String,
    pub service_port: u16,
    pub service_db_path: String,
    pub config_services: HashMap<String, ServiceConfig>,
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub url: String,
    pub data: Option<String>,
    pub headers: HashMap<String, String>,
    pub service: String,
}

impl OrchestrateConfig {
    pub fn new() -> Self {
        let mut config = OrchestrateConfig {
            service_host: "0.0.0.0".to_string(),
            service_port: 50011,
            service_db_path: "config/service_db_v8.csv".to_string(),
            config_services: HashMap::new(),
        };
        config.setup_service_config();
        config
    }

    fn setup_service_config(&mut self) {
        let tuberculosis_service_url = env::var("DL_URL_TB").unwrap_or_else(|_| {
            "http://tuberculosis_service:50001/deep-learning/service/tuberculosis/image_binary".to_string()
        });

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/x-image".to_string());

        self.config_services.insert(
            "tuberculosis_service".to_string(),
            ServiceConfig {
                url: tuberculosis_service_url,
                data: None,
                headers,
                service: "tuberculosis_service".to_string(),
            },
        );
    }
}
