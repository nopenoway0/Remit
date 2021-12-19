
pub mod rustssh {
use crate::systempaths::rustssh::SystemPath;
use ssh2::*;
use std::net::TcpStream;
use std::io::Read;
use std::fmt::Debug;
use std::fmt::Display;
use std::collections::HashMap;
use std::process::Command;
use std::io::stdout;
use std::io::stderr;
use std::io::Stdout;
use std::io::Stderr;
use std::collections::BTreeMap;


#[derive(Debug, Clone)]
pub enum FileType {
    TypeDirectory,
    TypeFile,
    TypeLink,
    TypeUnknown
}

#[derive(Debug, Clone)]
pub struct Permissions {
    pub write: bool,
    pub read: bool,
    pub exec: bool
}

impl Permissions {

    pub fn new(input: Option<String>) -> Permissions {
        if input.is_none() {
            return Permissions{write: false, read: false, exec: false};
        }
        let perm_str = input.unwrap();
        let mut p = Permissions::new(None);
        let length = 3;
        p.read = perm_str.chars().nth(0).unwrap() == 'r';
        p.write = perm_str.chars().nth(1).unwrap() == 'w';
        p.exec = perm_str.chars().nth(2).unwrap() == 'x';
        return p;
    }
}

pub struct SessionManager {
    session: Session,
    agent: Option<Agent>,
    user: String,
    pass: String,
    url: String
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String, // name of file
    pub size: u64, // in bytes
    pub file_type: FileType,
    pub group: Permissions,
    pub owner: Permissions,
    pub other: Permissions
}

impl FileInfo {
    pub fn new() -> FileInfo{
        return FileInfo {name: String::new(),
                        size: 0u64,
                        file_type: FileType::TypeUnknown,
                        group: Permissions::new(None), 
                        owner: Permissions::new(None),
                        other: Permissions::new(None)}
    }
}

#[derive(Debug, Clone)]
pub struct SRustFile {
    pub info: FileInfo,
    pub filetype: FileType
}

impl SRustFile {
    pub fn new() -> SRustFile {
        return SRustFile{info: FileInfo::new(), filetype: FileType::TypeUnknown};
    }
}

#[derive(Debug, Clone)]
pub struct Directory {
    pub files: BTreeMap<String, SRustFile>,//HashMap<String, SRustFile>,
    pub path: SystemPath
}

impl Directory {

    pub fn clear(&mut self) {
        self.path.clear();
        self.files.clear();
    }

    pub fn new(str_input: Option<String>) -> Directory{
        let mut dir = Directory{files: BTreeMap::new()/*HashMap::new()*/, path: SystemPath::new()};
        if str_input.is_some() {
            dir.files = Directory::parse_string(str_input.unwrap());
        }
        return dir;
    }

    // parse ls -al output
    pub fn parse_string(input: String) -> BTreeMap<String,SRustFile>{
        let mut files: BTreeMap<String, SRustFile> = BTreeMap::new();
        let lines = input.lines();
        for line in lines.skip(1) {
            let mut components = line.split_whitespace();
            let mut f = SRustFile::new();
            // 0 component is permissions string e.g. drwxr-xr-x
            Directory::parse_permissions_string(&mut f, components.nth(0).unwrap().to_string());
           
            // pop type number
            components.nth(0).unwrap();

            // pop group
            components.nth(0).unwrap();
            
            // pop owner
            components.nth(0).unwrap();

            // pop size
            f.info.size = str::parse::<u64>(components.nth(0).unwrap()).unwrap();

            // pop month
            components.nth(0).unwrap();

            // pop day
            components.nth(0).unwrap();

            //pop year
            components.nth(0).unwrap();

            // pop name handle spaces?
            f.info.name = components.nth(0).unwrap().to_string();
            files.insert(f.info.name.clone(), f);
        }
        return files;
    }

    // throw error if length isn't 10
    // expects format of drwxrwxrwx
    fn parse_permissions_string(f: &mut SRustFile, input: String) {
        f.filetype = Directory::parse_file_type(&input);
        f.info.group = Permissions::new(Some(input[1..4].to_string()));
        f.info.owner = Permissions::new(Some(input[4..7].to_string()));
        f.info.other = Permissions::new(Some(input[7..10].to_string()));
    }



    fn parse_file_type(input: &String) -> FileType{
        let filetype: FileType;
        match input.chars().nth(0).unwrap() {
            'l'=> filetype = FileType::TypeLink,
            'd'=> filetype = FileType::TypeDirectory,
            '-'=> filetype = FileType::TypeFile,
            _=> filetype = FileType::TypeUnknown
        }
        return filetype;
    }
}

impl SessionManager {

    pub fn set_params(&mut self, user: Option<String>, pass: Option<String>, url: Option<String>) {
        let mut username: String = "".to_string();
        let mut password: String = "".to_string();
        let mut url_str: String = "".to_string();
        if user.is_some() {
            username = user.unwrap();
        }
        if pass.is_some() {
            password = pass.unwrap();
        }
        if url.is_some() {
            url_str = url.unwrap();
        }  
        self.url = url_str;
        self.user = username;
        self.pass = password;
    }

    pub fn new(user: Option<String>, pass: Option<String>, url: Option<String>) -> SessionManager {
        let mut session: Session;
        {
            let r = Session::new();
            if r.is_err() {
                // throw error here
            }
            session = r.unwrap();
        }
        let mut username: String = "".to_string();
        let mut password: String = "".to_string();
        let mut url_str: String = "".to_string();
        if user.is_some() {
            username = user.unwrap();
        }
        if pass.is_some() {
            password = pass.unwrap();
        }
        if url.is_some() {
            url_str = url.unwrap();
        }
        let manager: SessionManager = SessionManager{session: session,
                                                     user: username,
                                                     pass: password,
                                                     agent: None,
                                                     url: url_str
                                                    };
        return manager;
    }

    pub fn disconnect(&mut self) -> Result<(), std::io::Error>{
        match self.session.disconnect(Some(ssh2::DisconnectCode::ByApplication), "disconnect requested by app", None) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e))
        }
    }

    pub fn connect(&mut self) -> Result<(), std::io::Error>{
        let tcp_status = TcpStream::connect(self.url.as_str());
        // check connection
        match tcp_status {
            Ok(connection) => {
                match Session::new() {
                    Ok(session) => {
                        self.session = session;
                    }
                    Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e))
                }
                self.session.set_tcp_stream(connection);
                match self.session.handshake() {
                    Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, e)),
                    _=>{
                    }
                }
                match self.session.userauth_password(self.user.as_str(), self.pass.as_str()) {
                    Ok(()) => return Ok(()),
                    Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, e))
                }
            },
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::ConnectionReset, e))
        };
    }

    pub fn start_agent(&mut self) -> Result<(), Error>{
        self.agent = Some(self.session.agent().unwrap());
        let agent = self.agent.as_mut();
        let r = agent.unwrap().connect();
        return r;
    }

    pub fn run_command(&mut self, command: String) -> Result<String, Error>{
        let mut channel: ssh2::Channel = self.session.channel_session()?;
        channel.exec(command.as_str())?;
        let mut s = String::new();
        let _r = channel.read_to_string(&mut s);
        return Ok(s.trim_end().to_string());
    }

    pub fn get_directory(&mut self, d: &mut Directory) -> Result<(), Error>{
        let dir_str = self.run_command(format!("ls -al {}", d.path.get_path()))?;
        d.files = Directory::parse_string(dir_str);
        return Ok(());
    }

    pub fn navigate(&mut self, d: &mut Directory, name: String) -> Result<(), std::io::Error>{
        // check if we should pop
        if name == ".." {
            d.path.popd();
            self.get_directory(d)?;
            return Ok(());
        }
        // error handling
        let check = &d.files.get(&name);
        if check.is_none() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Could not find directory in path"));
        }
        let file = check.unwrap();
        match file.filetype {
            FileType::TypeDirectory=> {
                d.path.pushd(name);
                return Ok(());
            },
            _=>return Err(std::io::Error::new(std::io::ErrorKind::Other, "selection not a directory"))
        }
    }

    fn add_trailing_slash(path:&mut String) {
        if path.chars().nth(path.len() - 1).unwrap() != '/' {
            path.push('/');
        }
    }

    /*pub fn sync_local_file(&mut self, d: &Directory, name: String) {
        let mut path = d.path.clone();
        SessionManager::add_trailing_slash(&mut path);
        println!("{}", path);
        let output = Command::new("rclone.exe")
                .arg("sync")
                .arg(format!("./{}", name))
                .arg(format!("{}{}", "test:", path))
                .output()
                .expect("couldnt run");
        println!("{:?}", output);
    }*/
}
}