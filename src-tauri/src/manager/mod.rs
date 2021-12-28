

pub mod rustssh{
use windows::Win32::Storage::FileSystem::{FILE_NOTIFY_CHANGE_ATTRIBUTES, FILE_NOTIFY_CHANGE_LAST_ACCESS};
use crate::systempaths::rustssh::SystemPath;
use crate::filetracker::rustssh::DirectoryTracker;
use crate::syncmanager::rustssh::*;
use crate::sessionmanager::rustssh::*;
use crate::configmanager::rustssh::{ConfigManager, RemitConfig};
use std::process::Command;
use std::env::current_dir;
use std::sync::{Arc, Mutex};
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
    rclone_m: Arc::<Mutex::<RCloneManager>>,

    /// Load Remit configs
    config_m: ConfigManager,

    /// Directory for tracking current path in the remote computer
    pub dir: Directory,

    file_tracker: DirectoryTracker,

    custom_path: String
}

#[allow(dead_code)]
impl Manager {

    /// Create a Manager with only Remit configs loaded
    pub fn new_empty() -> Result<Manager, IOError> {
        let mut path = SystemPath::new();
        path.set_path(".remote".to_string());
        let rclone_instance = Arc::new(Mutex::new(RCloneManager::new(None, Some(".remote".to_string()))));
        let mut m = Manager{ssh_m: SessionManager::new(None, None, None)?,
                        rclone_m: rclone_instance.clone(),
                        config_m: ConfigManager::new(),
                        dir: Directory::new(None),
                        file_tracker: DirectoryTracker::new(path, rclone_instance.clone()),
                        custom_path: ".remote".to_string()/*String::new()*/};
        m.config_m.load_configs()?;
        return Ok(m);
    }

    /// Create a manager with configs loaded and params pass into it
    pub fn new(host: String, username: String, pass: Option<String>, rclone_config: Option<String>, port_option: Option<String>) -> Result<Manager, IOError>{
        let mut m = Manager::new_empty()?;
        m.set_params(host, username, pass, rclone_config, None, port_option)?;
        return Ok(m);
    }

    /// set params for the manager and its sub managers
    /// 
    /// If the rclone_config provided doesn't exist it will be created with the parameters passed in
    pub fn set_params(&mut self, host:String, username: String, password: Option<String>, rclone_config: Option<String>,
                        pem_file: Option<String>, port_option: Option<String>) -> Result<(), IOError> {

        // load existing rclone configs by parsing rclone_m config show
        self.rclone_m.lock().unwrap().load_configs()?;
        let mut full_host = host.clone();
        full_host = format!("{}:{}", full_host, port_option.unwrap_or("22".to_string()));

        // if rclone_config doesn't exist create it and then set the name, otherwise just set the config name
        rclone_config.ok_or_else(||return IOError::new(IOErrorKind::Other, "no config")).and_then(|config: String| -> Result<String, IOError>{
            self.rclone_m.lock().unwrap().set_config(config.clone()).or_else(|_error: IOError| -> Result<(), IOError> {
                self.rclone_m.lock().unwrap().create_sftp_config(config.clone(),
                                                    username.clone(),
                                                    host.clone(),
                                                    password.clone(), pem_file)?;
                return Ok(());
            })?;
            self.rclone_m.lock().unwrap().set_config(config.clone())?;
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
        if !self.dir.path.set_path(self.ssh_m.run_command("pwd".to_string()).unwrap()) {
            println!("Error setting path");
        }
        let mut path = SystemPath::new();
        path.pushd(self.custom_path.clone());
        self.file_tracker.start_tracking(&mut path)?;
        return Ok(());
    }

    /// disconnect from current ssh endpoint
    /// TODO add error handling for stop tracking
    pub fn disconnect(&mut self) -> Result<(), IOError> {
        self.file_tracker.stop_tracking();
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

    /// downloads a file that exists in the current path. If the open flag contains true
    /// use window explorer to try and open the file
    /// 
    /// The file must exist in the path currently in Manager's dir file
    pub fn download_file(&mut self, name: String, open: Option<bool>) -> Result<(), IOError>{
        let r = self.rclone_m.lock().unwrap().download_remote_file(&mut self.dir, name.clone());
        if r?.success() {
            open.map(|open: bool| {
                if open {
                    let mut full_path = self.dir.path.clone();
                    full_path.pushd(name);
                    if self.custom_path.len() > 0 {
                        full_path.prepd(self.custom_path.clone());
                    }
                    let _output = Command::new("explorer")
                    .arg(format!("{}\\{}", current_dir().unwrap().to_str().unwrap(), full_path.get_windows_path_local()))
                    .output();
                }
            });
            return Ok(());
        } else {
            return Err(IOError::new(IOErrorKind::Other, "error during download"));
        }
    }

    pub fn upload_file(&mut self, name: String) -> Result<(), IOError>{
        let r = self.rclone_m.lock().unwrap().upload_local_file(&mut self.dir, name.clone());
        if r?.success() {
            return Ok(());
        } else {
            return Err(IOError::new(IOErrorKind::Other, "error during download"));
        }
    }

    /// get a list of remit configurations
    /// 
    /// returns a clone set of remit configurations
    pub fn get_configs(&mut self) -> Vec<RemitConfig>{
        return self.config_m.get_configs();
    }
}

}