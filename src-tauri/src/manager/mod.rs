//! This module is the main interface for the tauri backend. It is a manager that wraps and simplifies calls to the other managers. It consists
//! of a SessionManager - to make ssh commands, an RcloneManager for wrapping sync functions, a ConfigManager to load and save Remit configurations,
//! a Directory variable to track the current directory and a DirectoryTracker to start and stop tracking at request


pub mod rustssh{
use std::process::Command;
use std::env::current_dir;
use std::sync::{Arc, Mutex};
use std::fs::{remove_file, rename, remove_dir_all, create_dir_all};
use crate::*;

/// Primary manager interface for the tauri backend containg ssh, rclone and config managers. 
/// 
/// The app uses this as an interface to communicate with the other managers
pub struct Manager {
    /// Ssh session manager to manage ssh commands to the host
    ssh_m: Remit::SessionManager,
    /// rclone manager. Used to interface and use commands with the rclone binary
    rclone_m: Arc::<Mutex::<Remit::RCloneManager>>,

    /// Load Remit configs
    config_m: Remit::ConfigManager,

    /// Directory for tracking current path in the remote computer
    pub dir: Remit::Directory,

    /// Tracks all events for the chosen configuration.
    /// e.g. If your current configuration is MySampleConfig, then the DirectoryTracker will be watching for all changes
    /// under ./MySampleConfig. The folder will be made if it doesn't exist
    file_tracker: Remit::DirectoryTracker,

    /// Not currently used
    custom_path: String
}

#[allow(dead_code)]
impl Manager {

    /// Create a new manager with the current path set to .remote, rclone executable set to rclone-x86_64-pc-windows-msvc.xe,
    /// and custom path sent to .remote. Then load all Remit configurations
    pub fn new_empty() -> Result<Manager, IOError> {
        let mut path = Remit::SystemPath::new();
        path.set_path(".remote".to_string());
        let rclone_instance = Arc::new(Mutex::new(Remit::RCloneManager::new(Some("rclone-x86_64-pc-windows-msvc.exe".to_string()), None)));
        let mut m = Manager{ssh_m: Remit::SessionManager::new(None, None, None)?,
                        rclone_m: rclone_instance.clone(),
                        config_m: Remit::ConfigManager::new(true),
                        dir: Remit::Directory::new(None),
                        file_tracker: Remit::DirectoryTracker::new(path, rclone_instance.clone()),
                        custom_path: ".remote".to_string()/*String::new()*/};
        m.config_m.load_configs()?;
        return Ok(m);
    }

    /// Create folder in the current directory both locally and remotely
    /// # Arguments
    /// * `dirname` - Name of directory to change
    pub fn create_dir(&mut self, dirname: &String) -> Result<(), IOError> {
        let mut remote_path = self.dir.path.clone();
        remote_path.pushd(dirname.clone());
        self.ssh_m.run_command(format!("mkdir \"{}\"", remote_path.get_path()))?;
        create_dir_all(format!("{}\\.remote\\{}", self.rclone_m.lock().unwrap().chosen_config.clone(), remote_path.get_windows_path_local()))?;
        return Ok(());
    }

    /// Create a file on the remote machine at the current directory
    /// # Arguments
    /// * `filename` - File name to create
    pub fn create_file(&mut self, filename: &String) -> Result<(), IOError>{
        let mut remote_path = Remit::SystemPath::new();
        remote_path.set_win_path(self.dir.path.get_path());
        remote_path.pushd(filename.clone());
        self.ssh_m.run_command(format!("touch \"{}\"", remote_path.get_path()))?;
        return Ok(());
    }

    /// Rename a file locally ( if it exists ) and remotely
    /// # Arguments
    /// * `file` - old file name
    /// * `new_name` - name to rename file to
    pub fn rename_file(&mut self, file: String, new_name: String) -> Result<(), IOError> {
        let mut local_path = Remit::SystemPath::new();
        local_path.set_win_path(format!("{}\\.remote\\{}", self.rclone_m.lock().unwrap().chosen_config.clone(), self.dir.path.get_windows_path_local()));
        let mut remote_path = Remit::SystemPath::new();
        remote_path.set_win_path(self.dir.path.get_path());
        local_path.pushd(file.clone());
        remote_path.pushd(file);
        let mut new_path = local_path.clone();
        new_path.popd();
        new_path.pushd(new_name.clone());

        println!("rename \"{}\" to \"{}\"", local_path.get_windows_path_local(), new_path.get_windows_path_local());
        let _r = rename(local_path.get_windows_path_local(), new_path.get_windows_path_local());
        let mut remote_path_new = remote_path.clone();
        remote_path_new.popd();
        remote_path_new.pushd(new_name);
        
        println!("mv \"{}\" \"{}\"", remote_path.get_path(), remote_path_new.get_path());
        self.ssh_m.run_command(format!("mv \"{}\" \"{}\"", remote_path.get_path(), remote_path_new.get_path()))?;

        return Ok(());
    }

    /// Delete a file both remotely and locally. If the recursive option is true, use `rm -r` in ssh command
    /// # Arguments
    /// * `file` - File/Directory to delete
    /// * `recursive` - If true use `rm -r` else use `rm`. Needs to be set to true to delete directories
    pub fn delete_file(&mut self, file: String, recursive: bool) -> Result<String, IOError>{
        let mut local_path = Remit::SystemPath::new();
        local_path.set_win_path(format!("{}\\.remote\\{}", self.rclone_m.lock().unwrap().chosen_config.clone(), self.dir.path.get_windows_path_local()));
        let mut remote_path = Remit::SystemPath::new();
        remote_path.set_win_path(self.dir.path.get_path());
        local_path.pushd(file.clone());
        remote_path.pushd(file);

        let mut result_string: String = "".to_string();
        // delete locally
        let res1;
        if recursive {
            res1 = remove_dir_all(local_path.get_windows_path_local());
        } else {
            res1 = remove_file(local_path.get_windows_path_local());
        }
        if res1.is_err() {
            result_string += &res1.unwrap_err().to_string();
        }
        println!("{}",local_path.get_windows_path_local());
        // delete remotely
        println!("rm \"{}\"", remote_path.get_path());
        let res2;
        if recursive {
            res2 = self.ssh_m.run_command(format!("rm -r \"{}\"", remote_path.get_path()));
        } else {
            res2 = self.ssh_m.run_command(format!("rm \"{}\"", remote_path.get_path()));
        }
        if res2.is_err() {
            result_string += &res2.unwrap_err().to_string();
        } else {
            result_string += &res2.unwrap();
        }
        return Ok(result_string);
    }

    /// Create a manager using the given configuration parameters
    /// # Arguments
    /// * `host`
    /// * `username`
    /// * `pass` - Password
    /// * `rclone_config` -
    /// * `port-option` - Port in the form of a number e.g. 22
    /// * `rlcone_config` - Name of rclone configuration. The rclone configs are created alongside Remit configurations
    pub fn new(host: String, username: String, pass: Option<String>, rclone_config: Option<String>, port_option: Option<String>) -> Result<Manager, IOError>{
        let mut m = Manager::new_empty()?;
        m.set_params(host, username, pass, rclone_config, None, port_option)?;
        return Ok(m);
    }

    /// Check whether or not a valid rclone executable exists
    pub fn rclone_exe_exists(&self) -> bool {
        return self.rclone_m.lock().unwrap().rclone_exe_exists();
    }

    /// Set the credentials required to connect to a host.
    /// 
    /// The arguments passed into this method has the potential to trigger an rclone configuration creation. If the rclone_config name is not
    /// found, then the configuration will be created using the inputs
    /// # Arguments
    /// * `host` - Host ( endpoint )
    /// * `username`
    /// * `password`
    /// * `rlcone_config` - Name of the rclone configuration that contains this credential information
    /// * `pem_file` - A file containing the necessary key information to connect via ssh **currently not used**
    /// * `port_option` - Remote server ssh port. If no port is passed in, port 22 is assumed
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

    /// Create an ssh session with the stored endpoint.
    /// 
    /// Upon a successful connection, the manager will obtain the absolute path after connection. This path is stored in the manager
    /// and is used to navigate directories. Additionally, it will convert the remote path into a local path (config name/.remote/) to 
    /// save downloaded files. Once converted, it will trigger the start_tracking method to track any modifications to files.
    pub fn connect(&mut self) -> Result<(), IOError>{
        self.ssh_m.connect()?;
        if !self.dir.path.set_path(self.ssh_m.run_command("pwd".to_string()).unwrap()) {
            println!("Error setting path");
        }
        let mut path = Remit::SystemPath::new();
        path.pushd(self.rclone_m.lock().unwrap().chosen_config.clone());
        path.pushd(".remote".to_string());
        println!("tracking local changes at: {}", path.get_windows_path());
        self.file_tracker.start_tracking(&mut path)?;
        return Ok(());
    }

    /// Stop tracking the local directory for changes and end the current ssh session
    /// TODO add error handling for stop tracking
    pub fn disconnect(&mut self) -> Result<(), IOError> {
        self.file_tracker.stop_tracking();
        return self.ssh_m.disconnect();
    }

    /// Load a list of files at the current remote directory
    /// 
    /// This method only needs to be called when the path has changed. It performs
    /// and parses an `stat .* *` in the current remote directory.
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

    /// Add a RemitConfiguration to the manager. When added to the manager, it will automatically be saved to disc
    /// 
    /// # Arguments
    /// * `config` - Contains the necessary information to save the remit configuration
    /// * `rclone_config` - If the rclone configuration should be different than the remit configuration, pass in a separate configuration
    /// here. Otherwise, the information will be taken from the config parameter
    pub fn add_config(&mut self, config: RemitConfig, rclone_config: Option<RemitConfig>) -> Result<(), IOError>{
        self.config_m.insert_config(config.clone());
        let rclone_arg = rclone_config.unwrap_or(config.clone());
        self.rclone_m.lock().unwrap().create_sftp_config(rclone_arg.name.clone(), rclone_arg.username.clone() ,rclone_arg.host.clone(), 
                                            Some(rclone_arg.password.clone()), None)?;
        return self.config_m.save_config(config.name.clone().as_str());
    }

    /// Downloads a remote file to the current disc. If the open flag is Some(true), attempt to open the file
    /// using explorer. The file must exist in the current remote directory.
    /// 
    /// # Arguments
    /// * `name` - Name of the file to download
    /// * `open` - Whether or not to attempt to open this file. If None or Some(false) is passed, don't attempt to open. Otherwise, try 
    /// to open it.
    pub fn download_file(&mut self, name: String, open: Option<bool>) -> Result<(), IOError>{
        let mut local_path = Remit::SystemPath::new();
        local_path.set_win_path(format!("{}\\.remote\\{}", self.rclone_m.lock().unwrap().chosen_config.clone(), self.dir.path.get_windows_path_local()));
        let mut remote_path = Remit::SystemPath::new();
        remote_path.set_win_path(self.dir.path.get_path());
        let r = self.rclone_m.lock().unwrap().download_remote_file(local_path.clone(), remote_path, name.clone()).unwrap();
        // on a success, if open is set then open the file using windows explorer ( allows a chance to set the default application)
        if r.success() {
            open.map(|open: bool| {
                if open {
                    local_path.pushd(name);
                    let _output = Command::new("explorer")
                    .arg(format!("{}\\{}", current_dir().unwrap().to_str().unwrap(), local_path.get_windows_path_local()))
                    .output();
                }
            });
            return Ok(());
        } else {
            return Err(IOError::new(IOErrorKind::Other, r.to_string()));
        }
    }

    /// Use rclone to upload a local file
    /// 
    /// This method uses the file name and the manager directory object to construct a file path on the local and remote system.
    /// Once constructed, it will pass the paths to rclone and trigger a sync
    /// # Arguments
    /// * `name` - Name of file to upload
    pub fn upload_file(&mut self, name: String) -> Result<(), IOError>{
        let r = self.rclone_m.lock().unwrap().upload_local_file(self.dir.path.clone(), self.dir.path.clone(), name.clone());
        if r?.success() {
            return Ok(());
        } else {
            return Err(IOError::new(IOErrorKind::Other, "error during upload"));
        }
    }

    /// Load a list of remit configurations
    pub fn get_configs(&mut self) -> Vec<Remit::Config>{
        return self.config_m.get_configs();
    }
}

}