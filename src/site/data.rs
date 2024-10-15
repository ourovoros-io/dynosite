/// Represents the data folder in the site
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Data {
    pub root_folder: std::path::PathBuf,
    pub executions: Vec<crate::site::execution::Execution>,
}

impl Data {
    /// Creates a new data folder in the site
    pub fn new(root_folder: std::path::PathBuf) -> Self {
        if !root_folder.exists() {
            std::fs::create_dir_all(&root_folder).unwrap();
        }

        Self {
            root_folder,
            executions: Vec::new(),
        }
    }
}
