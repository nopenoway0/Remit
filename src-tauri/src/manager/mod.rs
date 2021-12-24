

pub mod rustssh{

use crate::syncmanager::rustssh::*;
use crate::sessionmanager::rustssh::*;
use crate::sessionmanager::rustssh::Directory;
use crate::configmanager::rustssh::ConfigManager;
use crate::configmanager::rustssh::RemitConfig;
use std::process::Command;
use std::env::current_dir;

type IOError = std::io::Error;
type IOErrorKind = std::io::ErrorKind;

/// Primary manager over the ssh, rclone and config managers. 
/// 
/// The app uses this
/// as an interface to communicate with the other managers
pub struct Manager {
    /// Ssh session manager to manage ssh commands to the host
    ssh_m: SessionManager,
    /// rclone manager. Used to interface and use commands with the rclone binary
    rclone_m: RCloneManager,

    /// Load Remit configs
    config_m: ConfigManager,

    /// Directory for tracking current path in the remote computer
    pub dir: Directory
}

#[allow(dead_code)]
impl Manager {

    /// Create a Manager with only Remit configs loaded
    pub fn new_empty() -> Manager {
        let mut m = Manager{ssh_m: SessionManager::new(None, None, None),
                        rclone_m: RCloneManager::new(),
                        config_m: ConfigManager::new(),
                        dir: Directory::new(None)};
        match m.config_m.load_configs() {
            _=>{}
        }
        return m;
    }

    /// Create a manager with configs loaded and params pass into it
    pub fn new(host: String, username: String, pass: Option<String>, rclone_config: Option<String>, port_option: Option<String>) -> Result<Manager, IOError>{
        let mut m = Manager::new_empty();
        m.set_params(host, username, pass, rclone_config, None, port_option)?;
        return Ok(m);
    }

    /// set params for the manager and its sub managers
    /// 
    /// If the rclone_config provided doesn't exist it will be created with the parameters passed in
    pub fn set_params(&mut self, host:String, username: String, password: Option<String>, rclone_config: Option<String>,
                        pem_file: Option<String>, port_option: Option<String>) -> Result<(), IOError> {

        // load existing rclone configs by parsing rclone_m config show
        self.rclone_m.load_configs();
        let mut full_host = host.clone();
        full_host = format!("{}:{}", full_host, port_option.unwrap_or("22".to_string()));

        // if rclone_config doesn't exist create it and then set the name, otherwise just set the config name
        rclone_config.ok_or_else(||return IOError::new(IOErrorKind::Other, "no config")).and_then(|config: String| -> Result<String, IOError>{
            self.rclone_m.set_config(config.clone()).or_else(|_error: IOError| -> Result<(), IOError> {
                self.rclone_m.create_sftp_config(config.clone(),
                                                    username.clone(),
                                                    host.clone(),
                                                    password.clone(), pem_file)?;
                return Ok(());
            })?;
            self.rclone_m.set_config(config.clone())?;
            return Ok("".to_string());
        })?;

        // set up our credentials for ssh
        self.ssh_m.set_params(Some(username.clone()), password.clone(), Some(full_host.clone()));
        return Ok(());
    }

    /// connect to the ssh endpoint using the set parameter
    /// 
    /// If connected succesfully, the ssh_m manager with modify the Manager directory
    /// to have the absolute path by parsing pwd
    pub fn connect(&mut self) -> Result<(), IOError>{
        self.ssh_m.connect()?;
        self.dir.path.set_path(self.ssh_m.run_command("pwd".to_string()).unwrap());
        return Ok(());
    }

    /// disconnect from current ssh endpoint
    pub fn disconnect(&mut self) -> Result<(), IOError> {
        return self.ssh_m.disconnect();
    }

    /// update the manager directory files to reflect the current path
    /// 
    /// This method only needs to be called when the path has changed. It performs
    /// and parses an ls -al on the path present in the directory to read files
    /// and their attributes into the same directory
    pub fn get_directory(&mut self) -> Result<(), IOError>{
        self.ssh_m.get_directory(&mut self.dir)?;
        return Ok(());
    }

    /// adds the name variable to the path after performing a check it exists
    /// and is a directory
    /// 
    /// Navigate updates the path in the Manager's dir object. It does not call get_directory
    /// and therefore does not update the file contents
    pub fn navigate(&mut self, name: String) -> Result<(), std::io::Error>{
        let mut dir = self.dir.clone();
        self.ssh_m.navigate(&mut dir, name)?;
        self.dir = dir;
        return Ok(());
    }

    /// downloads a fail that exists in the current path. If the open flag contains true
    /// use window explorer to try and open the file
    /// 
    /// The file must exist in the path currently in Manager's dir file
    pub fn download_file(&mut self, name: String, open: Option<bool>) -> Result<(), IOError>{
        let r = self.rclone_m.download_remote_file(&mut self.dir, name.clone());
        if r.success() {
            open.map(|open: bool| {
                if open {
                    let mut full_path = self.dir.path.clone();
                    full_path.pushd(name);
                    let _output = Command::new("explorer")
                    .arg(format!("{}{}", current_dir().unwrap().to_str().unwrap(), full_path.get_windows_path()))
                    .output();
                }
            });
            return Ok(());
        } else {
            return Err(IOError::new(IOErrorKind::Other, "error during download"));
        }
    }

    pub fn push_local_file(&mut self, d:&mut Directory, name: String) {
        self.rclone_m.push_local_file(d, name);
    }

    /// get a list of remit configurations
    /// 
    /// returns a clone set of remit configurations
    pub fn get_configs(&mut self) -> Vec<RemitConfig>{
        return self.config_m.get_configs();
    }
}

}