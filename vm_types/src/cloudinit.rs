use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chpasswd {
    pub expire: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerState {
    pub delay: String,
    pub mode: String,
    pub message: String,
    pub timeout: u32,
    pub condition: String,
}

impl Default for PowerState {
    fn default() -> Self {
        PowerState {
            delay: "now".to_string(),
            mode: "poweroff".to_string(),
            message: "The system is going down for power off NOW!".to_string(),
            timeout: 1,
            condition: "true".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    pub password: String,
    pub chpasswd: Chpasswd,
    pub ssh_pwauth: bool,
    pub power_state: PowerState,
}

impl CloudConfig {
    pub fn to_yaml(&self) -> Result<String, crate::Error> {
        let yaml_str = serde_yaml::to_string(&self)?;
        Ok("#cloud-config\n".to_string() + &yaml_str)
    }
}
