use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChpasswdUser {
    pub name: String,
    pub password: String,
    #[serde(rename = "type")]
    pub type_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chpasswd {
    pub expire: bool,
    pub users: Vec<ChpasswdUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerState {
    pub delay: String,
    pub mode: String,
    pub message: String,
    pub timeout: u32,
    pub condition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchInterface {
    pub macaddress: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthernetConfig {
    #[serde(rename = "match")]    
    pub match_interface: MatchInterface,
    pub addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkConfig {
    pub network: PresetNetworkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetNetworkConfig {
    pub version: u8,
    pub ethernets: HashMap<String, EthernetConfig>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDataCloudInit {
    pub hostname: String,
    pub chpasswd: Chpasswd,
    pub ssh_pwauth: bool,
    pub power_state: PowerState,
    pub disable_root: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudInit {
    pub user_data: UserDataCloudInit,
    pub network_config: PresetNetworkConfig,
}

impl UserDataCloudInit {
    pub fn to_yaml(&self) -> Result<String, crate::Error> {
        let yaml_str = serde_yaml::to_string(&self)?;
        Ok("#cloud-config\n".to_string() + &yaml_str)
    }
}

impl PresetNetworkConfig {
    pub fn to_yaml(&self) -> Result<String, crate::Error> {
        let yaml_str = serde_yaml::to_string(&NetworkConfig {
            network: self.clone(),
        })?;
        Ok(yaml_str)
    }
}
