pub mod rustssh {

#[derive(Clone, Debug)]
pub struct SystemPath {
    path: Vec<String>,
    path_str: String
}

impl SystemPath {
    pub fn new() -> SystemPath{
        return SystemPath{path: Vec::new(),
                            path_str: String::new()};
    }

    pub fn get_path(&mut self) -> String{
        return self.path_str.clone();
    }

    pub fn get_windows_path(&mut self) -> String {
        return self.get_path().replace("/", "\\").replace("\\\\", "\\");
    }

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

    pub fn popd(&mut self) -> String{
        if self.path.len() == 0 {
            return "/".to_string();
        }
        let s = self.path.pop().unwrap();
        self.create_path_str();
        return s;
    }

    pub fn pushd(&mut self, d: String) {
        if d.len() == 0 {
            return;
        }
        self.path.push(d.clone());
        self.create_path_str();
    }

    pub fn clear(&mut self) {
        self.path_str.clear();
        self.path.clear();
    }

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