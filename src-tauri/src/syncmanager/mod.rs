pub mod rustssh {
use std::collections::HashMap;
use std::process::Command;
use std::string::ToString;
use std::fmt::Debug;
use std::os::windows::process::CommandExt;
use crate::*;

/// MS Windows flag for process creation, prevents window from being called
/// when starting up separate programs
static CREATE_NO_WINDOW: u32 = 0x08000000;

/// Represents an rclone configuration
#[derive(Clone, Debug)]
pub struct RCloneConfig {
    /// type of rclone config e.g. sftp
    config_type: String,
    /// configuration name
    pub name: String,
    host: String,
    user: String,
    pass: String
}

impl RCloneConfig {
    pub fn new(input: Option<String>) -> Result<RCloneConfig, IOError> {
        if input.is_none() {
            return Ok(RCloneConfig{config_type: String::new(),
                                name: String::new(),
                                host: String::new(),
                                user: String::new(),
                                pass: String::new()});
        }
        return RCloneConfig::parse_config(input.unwrap());
    }

    /// parse the information of an rclone config file
    /// 
    /// If any error occurs during parsing throws an error
    fn parse_config(input: String) -> Result<RCloneConfig, IOError>{
        let mut config = RCloneConfig::new(None).unwrap();
        for line in input.lines() {
            // process name of config
            if line.len() <= 2 {
                continue;
            }
            if line.chars().nth(0).unwrap() == '[' {
                config.name = line[1..line.len()-1].to_string();
            } else {
                let mut categories = line.split_whitespace();
                match categories.nth(0).ok_or(IOError::new(IOErrorKind::UnexpectedEof, "Error parsing configuration file"))? {
                    "type"=> config.config_type = categories.nth(1).ok_or(IOError::new(IOErrorKind::UnexpectedEof, "Error parsing configuration file"))?.to_string(),
                    "host"=> config.host = categories.nth(1).ok_or(IOError::new(IOErrorKind::UnexpectedEof, "Error parsing configuration file"))?.to_string(),
                    "pass"=> config.pass = categories.nth(1).ok_or(IOError::new(IOErrorKind::UnexpectedEof, "Error parsing configuration file"))?.to_string(),
                    "user"=> config.user = categories.nth(1).ok_or(IOError::new(IOErrorKind::UnexpectedEof, "Error parsing configuration file"))?.to_string(),
                    _=>{}

                }
            }
        }
        return Ok(config);
    }
}

/// manager for rclone callouts. only supports sftp
/// 
/// This manager converts common necessary functions used by Remit into 
/// appropriate commands to run in rclone and then parses the output
pub struct RCloneManager {
    exe: String,
    configs: HashMap<String, RCloneConfig>,   
    chosen_config: String,
    custom_path: String
}
#[allow(dead_code)]
impl RCloneManager {
    /// produce a new rlcone manager with the exe at the passed in 
    /// input. If input is none assume rclone.exe is in the current
    /// directory
    pub fn new(exe: Option<String>, custom_path: Option<String>) -> RCloneManager{
        return RCloneManager{exe: exe.unwrap_or("rclone.exe".to_string()), configs: HashMap::new(), chosen_config: String::new(),
                                custom_path: custom_path.unwrap_or("".to_string())};
    }

    /// load all rclone configurations
    /// 
    /// Output parses the results of rclone config show into RCloneConfig structs
    /// and then stores them in this manager
    pub fn load_configs(&mut self) -> Result<(), IOError>{
        let output = Command::new(self.exe.clone())
                    .arg("config")
                    .arg("show")
                    .creation_flags(CREATE_NO_WINDOW).output()?;
        let output_str = String::from_utf8(output.stdout).or(Err(IOError::new(IOErrorKind::UnexpectedEof, "Error converting stdout")))?;
        self.configs.clear();
        for config in output_str.split("\n") {
            let c = RCloneConfig::parse_config(config.to_string())?;
            self.configs.insert(c.name.clone(), c.clone());
        }
        return Ok(());
    }

    /// gets the list of configs as a vector of strings
    /// 
    /// only returns the configuration names not their contents
    pub fn list_config_names(&mut self) -> Vec<String> {
        let mut res: Vec<String> = Vec::new();
        for entry in &self.configs {
            res.push(entry.0.clone());
        }
        return res;
    }

    /// set the manager to use the config by name
    pub fn set_config(&mut self, name: String) -> Result<(), std::io::Error> {
        if  self.config_exists(&name) {
            self.chosen_config = name;
            return Ok(());      
        }
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Could not find config"));
    }

    /// check if rclone configuration exists by name
    pub fn config_exists(&mut self, name: &String) -> bool{
        return self.configs.contains_key(name);
    }

    /// delete an rclone configuration by name
    /// 
    /// If the configuration doesn't exist or an error occurs during the execution
    /// throw an error
    pub fn delete_config(&mut self, name: String) -> Result<std::process::Output, std::io::Error>{
        if self.config_exists(&name) {
            let output = Command::new(self.exe.clone())
                    .arg("config")
                    .arg("delete")
                    .arg(name.clone())
                    .creation_flags(CREATE_NO_WINDOW)
                    .output();
            if output.is_ok() {
                self.configs.remove(&name);
            }
            return output;
        }
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "config doesn't exist"));
    }

    /// create an sftp rclone config with the given parameters
    /// 
    /// If a configuration already exists with that name, return an error
    pub fn create_sftp_config(&mut self, name: String, username: String, 
                                host: String, password: Option<String>,
                                pem_file: Option<String>) -> Result<std::process::Output, std::io::Error>{
        if self.config_exists(&name) {
            return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Config already exists"));
        }
        let mut command = Command::new(self.exe.clone());
        command.arg("config")
                .arg("create")
                .arg(name)
                .arg("sftp")
                .arg("host")
                .arg(host)
                .arg("user")
                .arg(username)
                .creation_flags(CREATE_NO_WINDOW);
        if password.is_some() {
            command.arg("pass")
                    .arg(format!("{}", password.unwrap()));
        }
        if pem_file.is_some() {
            command.arg("key-file")
                    .arg(format!("\"{}\"", pem_file.unwrap()));
        }
        command.arg("--non-interactive");
        self.load_configs()?;
        return command.output();
    }

    /// Attempt to download a remote file using directory and filename
    pub fn download_remote_file(&mut self, d: &mut Remit::Directory, filename: String) -> Result<std::process::ExitStatus, IOError>{
        let mut file_path = d.path.clone();
        let mut local_path = file_path.clone();
        if self.custom_path.len() > 0 {
            local_path.prepd(self.custom_path.clone());
        }
        file_path.pushd(filename.clone());
        let output = Command::new(self.exe.clone())
                                    .arg("sync")
                                    .arg(format!("{}:{}", self.chosen_config, file_path.get_path()))
                                    .arg(format!("{}", local_path.get_windows_path_local()))
                                    .creation_flags(CREATE_NO_WINDOW)
                                    .output()?;
        return Ok(output.status);
    }

    pub fn upload_local_file(&mut self, d: &mut Remit::Directory, filename: String) -> Result<std::process::ExitStatus, IOError>{
        let file_path = d.path.clone();
        let mut local_path = file_path.clone();
        if self.custom_path.len() > 0 {
            local_path.prepd(self.custom_path.clone());
        }
        local_path.pushd(filename.clone());
        println!("rclone.exe sync {} {}:{}", local_path.get_windows_path_local(), self.chosen_config, file_path.get_path());
        let output = Command::new(self.exe.clone())
                                    .arg("sync")
                                    .arg(format!("{}", local_path.get_windows_path_local()))
                                    .arg(format!("{}:{}", self.chosen_config, file_path.get_path()))
                                    .creation_flags(CREATE_NO_WINDOW)
                                    .output()?;
        return Ok(output.status);
    }
}



}