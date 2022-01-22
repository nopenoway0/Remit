//! A Remit configuration manager. This class is responsible for loading and saving all the Remit configurations.
//! 
//! A Remit configuration file has a suffix .rcfg. It has very basic very strict format. Since the parsing is very basic, each file
//! must match this format exactly. All this information is stored in order to connect to servers via ssh
//! 
//! ```
//! name Name of the configuration
//! password ssh password
//! host the host
//! port portasnumber
//! username ssh username 
//! ```
//! 
//! The configuration manager will store these configurations under the configs folder in the local directory. This is currently hardcoded but will most likely
//! change to allow users to better manager their configuration files.

pub mod rustssh {
use std::collections::HashMap;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::fs::write;
use std::fs::create_dir_all;
use crate::*;

/// Contains the information of a Remit configuration file. Since Remit currently only supports username/password auth,
/// the information in this struct is all that's needed to connect to a server
#[derive(Clone)]
pub struct RemitConfig {
    pub username: String,
    pub password: String,
    pub host: String,
    pub name: String,
    pub port: String,

    /// This is the path from the local directory to the config file location. **Not currently used**
    pub path: Remit::SystemPath
}

/// ConfigManager is responsible for loading, reading, and saving Remit Configurations
pub struct ConfigManager{
    /// All loaded [`RemitConfig`]s stored by name
    configs: HashMap<String, RemitConfig>,

    /// The path to where the configuration files are
    config_path: Remit::SystemPath
}

#[allow(dead_code)]
impl ConfigManager {

    /// Creates a new ConfigManager with its config_path set to ./configs
    /// # Arguments
    /// * `initialize_dir` - If true, the directory in the config_path will be created. Recommended, as errors may occur if the directory does not
    /// exist
    pub fn new(initialize_dir: bool) -> ConfigManager {
        let mut manager = ConfigManager{configs: HashMap::new(), config_path: Remit::SystemPath::new()};
        manager.config_path.pushd("configs".to_string());
        if initialize_dir {
            manager.force_config_directory();
        }
        return manager;
    }

    /// This forces creation of the configuration directory. If the directory has multiple levels, those will be created as well.
    pub fn force_config_directory(&self) {
        let path = self.config_path.get_windows_path_local();
        let _r = create_dir_all(path);
    }

    /// Load all of the RemitConfiguration files - all files that contain `.rcfg` - in the config_path directory
    pub fn load_configs(&mut self) -> Result<(), IOError>{
        return read_dir(self.config_path.get_path().clone()).and_then(|paths: std::fs::ReadDir| -> Result<(), IOError>{
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

    /// Parse a configuration file. The parsed file will be stored in the manager's config map
    /// # Arguments
    /// * `filename` - Name of configuration file
    fn read_config(&mut self, filename: String) {
        let contents = read_to_string(filename)
            .expect("could not read config file");
        let mut config = RemitConfig::new();
        for line in contents.lines() {
            let mut args = line.split_whitespace();
            match args.next().unwrap() {
                "username"=>config.username.push_str(&line[9..line.len()]),
                "password"=>config.password.push_str(&line[9..line.len()]),
                "host"=>config.host.push_str(args.next().unwrap()),
                "name"=>config.name.push_str(&line[5..line.len()]),
                "port"=>config.port.push_str(args.next().unwrap()),
                _=>{}
            }
        }
        self.configs.insert(config.name.clone(), config);
    }

    /// Add a configuration file to the manager's config map. If 2 configurations have the same name, the old configuration
    /// will be overwritten
    /// # Arguments
    /// * `config` - Config to add
    pub fn insert_config(&mut self, config: RemitConfig) {
        self.configs.insert(config.name.clone(), config);
    }

    /// Save a configuration to file. If the names does not exist in the map of configs, then throw an error
    /// # Arguments
    /// * `name` - Name of config to save to file
    pub fn save_config(&mut self, name: &str) -> Result<(), std::io::Error> {
        if !self.configs.contains_key(name) {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Config not found"));
        }
        let c = self.configs.get(name).unwrap();
        let full_path = self.config_path.get_path() + "/" + name + ".rcfg";
        let contents = format!("username {}\npassword {}\nhost {}\nname {}\nport {}\n", c.username, c.password, c.host, c.name, c.port);
        match write(full_path, contents) {
            Ok(_)=>return Ok(()),
            Err(e)=>return Err(e)
        }
    }

    /// Get a vector of all [`RemitConfig`] files in the manager's map
    pub fn get_configs(&mut self) -> Vec<RemitConfig>{
        let mut configs = Vec::<RemitConfig>::new();
        for c in self.configs.clone() {
            configs.push(c.1);
        }
        return configs;
    }
}

impl RemitConfig {
    /// Create an empty [`RemitConfig`] object
    pub fn new() -> RemitConfig{
        return RemitConfig{username: String::new(), password: String::new(),
                                host: String::new(), name: String::new(),
                                port: String::new(),
                                path: Remit::SystemPath::new()};
    }
}

}