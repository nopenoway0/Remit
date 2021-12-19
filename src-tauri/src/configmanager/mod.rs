pub mod rustssh {
use crate::systempaths::rustssh::SystemPath;
use std::collections::HashMap;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::fs::write;

type ioError = std::io::Error;

#[derive(Clone)]
pub struct RemitConfig {
    pub username: String,
    pub password: String,
    pub host: String,
    pub name: String,
    pub port: String,

    /// contains path from default directory to config not currently used
    pub path: SystemPath
}

/// manage remit configurations
/// 
/// ConfigManager is responsible for loading, reading, and saving Remit Configurations
pub struct ConfigManager{
    /// list of configs hashed by name
    configs: HashMap<String, RemitConfig>,

    /// where to find the configurations to load
    config_path: SystemPath
}

impl ConfigManager {

    /// construct config manager with default path set to ./configs
    pub fn new() -> ConfigManager {
        let mut manager = ConfigManager{configs: HashMap::new(), config_path: SystemPath::new()};
        manager.config_path.pushd("configs".to_string());
        return manager;
    }

    /// load all the remit configs (.rcfg) found in the directory
    pub fn load_configs(&mut self) -> Result<(), ioError>{
        return read_dir(self.config_path.get_path().clone()).and_then(|paths: std::fs::ReadDir| -> Result<(), ioError>{
            for path in paths {
                let dir = path.unwrap();
                let name = dir.file_name().into_string().unwrap();
                if name.contains(".rcfg") {
                    let full_filename = self.config_path.get_path().clone() + "/" + name.as_str();
                    self.read_config(full_filename);
                }
            }            
            return Ok(());
        });
    }

    /// use filename to parse remit config
    fn read_config(&mut self, filename: String) {
        let contents = read_to_string(filename)
            .expect("could not read config file");
        let mut config = RemitConfig::new();
        for line in contents.lines() {
            let mut args = line.split_whitespace();
            match args.next().unwrap() {
                "user"=>config.username.push_str(args.next().unwrap()),
                "pass"=>config.password.push_str(&line[5..line.len()]),
                "host"=>config.host.push_str(args.next().unwrap()),
                "name"=>config.name.push_str(args.next().unwrap()),
                "port"=>config.port.push_str(args.next().unwrap()),
                _=>{}
            }
        }
        self.configs.insert(config.name.clone(), config);
    }

    pub fn insert_config(&mut self, config: RemitConfig) {
        self.configs.insert(config.name.clone(), config);
    }

    pub fn save_config(&mut self, name: &str) -> Result<(), std::io::Error> {
        if !self.configs.contains_key(name) {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Config not found"));
        }
        let c = self.configs.get(name).unwrap();
        let full_path = self.config_path.get_path() + "/" + name;
        let contents = format!("user {}\npass {}\nhost {}\nname {}\nport {}\n", c.username, c.password, c.host, c.name, c.port);
        match write(full_path, contents) {
            Ok(_)=>return Ok(()),
            Err(e)=>return Err(e)
        }
    }

    // get a list of copied RemitConfigurations
    pub fn get_configs(&mut self) -> Vec<RemitConfig>{
        let mut configs = Vec::<RemitConfig>::new();
        for c in self.configs.clone() {
            configs.push(c.1);
        }
        return configs;
    }
}

impl RemitConfig {

    // create an empty Remit Config
    pub fn new() -> RemitConfig{
        return RemitConfig{username: String::new(), password: String::new(),
                                host: String::new(), name: String::new(),
                                port: String::new(),
                                path: SystemPath::new()};
    }
}


}