pub mod rustssh {

/// used to track linux system paths through popping and pushing directories. Can convert
/// to windows paths although not very robust
#[derive(Clone, Debug)]
pub struct SystemPath {
    path: Vec<String>,
    path_str: String
}

#[allow(dead_code)]
impl SystemPath {
    pub fn new() -> SystemPath{
        return SystemPath{path: Vec::new(),
                            path_str: String::new()};
    }

    /// get the created path string
    /// 
    /// this does not return the individual elements in the vector
    pub fn get_path(&mut self) -> String{
        return self.path_str.clone();
    }

    /// convert path string to windows by replacing / with \ and \\ with \
    pub fn get_windows_path(&mut self) -> String {
        return self.get_path().replace("/", "\\").replace("\\\\", "\\");
    }

    /// create string from elements in the vec::<string>
    /// ran after every popd and pushd
    fn create_path_str(&mut self) {
        if self.path.len() == 0 {
            self.path.push("/".to_string());
            return;
        }
        let mut path_str = String::new();
        path_str.push_str(&self.path[0].clone());
        for i in 1..self.path.len() {
            if self.path[i].len() > 0 {
                path_str.push_str(&format!("/{}", self.path[i].clone()).to_string());
            }
        }
        self.path_str = path_str;
    }

    /// pop the last element, unless empty. return
    /// 
    /// method is used to go up one directory
    pub fn popd(&mut self) -> String{
        if self.path.len() == 0 {
            return "/".to_string();
        }
        let s = self.path.pop().unwrap();
        self.create_path_str();
        return s;
    }

    /// push directory
    /// 
    /// used to descend into directories
    pub fn pushd(&mut self, d: String) {
        if d.len() == 0 {
            return;
        }
        self.path.push(d.clone());
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
            self.path.push("/".to_string());
        }
        for p in path.split("/") {
            self.path.push(p.to_string());
        }
        self.create_path_str();
        return self.path_str == path;
    }
}

}