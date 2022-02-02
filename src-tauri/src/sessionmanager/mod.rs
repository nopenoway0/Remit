//! Ths module is responsible for creating and managing ssh sessions. Through its manager class, remote commands can be executed to gather information
//! about the server. Although any command can be run through the run_command method, the main use of this class is to gather file and directory 
//! information to feed back to the front end. This is typically done through `stat`. To facilitate this information, directory information is stored
//! in the Directory object. A Directory object contains a list of files - represented by the RemitFile class - and a path.

pub mod rustssh {
use ssh2::*;
use std::net::TcpStream;
use std::io::Read;
use std::fmt::Debug;
use std::collections::BTreeMap;
use crate::*;

/// Denotes the RemitFile type
#[derive(Debug, Clone)]
pub enum FileType {
    /// A directory
    TypeDirectory,
    /// A file
    TypeFile,
    /// Denotes symlink type
    TypeLink,
    /// Unknown type
    TypeUnknown
}

/// Permission information for a file
/// 
/// Each file has 3 permissions objects - owner, group, other
#[derive(Debug, Clone)]
pub struct Permissions {
    pub write: bool,
    pub read: bool,
    pub exec: bool
}

impl Permissions {

    /// Create a new permissions object with the default permissions set to all false
    /// 
    /// Pass in a 3 character string to build a permission object with designated permissions. These permissions
    /// are expected as: ---. For example, r-x.
    /// # Arguments
    /// * `input` - If some string, parse the string and create a permissions object. Otherwise, just create a permissions object
    /// with all bits set to 0
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

/// SessionManager manages an ongoing ssh session.
pub struct SessionManager {
    /// An ongoing ssh session
    session: Session,
    /// A created user agent **not currently used**
    agent: Option<Agent>,
    /// username
    user: String,
    /// password
    pass: String,
    //destination
    url: String
}

/// Represents information about a file. Can be directory, file, symlink or other
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// file name
    pub name: String,
    /// size in bytes
    pub size: u64,
    /// Type of file
    pub file_type: FileType,
    /// Group permissions
    pub group: Permissions,
    /// Owner permissions
    pub owner: Permissions,
    /// Other permissions
    pub other: Permissions
}

impl FileInfo {
    /// Construct a new fileinfo object with empty name, 0 bytes, and all permissions false
    pub fn new() -> FileInfo{
        return FileInfo {name: String::new(),
                        size: 0u64,
                        file_type: FileType::TypeUnknown,
                        group: Permissions::new(None), 
                        owner: Permissions::new(None),
                        other: Permissions::new(None)}
    }
}

/// A loaded file that contains information about a file
#[derive(Debug, Clone)]
pub struct RemitFile {
    pub info: FileInfo,
}

impl RemitFile {
    /// Create a new empty RemitFile
    pub fn new() -> RemitFile {
        return RemitFile{info: FileInfo::new()};
    }

    /// Create a RemitFile object from the supplied parameters
    /// # Arguments
    /// * `name` - Name of the file
    /// * `size` - Optional file size. If none passed, assume 0
    /// * `file_type` - Type of file. If none is passed create using [`FileType::TypeUnknown`]
    pub fn new_populated(name: String, size: Option<u64>, file_type: Option<FileType>) -> RemitFile {
        let mut info = FileInfo::new();
        info.name = name;
        info.size = size.unwrap_or(0);
        info.file_type = file_type.unwrap_or(FileType::TypeUnknown);
        return RemitFile{info: info};
    }
}

/// represents a directory and its contents
/// 
/// files: all files stored in order of appearance in the ls command by their name
/// path: path to this directory
#[derive(Debug, Clone)]
pub struct Directory {
    pub files: BTreeMap<String, RemitFile>,
    pub path: Remit::SystemPath
}

#[allow(dead_code)]
impl Directory {

    /// Clear path and all file contents
    pub fn clear(&mut self) {
        self.path.clear();
        self.files.clear();
    }

    /// Construct a new directory either empty or parse a the incoming string
    /// # Arguments
    /// * `str_input` - the results of the stat command. Pass in None to create an empty directory
    pub fn new(str_input: Option<String>) -> Directory{
        let mut dir = Directory{files: BTreeMap::new(), path: Remit::SystemPath::new()};
        if str_input.is_some() {
            dir.files = Directory::parse_string(str_input.unwrap());
        }
        return dir;
    }

    /// Parse a string to create the file structure in a directory
    /// # Arguments
    /// * `input` - The output of a `stat .* * --printf='Name: %n\\nPermissions: %a\\nSize: %s\\nType: %F\\n\\n` command
    fn parse_string(input: String) -> BTreeMap<String,RemitFile>{
        let mut files: BTreeMap<String, RemitFile> = BTreeMap::new();
        // default structure including . and ..
        let current = RemitFile::new_populated(".".to_string(), None, Some(FileType::TypeDirectory));
        let updir = RemitFile::new_populated("..".to_string(), None, Some(FileType::TypeDirectory));
        files.insert(current.info.name.clone(), current);
        files.insert(updir.info.name.clone(), updir);

        let chunks = input.split("\n\n");
        for chunk in chunks {
            let mut f = RemitFile::new();
            let mut lines = chunk.split("\n");
            // get name
            let name_line = lines.next().unwrap();
            f.info.name = name_line.chars().skip(6).collect();
            // skip permissions for now
            if lines.next().is_none() {
                continue;
            };
            // get size;
            let size_line = lines.next().unwrap();
            let size_str:String = size_line.chars().skip(6).collect();
            f.info.size = str::parse::<u64>(size_str.as_str()).unwrap();
            // get file type
            let file_type_line = lines.next().unwrap();
            let file_type_str: String = file_type_line.chars().skip(6).collect();
            f.info.file_type = Directory::parse_file_type(file_type_str.as_str());

            // pop name handle spaces?
            files.insert(f.info.name.clone(), f);
        }
        return files;
    }

    /// #DEPRECATED
    fn parse_permissions_string(f: &mut RemitFile, input: String) {
        f.info.file_type = Directory::parse_file_type(&input);
        f.info.group = Permissions::new(Some(input[1..4].to_string()));
        f.info.owner = Permissions::new(Some(input[4..7].to_string()));
        f.info.other = Permissions::new(Some(input[7..10].to_string()));
    }


    /// Convert the incoming string type into the proper enum. 
    /// For example, `link` will get converted to [`FileType::TypeLink`]
    /// # Arguments
    /// * `input` - String to be converted into enum
    fn parse_file_type(input: &str) -> FileType{
        let filetype: FileType;
        match input {
            "link"=> filetype = FileType::TypeLink,
            "directory"=> filetype = FileType::TypeDirectory,
            "regular file"|"regular empty file"=> filetype = FileType::TypeFile,
            _=> filetype = FileType::TypeUnknown
        }
        return filetype;
    }
}

#[allow(dead_code)]
impl SessionManager {

    /// Set the session manager parameters
    /// # Arguments
    /// * `user` - Username. If none, assume empty
    /// * `pass` - Password. If none, assume empty
    /// * `url` - Host endpoint. If none assume empty
    pub fn set_params(&mut self, user: Option<String>, pass: Option<String>, url: Option<String>) {
        self.url = url.unwrap_or("".to_string());
        self.user = user.unwrap_or("".to_string());
        self.pass = pass.unwrap_or("".to_string());
    }

    /// Create a new session manager with the designated parameters
    /// # Arguments
    /// * `user` - Username. If none, assume empty
    /// * `pass` - Password. If none, assume empty
    /// * `url` - Host endpoint. If none assume empty
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

    /// End the current ssh session
    pub fn disconnect(&mut self) -> Result<(), IOError>{
        match self.session.disconnect(Some(ssh2::DisconnectCode::ByApplication), "disconnect requested by app", None) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(IOError::new(IOErrorKind::Other, e))
        }
    }

    /// Connect using the already set parameters
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

    /// Method that starts session agent. **Does not currently work and is not used**
    pub fn start_agent(&mut self) -> Result<(), Error>{
        self.agent = Some(self.session.agent().unwrap());
        let agent = self.agent.as_mut();
        let r = agent.unwrap().connect();
        return r;
    }

    /// Run an ssh command on the remote machine
    /// # Arguments
    /// * `command` - Command to be ran on the remote machine
    pub fn run_command(&mut self, command: String) -> Result<String, Error>{
        let mut channel: ssh2::Channel = self.session.channel_session()?;
        channel.exec(command.as_str())?;
        let mut s = String::new();
        let _r = channel.read_to_string(&mut s);
        return Ok(s.trim_end().to_string());
    }

    /// Load the file contents of the directory into that directory
    /// 
    /// Parses a stat command - see source for full command - at the directory's path. This will store file information
    /// into the directory object
    /// # Arguments
    /// * `d` - Directory to store file contents
    pub fn get_directory(&mut self, d: &mut Directory) -> Result<(), Error>{
        println!("cd {} && stat .* --printf='Name: %n\\nPermissions: %a\\nSize: %s\\nType: %F\\n\\n'", d.path.get_path());
        let dir_str = self.run_command(format!("(cd {} && stat .* * --printf='Name: %n\\nPermissions: %a\\nSize: %s\\nType: %F\\n\\n')", d.path.get_path()))?;
        d.files = Directory::parse_string(dir_str);
        return Ok(());
    }

    /// Push the name onto the directory path essentially "navigating" to that path.
    /// 
    /// This method does not update the file contents, only modifies the path. Additionally, before navigating, the
    /// method will check if the name exists in the current directory and it is a file. Therefore, symlinks are not currently supported.
    /// # Arguments
    /// * `d` - Directory to push name into
    /// * `name` - Name of directory to navigate to
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
}
}