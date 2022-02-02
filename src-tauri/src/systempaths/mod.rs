//! The SystemPath struct is used to track a path both in Linux and in Windows.
//! It can parse simple Windows/Linux paths. However, the most reliable way to use the
//! struct is to make changes one directory at a time. For example, if you are in the path
//! /home when you move to /home/username you would push "username" into the path.

pub mod rustssh {

use std::collections::VecDeque;
use std::string::String;
use std::iter::Iterator;

/// Used to track linux system paths through popping and pushing directories. It can convert
/// to windows paths although not very robust
#[derive(Clone, Debug)]
pub struct SystemPath {
    /// Each section of a path
    path: VecDeque<String>,
    /// The path queue converted into a linux path as a string
    path_str: String
}

#[allow(dead_code)]
impl SystemPath {
    /// Create a new empty path
    pub fn new() -> SystemPath{
        return SystemPath{path: VecDeque::new(),
                            path_str: String::new()};
    }

    /// Get the stored path as a string in Linux format. e.g. /home/username
    pub fn get_path(&self) -> String{
        return self.path_str.clone();
    }

    /// Convert the stored Linux path string to Windows by replacing / with \
    pub fn get_windows_path(&self) -> String {
        return self.get_path().replace("/", "\\");
    }

    /// If the first character is a backslash remove it. This occurs with aboslute paths
    /// For example, /home/homeadmin will convert to \home\homeadmin. This method changes that to
    /// home\homeadmin keeping it in the same relative directory
    pub fn get_windows_path_local(&self) -> String {
        let s = self.get_windows_path();
        if s.chars().next().unwrap() == '\\' {
            return s[1..s.len()].to_string();
        } else {
            return s;
        }
    }

    /// Creates a string from elements in the vec
    /// This method is ran after every popd and pushd call
    fn create_path_str(&mut self) {
        if self.path.len() == 0 {
            return self.path_str = "/".to_string();
        }
        let mut path_str = String::new();
        let slash_check = self.path[0].clone();
        if slash_check != "/".to_string() || self.path.len() == 1{
            path_str.push_str(&slash_check);
        }
        for i in 1..self.path.len() {
            path_str.push_str(&format!("/{}", self.path[i].clone()).to_string());
        }
        self.path_str = path_str;
    }

    /// Pop the last element in the queue. If the queue is empty
    /// a / will be inserted denoting the top directory in Linux
    /// 
    /// This method is used to go to the parent directory
    pub fn popd(&mut self) -> String{
        let r = self.path.pop_back();
        if self.path.len() == 0 {
            self.path.push_back("/".to_string());
        }
        self.create_path_str();
        return r.unwrap_or("".to_string());
    }

    /// Add a directory to the path.
    /// 
    /// This method is used to descend into directories
    /// # Arguments
    /// * `d` - Directory name to add to path
    pub fn pushd(&mut self, d: String) {
        if !self.check_should_add(d.clone()){
            return;
        }
        self.path.push_back(d.clone());
        self.create_path_str();
    }

    /// Checks if the incoming name is valid to add. The incoming string is valid if
    /// its length is greater than 1, it does not equal / or . and is not just spaces
    /// 
    /// # Arguments
    /// * `d` - String to check is valid to add to path
    fn check_should_add(&mut self, d: String) -> bool{
        return !(d.len() == 0 || d == "/" || d == "." || d.replace(" ", "").len() == 0);
    }

    /// Push this directory at the beginning of the path. For example,
    /// to go from username/.ssh to /home/username/.ssh
    /// 
    /// # Arguments
    /// * `d` - Directory to add to beginning of path
    pub fn prepd(&mut self, d: String) {
        if !self.check_should_add(d.clone()){
            return;
        }
        if self.path[0] == "/".to_string() {
            self.path[0] = d.clone();
            self.path.push_front("/".to_string());
        } else {
            self.path.push_front(d.clone());
        }
        self.create_path_str();  
    }

    /// Clear both path and vector data
    pub fn clear(&mut self) {
        self.path_str.clear();
        self.path.clear();
    }

    /// Parse a Linux path string
    /// 
    /// If the reconstruction of the parsed string differs
    /// from the input return false, else return true
    /// # Arguments
    /// * `path` - Linux path string to parse
    pub fn set_path(&mut self, path: String) -> bool{
        self.path.clear();
        if path.len() == 0 {
            return false;
        }
        if path.chars().next().unwrap() == '/' {
            self.path.push_back("/".to_string());
        }
        for p in path.split("/") {
            self.pushd(p.to_string());
        }
        self.create_path_str();
        return self.path_str == path;
    }

    /// Performs the same function as [`SystemPath::set_path`] except for a Windows style path string
    /// 
    /// # Arguments
    /// * `path` - Windows path to parse
    pub fn set_win_path(&mut self, path: String) -> bool {
        self.path.clear();
        if path.len() == 0 {
            return false;
        }
        if path.chars().next().unwrap() == '\\' {
            self.path.push_back("/".to_string());
        }
        for p in path.split("\\") {
            self.pushd(p.to_string());
        }
        self.create_path_str();
        return self.path_str == path;
    }
}

}