#[allow(dead_code)]
pub trait Storage {
    fn save_progress(&self, data: &str);
    fn load_progress(&self) -> Option<String>;
}

// future: file/db implementations will go here
