pub mod rustssh {

use std::collections::VecDeque;
use std::string::String;
use std::iter::Iterator;

/// used to track linux system paths through popping and pushing directories. Can convert
/// to windows paths although not very robust
#[derive(Clone, Debug)]
pub struct SystemPath {
    path: VecDeque<String>,
    path_str: String
}

#[allow(dead_code)]
impl SystemPath {
    pub fn new() -> SystemPath{
        return SystemPath{path: VecDeque::new(),
                            path_str: String::new()};
    }

    /// get the created path string
    /// 
    /// this does not return the individual elements in the vector
    pub fn get_path(&self) -> String{
        return self.path_str.clone();
    }

    /// convert path string to windows by replacing / with \ and \\ with \
    pub fn get_windows_path(&self) -> String {
        return self.get_path().replace("/", "\\");
    }

    pub fn get_windows_path_local(&self) -> String {
        let s = self.get_windows_path();
        if s.chars().next().unwrap() == '\\' {
            return s[1..s.len()].to_string();
        } else {
            return s;
        }
    }

    /// create string from elements in the vec::<string>
    /// ran after every popd and pushd
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

    /// pop the last element, unless empty. return
    /// 
    /// method is used to go up one directory
    pub fn popd(&mut self) -> String{
        let r = self.path.pop_back();
        if self.path.len() == 0 {
            self.path.push_back("/".to_string());
        }
        self.create_path_str();
        return r.unwrap_or("".to_string());
    }

    /// push directory
    /// 
    /// used to descend into directories
    pub fn pushd(&mut self, d: String) {
        if !self.check_should_add(d.clone()){
            return;
        }
        self.path.push_back(d.clone());
        self.create_path_str();
    }

    fn check_should_add(&mut self, d: String) -> bool{
        return !(d.len() == 0 || d == "/" || d == "." || d.replace(" ", "").len() == 0);
    }

    /// push directory at beginning of vec
    /// 
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

    /// clear both path and vector data
    pub fn clear(&mut self) {
        self.path_str.clear();
        self.path.clear();
    }

    /// parse a path string
    /// 
    /// if the reconstruction of the parsed string differs
    /// from the input return false, else return true
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