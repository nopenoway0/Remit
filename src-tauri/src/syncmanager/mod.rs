pub mod rustssh {
use std::collections::HashMap;
use std::process::Command;
use std::string::ToString;
use std::fmt::Debug;
use std::os::windows::process::CommandExt;
use crate::sessionmanager::rustssh::Directory;

static CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Clone, Debug)]
pub struct RCloneConfig {
    config_type: String,
    pub name: String,
    host: String,
    user: String,
    pass: String
}

impl RCloneConfig {
    pub fn new(input: Option<String>) -> RCloneConfig {
        if input.is_none() {
            return RCloneConfig{config_type: String::new(),
                                name: String::new(),
                                host: String::new(),
                                user: String::new(),
                                pass: String::new()};
        }
        return RCloneConfig::parse_config(input.unwrap());
    }

    fn parse_config(input: String) -> RCloneConfig{
        let mut config = RCloneConfig::new(None);
        for line in input.lines() {
            // process name of config
            if line.len() <= 0 {
                break
            }
            if line.chars().nth(0).unwrap() == '[' {
                config.name = line[1..line.len()-1].to_string();
            } else {
                let mut categories = line.split_whitespace();
                match categories.nth(0).unwrap() {
                    "type"=> config.config_type = categories.nth(1).unwrap().to_string(),
                    "host"=> config.host = categories.nth(1).unwrap().to_string(),
                    "pass"=> config.pass = categories.nth(1).unwrap().to_string(),
                    "user"=> config.user = categories.nth(1).unwrap().to_string(),
                    _=>{}

                }
            }
        }
        return config;
    }
}

pub struct RCloneManager {
    exe: String,
    configs: HashMap<String, RCloneConfig>,   
    chosen_config: String
}
#[allow(dead_code)]
impl RCloneManager {
    pub fn new() -> RCloneManager{
        return RCloneManager{exe: "rclone.exe".to_string(), configs: HashMap::new(), chosen_config: String::new()};
    }

    pub fn load_configs(&mut self) {
        let output = Command::new(self.exe.clone())
                    .arg("config")
                    .arg("show")
                    .creation_flags(CREATE_NO_WINDOW).output().unwrap();
        let output_str = String::from_utf8(output.stdout).unwrap();
        self.configs.clear();
        for config in output_str.split("\n") {
            let c = RCloneConfig::parse_config(config.to_string());
            self.configs.insert(c.name.clone(), c.clone());
        }
    }

    pub fn list_config_names(&mut self) -> Vec<String> {
        let mut res: Vec<String> = Vec::new();
        for entry in &self.configs {
            res.push(entry.0.clone());
        }
        return res;
    }

    pub fn set_config(&mut self, name: String) -> Result<(), std::io::Error> {
        if  self.config_exists(&name) {
            self.chosen_config = name;
            return Ok(());      
        }
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Could not find config"));
    }

    pub fn config_exists(&mut self, name: &String) -> bool{
        return self.configs.contains_key(name);
    }

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
        self.load_configs();
        return command.output();
    }

    pub fn download_remote_file(&mut self, d: &mut Directory, filename: String) -> std::process::ExitStatus{
        let mut file_path = d.path.clone();
        file_path.pushd(filename.clone());
        let output = Command::new(self.exe.clone())
                                    .arg("sync")
                                    .arg(format!("{}:{}", self.chosen_config, file_path.get_path()))
                                    .arg(format!(".{}", d.path.get_path()))
                                    .creation_flags(CREATE_NO_WINDOW)
                                    .output().unwrap();
        return output.status;
    }

    pub fn push_local_file(&mut self, d:&mut Directory, filename: String) {
        let file_path = format!(".{}/{}", d.path.get_path(), filename);
        let _output = Command::new(self.exe.clone())
                                    .arg("sync")
                                    .arg(file_path)
                                    .arg(format!("{}:{}", self.chosen_config, d.path.get_path()))
                                    .creation_flags(CREATE_NO_WINDOW)
                                    .output();
    }
}



}