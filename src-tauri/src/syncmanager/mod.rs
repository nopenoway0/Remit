//! The syncmanager is used to pass commands to rclone. It doesn't automatically sync files, that is what the FileTracker is for. Instead 
//! it makes common rsync commands. With an accessible rclone executable, this module can construct 
//! queries to manage rclone configurations and push/pull files
//! 
pub mod rustssh {
use std::collections::HashMap;
use std::process::Command;
use std::string::ToString;
use std::fmt::Debug;
use std::os::windows::process::CommandExt;
use std::fs::metadata;
use crate::*;

/// MS Windows flag for process creation. This flag prevents a console window from appearing when doing
/// callouts to the rclone exe
static CREATE_NO_WINDOW: u32 = 0x08000000;

/// Represents an rclone configuration
#[derive(Clone, Debug)]
pub struct RCloneConfig {
    /// Type of rclone configuration e.g. sftp
    config_type: String,
    /// configuration name
    pub name: String,
    host: String,
    user: String,
    pass: String
}

impl RCloneConfig {
    /// Create a new RCloneConfiguration object
    /// # Arguments
    /// * `input` - The text source of an rclone configuration. None creates an empty configuration
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

    /// Parse the information contained in an rclone configuration
    /// # Arguments
    /// * `input` - rclone configuration file text
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

/// Manager for rclone commands. Currently only supports sftp configurations
/// 
/// This manager converts common necessary functions used by Remit into 
/// appropriate commands to run in rclone and then parses the output
pub struct RCloneManager {
    /// The name of the rclone executable
    exe: String,
    /// A map of rclone configurations. These are stored by name
    configs: HashMap<String, RCloneConfig>,   
    /// The currently chosen configuration by name
    pub chosen_config: String,
    /// Path which contains the rclone executable
    custom_path: String
}
#[allow(dead_code)]
impl RCloneManager {
    /// Produces a new rlcone manager. If input is none assume rclone.exe is in the current
    /// directory
    /// # Arguments
    /// * `exe` - Name of the rclone executable, if None assume rclone.exe
    /// * `custom_path` - Path to this executable. If None, assume in the current directory
    pub fn new(exe: Option<String>, custom_path: Option<String>) -> RCloneManager{
        return RCloneManager{exe: exe.unwrap_or("rclone.exe".to_string()), configs: HashMap::new(), chosen_config: String::new(),
                                custom_path: custom_path.unwrap_or("".to_string())};
    }

    /// Check if the required rclone executable exists
    pub fn rclone_exe_exists(&self) -> bool{
        let full_path = self.custom_path.clone() + &self.exe.clone();
        let res = metadata(full_path);
        if res.is_ok() {
            return res.unwrap().is_file();
        }
        return false;
    }

    /// Load all rclone configurations
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

    /// Get all the current rclone configuration names
    /// 
    /// This method only returns the configuration names not their contents
    pub fn list_config_names(&mut self) -> Vec<String> {
        let mut res: Vec<String> = Vec::new();
        for entry in &self.configs {
            res.push(entry.0.clone());
        }
        return res;
    }

    /// Set the manager to use a configuration by name
    /// # Arguments
    /// * `name` - Name of configuration to use
    pub fn set_config(&mut self, name: String) -> Result<(), std::io::Error> {
        if  self.config_exists(&name) {
            self.chosen_config = name;
            return Ok(());      
        }
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Could not find config"));
    }

    /// Checks if the rclone configuration exists by name
    /// # Arguments
    /// * `name` - Name of configuration to check if exists
    pub fn config_exists(&mut self, name: &String) -> bool{
        return self.configs.contains_key(name);
    }

    /// Deletes an rclone configuration by name
    /// 
    /// If the configuration doesn't exist or an error occurs during the execution
    /// throw an error
    /// # Arguments
    /// * `name` - Name of configuration to delete
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

    /// Create an sftp rclone config with the given parameters
    /// 
    /// If a configuration already exists with that name, returns an error
    /// # Arguments
    /// * `name` - Name of the configuration
    /// * `username` - 
    /// * `host` - 
    /// * `password`
    /// * `pem_file` - Location and name of PEM file. **Currently not used**
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

    /// Attempt to download a remote file using directory and filename. Throws an error if rsync exits improperly.
    /// 
    /// The local directory and remote directory work in tandem to download the file to the correct location.
    /// For example, local_dir may point to `C:\Users\user\Desktop\config_name\.remote\home\user\` and remote_dir would point to
    /// `/home/user`. When a file is copied from `/home/user` it will be copied to the Windows path on the local machine.
    /// # Arguments
    /// * `local_dir` - A system path object that is set to point where the file should be created
    /// * `remote_dir` - A system path that should point to the file's location on the remote server
    /// * `filename` - Name of file to upload
    pub fn download_remote_file(&mut self, local_dir: Remit::SystemPath, remote_dir: Remit::SystemPath, filename: String) -> Result<std::process::ExitStatus, IOError>{
        let mut local_path = local_dir.clone();
        let mut remote_path = remote_dir.clone();
        if self.custom_path.len() > 0 {
            local_path.prepd(self.custom_path.clone());
        }
        remote_path.pushd(filename.clone());
        println!("rclone.exe sync {}:{} {}", self.chosen_config, remote_path.get_path(), local_path.get_windows_path_local(), );
        let output = Command::new(self.exe.clone())
                                    .arg("sync")
                                    .arg(format!("{}:{}", self.chosen_config, remote_path.get_path()))
                                    .arg(format!("{}", local_path.get_windows_path_local()))
                                    .creation_flags(CREATE_NO_WINDOW)
                                    .output()?;
        return Ok(output.status);
    }

    /// Uploads a local file to the remote machine
    /// 
    /// This function operates in the opposite fashion as the [`RCloneManager::download_remote_file`].
    /// # Arguments
    /// * `local_dir` - A system path object that is set to point where the file is located
    /// * `remote_dir` - A system path that should point to the file's location on the remote server
    /// * `filename` - Name of file to upload
    pub fn upload_local_file(&mut self, local_path: Remit::SystemPath, remote_path: Remit::SystemPath, filename: String) -> Result<std::process::ExitStatus, IOError>{
        let mut local_path = local_path.clone();
        let remote_path = remote_path.clone();
        if self.custom_path.len() > 0 {
            local_path.prepd(self.custom_path.clone());
        }
        local_path.pushd(filename.clone());
        println!("rclone.exe sync {} {}:{}", local_path.get_windows_path_local(), self.chosen_config, remote_path.get_path());
        let output = Command::new(self.exe.clone())
                                    .arg("sync")
                                    .arg(format!("{}", local_path.get_windows_path_local()))
                                    .arg(format!("{}:{}", self.chosen_config, remote_path.get_path()))
                                    .creation_flags(CREATE_NO_WINDOW)
                                    .output()?;
        return Ok(output.status);
    }
}



}