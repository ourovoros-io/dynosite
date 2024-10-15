use std::path::PathBuf;

use super::{
    data::Data,
    execution::Execution,
    html::{generate, generate_error_page},
};
use crate::{error::Result, wrap};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DynoSite {
    pub root_folder: PathBuf,
    pub data_folder: PathBuf,
    pub index_html: PathBuf,
    pub error_html: PathBuf,
    pub data: Data,
}

impl DynoSite {
    /// Initialize a new site either from a file or from scratch
    pub fn init(site_name: &str) -> Result<Self> {
        let site_json = PathBuf::from(site_name).with_extension("json");
        let site = Self::new(site_name);

        if site.root_folder.join(&site_json).exists() {
            Ok(serde_json::from_str(
                std::fs::read_to_string(
                    site.root_folder
                        .join(site_json)
                        .display()
                        .to_string()
                        .as_str(),
                )
                .map_err(|e| wrap!(e.into()))?
                .as_str(),
            )
            .map_err(|e| wrap!(e.into()))?)
        } else {
            Ok(site)
        }
    }

    /// Create a new site with default values
    pub fn new(site_name: &str) -> Self {
        Self {
            root_folder: PathBuf::from(site_name),
            data_folder: PathBuf::from(site_name).join("data"),
            index_html: PathBuf::from(site_name).join("index.html"),
            error_html: PathBuf::from(site_name).join("error.html"),
            data: Data::new(PathBuf::from(site_name).join("data")),
        }
    }

    /// Add an execution to the site
    pub fn add_execution(&mut self, execution: &Execution) -> Result<()> {
        self.data.executions.push(execution.clone());

        Self::generate_html(self).map_err(|e| wrap!(e))?;

        Ok(())
    }

    /// Write the site to a file
    pub fn store(&self) -> Result<()> {
        let site = serde_json::to_string(&self).map_err(|e| wrap!(e.into()))?;
        std::fs::write(self.root_folder.join("site.json"), site).map_err(|e| wrap!(e.into()))?;
        Ok(())
    }

    /// Generate the HTML for the site and write the files to disk
    pub fn generate_html(site: &DynoSite) -> Result<()> {
        // Generate the index HTML
        let index_html = generate(site).map_err(|e| wrap!(e))?;

        // Generate the HTML for the error page
        let error_html = generate_error_page();

        // Write the index HTML to a file
        std::fs::write(&site.index_html, &index_html).map_err(|e| wrap!(e.into()))?;

        // Write the error HTML to a file
        std::fs::write(&site.error_html, &error_html).map_err(|e| wrap!(e.into()))?;

        Ok(())
    }
}
