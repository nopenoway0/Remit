
pub mod rustssh {
use crate::systempaths::rustssh::SystemPath;
use ssh2::*;
use std::net::TcpStream;
use std::io::Read;
use std::fmt::Debug;
use std::collections::BTreeMap;

type IOError = std::io::Error;
type IOErrorKind = std::io::ErrorKind;

/// denotes the type of file
#[derive(Debug, Clone)]
pub enum FileType {
    TypeDirectory,
    TypeFile,
    TypeLink,
    TypeUnknown
}

/// denotes permission for a file or directory
/// 
/// each file has 3 permissions objects - owner, group, other
#[derive(Debug, Clone)]
pub struct Permissions {
    pub write: bool,
    pub read: bool,
    pub exec: bool
}

impl Permissions {

    /// create a new permissions object with the default permissions set to all false
    /// 
    /// pass in a 3 character string to build a permission object with designated permissions
    /// string should be as it appears in a ls -a command. for example rwx or r--
    pub fn new(input: Option<String>) -> Permissions {
        if input.is_none() {
            return Permissions{write: false, read: false, exec: false};
        }
        let perm_str = input.unwrap();
        let mut p = Permissions::new(None);
        p.read = perm_str.chars().nth(0).unwrap() == 'r';
        p.write = perm_str.chars().nth(1).unwrap() == 'w';
        p.exec = perm_str.chars().nth(2).unwrap() == 'x';
        return p;
    }
}

/// ssh manager makes and manages an ssh connection
pub struct SessionManager {
    session: Session,
    agent: Option<Agent>,
    user: String,
    pass: String,
    url: String
}

/// contains information for the file it belongs to
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    /// size in bytes
    pub size: u64,
    pub file_type: FileType, // delete this?
    pub group: Permissions,
    pub owner: Permissions,
    pub other: Permissions
}

impl FileInfo {
    /// construct a new fileinfo object with empty name, 0 bytes, and all permissions false
    pub fn new() -> FileInfo{
        return FileInfo {name: String::new(),
                        size: 0u64,
                        file_type: FileType::TypeUnknown,
                        group: Permissions::new(None), 
                        owner: Permissions::new(None),
                        other: Permissions::new(None)}
    }
}

/// contains fileinfo and type
#[derive(Debug, Clone)]
pub struct RemitFile {
    pub info: FileInfo,
}

/// representation of a file to be consumed by Remit
impl RemitFile {
    pub fn new() -> RemitFile {
        return RemitFile{info: FileInfo::new()};
    }
}

/// represents a directory and its contents
/// 
/// files: all files stored in order of appearance in the ls command by their name
/// path: path to this directory
#[derive(Debug, Clone)]
pub struct Directory {
    pub files: BTreeMap<String, RemitFile>,
    pub path: SystemPath
}

#[allow(dead_code)]
impl Directory {

    /// clear path and file contents
    pub fn clear(&mut self) {
        self.path.clear();
        self.files.clear();
    }

    /// construct a new directory. It will be empty unless an string, obtained by running ls -al, is passed
    /// to it
    pub fn new(str_input: Option<String>) -> Directory{
        let mut dir = Directory{files: BTreeMap::new(), path: SystemPath::new()};
        if str_input.is_some() {
            dir.files = Directory::parse_string(str_input.unwrap());
        }
        return dir;
    }

    /// parse ls -al output storing file name, permissions, size and type
    fn parse_string(input: String) -> BTreeMap<String,RemitFile>{
        let mut files: BTreeMap<String, RemitFile> = BTreeMap::new();
        let lines = input.lines();
        for line in lines.skip(1) {
            let mut components = line.split_whitespace();
            let mut f = RemitFile::new();
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

    /// parses a permission string given by ls -al
    /// expects format of drwxrwxrwx
    fn parse_permissions_string(f: &mut RemitFile, input: String) {
        f.info.file_type = Directory::parse_file_type(&input);
        f.info.group = Permissions::new(Some(input[1..4].to_string()));
        f.info.owner = Permissions::new(Some(input[4..7].to_string()));
        f.info.other = Permissions::new(Some(input[7..10].to_string()));
    }


    /// takes the first element of a permission string acquired by ls -al
    /// l -> link
    /// d -> directory
    /// - -> file
    /// anything else -> unknown
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

#[allow(dead_code)]
impl SessionManager {

    pub fn set_params(&mut self, user: Option<String>, pass: Option<String>, url: Option<String>) {
        self.url = url.unwrap_or("".to_string());
        self.user = user.unwrap_or("".to_string());
        self.pass = pass.unwrap_or("".to_string());
    }

    pub fn new(user: Option<String>, pass: Option<String>, url: Option<String>) -> Result<SessionManager, Error> {
        let session = Session::new()?;
        let username: String = user.unwrap_or("".to_string());
        let password: String = pass.unwrap_or("".to_string());
        let url_str: String = url.unwrap_or("".to_string());
        let manager: SessionManager = SessionManager{session: session,
                                                     user: username,
                                                     pass: password,
                                                     agent: None,
                                                     url: url_str
                                                    };
        return Ok(manager);
    }

    pub fn disconnect(&mut self) -> Result<(), IOError>{
        match self.session.disconnect(Some(ssh2::DisconnectCode::ByApplication), "disconnect requested by app", None) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(IOError::new(IOErrorKind::Other, e))
        }
    }

    /// connect using the set parameters. 
    /// 
    /// Currently only supports username and pass. TODO support keyfile
    pub fn connect(&mut self) -> Result<(), IOError>{
        let connection = TcpStream::connect(self.url.as_str())?;
        self.session = Session::new()?;
        self.session.set_tcp_stream(connection);
        self.session.handshake()?;
        self.session.userauth_password(self.user.as_str(), self.pass.as_str())?;
        return Ok(());
    }

    /// unused method that starts session agent. may be useful for keyfiles
    pub fn start_agent(&mut self) -> Result<(), Error>{
        self.agent = Some(self.session.agent().unwrap());
        let agent = self.agent.as_mut();
        let r = agent.unwrap().connect();
        return r;
    }

    /// run ssh command through current connection
    pub fn run_command(&mut self, command: String) -> Result<String, Error>{
        let mut channel: ssh2::Channel = self.session.channel_session()?;
        channel.exec(command.as_str())?;
        let mut s = String::new();
        let _r = channel.read_to_string(&mut s);
        return Ok(s.trim_end().to_string());
    }

    /// get file contents of the current directory using the directory's path
    /// 
    /// Parses an ls -al command at the directory's path. This will store file information
    /// into the directory object
    pub fn get_directory(&mut self, d: &mut Directory) -> Result<(), Error>{
        let dir_str = self.run_command(format!("ls -al {}", d.path.get_path()))?;
        d.files = Directory::parse_string(dir_str);
        return Ok(());
    }

    /// modify the directory's path by pushing the name string
    /// 
    /// Does not update the file contents, only modifies the path
    pub fn navigate(&mut self, d: &mut Directory, name: String) -> Result<(), IOError>{
        // check if we should pop
        if name == ".." {
            d.path.popd();
            return Ok(());
        }
        // error handling
        let file = &d.files.get(&name).ok_or(IOError::new(IOErrorKind::NotFound, "Directory not found"))?;
        match file.info.file_type {
            FileType::TypeDirectory=> {
                d.path.pushd(name);
                return Ok(());
            },
            _=>return Err(IOError::new(IOErrorKind::Other, "selection not a directory"))
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