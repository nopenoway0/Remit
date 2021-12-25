
/// Contains the backend for the remit application
/// 
/// main.rs contains bindngs to perform all io and system operations. This file starts the application as well
/// as providing an api for the frontend to communicate with file systems. functions that are prepended
/// with the tauri::command macro are available for the rendered web view to hook. The other parts of the file
/// are set up and configuration

//#![windows_subsystem = "windows"]

use tauri::{plugin::{Plugin, Result as PluginResult}, Runtime, PageLoadPayload, Window, Invoke, AppHandle};
use std::{sync::Mutex, sync::MutexGuard};
use once_cell::sync::Lazy;
use std::io::Error;
use std::path::Path;
use std::collections::HashMap;
mod configmanager;
mod sessionmanager;
mod syncmanager;
mod systempaths;
mod manager;

/// Mutex controlled Manager. Used to make all api calls in the backend
/// once control is given
type ApiRef<'a> = MutexGuard<'a, manager::rustssh::Manager>;

/// Api that contains global state of the program
static REMIT_API: Lazy<Mutex<manager::rustssh::Manager>> = Lazy::new(|| {
  let manager = manager::rustssh::Manager::new_empty().unwrap();
  return Mutex::new(manager);
});

struct Remit<R: Runtime> {
    invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
    // plugin state, configuration fields
  }


  /// Helper method to grab and use the globa Manager
  fn run_api_command<T>(output:&mut T, callback: &dyn Fn(&mut T, &mut ApiRef) -> Result<(), std::io::Error> ) -> Result<(), String> {
    match REMIT_API.lock() {
      Ok(mut api)=>{
        match callback(output, &mut api) {
          Ok(_)=>return Ok(()),
          Err(e)=> return Err(e.to_string())
        }
      }
      Err(e)=> return Err(e.to_string())
    }
  }

  /// Download file
  #[tauri::command]
  async fn download(filename: String, open: Option<bool>) -> Result<(), String> {
    let mut var: u32 = 0;
    let r = run_api_command::<u32>(&mut var, &|_output: &mut u32, api: &mut ApiRef| -> Result<(), std::io::Error>{
      api.download_file(filename.clone(), open)?;
      return Ok(());
    })?;
    return Ok(r);
  }

  /// Push filename/directory in global api
  #[tauri::command]
  async fn pushd(d: String) -> Result<(), String> {
    run_api_command::<String>(&mut d.clone(), &|d: &mut String, api: &mut ApiRef| -> Result<(), Error> {
      api.navigate(d.clone())?;
      return Ok(());
    })?;
    return Ok(());
  }

  /// List all files an directories at current path
  #[tauri::command]
  async fn list_current_directory() -> Result<Vec<HashMap<String, String>>, String> {
    let mut filenames: Vec<HashMap<String, String>> = Vec::new();
    run_api_command::<Vec::<HashMap::<String,String>>>(&mut filenames, &|filenames: &mut Vec<HashMap<String, String>>, api: &mut ApiRef| -> Result<(), Error>{
      api.get_directory()?;
      for entry in &api.dir.files {
        let mut file = HashMap::<String,String>::new();
        file.insert("name".to_string(), entry.0.clone());
        file.insert("type".to_string(), format!("{:?}", entry.1.info.file_type));
        file.insert("size".to_string(), entry.1.info.size.to_string());
        filenames.push(file);
      }
      return Ok(());
    })?;
    return Ok(filenames);
  }

  // the plugin custom command handlers if you choose to extend the API.
  #[tauri::command]
  async fn connect(username: String, host: String, port: String, password: String) -> Result<(), String> {
    let mut fields = vec![host, username, password, port];
    let _r = run_api_command::<Vec::<String>>(&mut fields, &|fields: &mut Vec::<String>, api: &mut ApiRef|-> Result<(), std::io::Error>{
      api.set_params(fields[0].clone(), fields[1].clone(), Some(fields[2].clone()), Some("default_remitconfig".to_string()), None, Some(fields[3].clone()))?;
      api.connect()?;
      return Ok(());
    })?;
    return Ok(());
  }
  
  // check for dependencies, not working
  #[tauri::command]
  async fn check_dependencies() -> Result<(), String> {
    match Path::new("rclone.exe").is_file() {
      true=>return Ok(()),
      false=>return Err("missing dependencies".to_string())
    }
  }

  // get ssh config files
  #[tauri::command]
  async fn get_config_names() -> Result<Vec<HashMap<String, String>>, String> {
    let mut configs = Vec::<HashMap::<String,String>>::new();
    run_api_command::<Vec::<HashMap::<String,String>>>(&mut configs, &|json: &mut Vec::<HashMap::<String,String>>,api: &mut ApiRef| -> Result<(), std::io::Error> {
      //let mut json = Vec::<HashMap<String, String>>::new();
      for c in api.get_configs() {
        let mut config_json = HashMap::<String,String>::new();
        config_json.insert("name".to_string(), c.name);
        config_json.insert("port".to_string(), c.port);
        config_json.insert("pass".to_string(), c.password);
        config_json.insert("host".to_string(), c.host);
        config_json.insert("user".to_string(), c.username);
        json.push(config_json);
      }
      return Ok(());
    })?;
    return Ok(configs);
  }

  /// disconnect current ssh session
  #[tauri::command]
  fn disconnect() -> Result<(), String>{
    let mut var = 1u8;
    run_api_command::<u8>(&mut var, &|_var: &mut u8, api:&mut ApiRef| -> Result<(), std::io::Error> {
      api.disconnect()?;
      return Ok(());
    })?;
    return Ok(());
  }

  impl<R: Runtime> Remit<R> {
    // you can add configuration fields here,
    // see https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
    pub fn new() -> Self {
      Self {
        invoke_handler: Box::new(tauri::generate_handler![connect,disconnect, 
                                                          check_dependencies, 
                                                          get_config_names,
                                                          list_current_directory,
                                                          pushd, download]),
      }
    }
  }
  
  impl<R: Runtime> Plugin<R> for Remit<R> {
    /// The plugin name. Must be defined and used on the `invoke` calls.
    fn name(&self) -> &'static str {
      "Remit"
    }


    /// The JS script to evaluate on initialization.
    /// Useful when your plugin is accessible through `window`
    /// or needs to perform a JS task on app initialization
    /// e.g. "window.awesomePlugin = { ... the plugin interface }"
    fn initialization_script(&self) -> Option<String> {
      None
    }
  
    /// initialize plugin with the config provided on `tauri.conf.json > plugins > $yourPluginName` or the default value.
    fn initialize(&mut self, _app: &AppHandle<R>, _config: serde_json::Value) -> PluginResult<()> {
      Ok(())
    }
  
    /// Callback invoked when the Window is created.
    fn created(&mut self, _window: Window<R>) {}
  
    /// Callback invoked when the webview performs a navigation.
    fn on_page_load(&mut self, _window: Window<R>, _payload: PageLoadPayload) {}

    /// Extend the invoke handler.
    fn extend_api(&mut self, message: Invoke<R>) {
      (self.invoke_handler)(message)
    }
  }


  fn main() {
    let remit = Remit::new();
    tauri::Builder::default()
        .plugin(remit)
        .setup(|_app| {
            // listen to the `event-name` (emitted on any window)
            //let id = app.listen_global("login", |event| {
            //    println!("got event-name with payload {:?}", "");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running application");
}