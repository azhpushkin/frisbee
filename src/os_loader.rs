use std::path::{Path, PathBuf};

use crate::{alias, loader};

pub struct FileSystemLoader {
    pub workdir: PathBuf,
}

impl loader::FrisbeeModuleLoader for FileSystemLoader {
    fn load_module(&self, module: &alias::ModuleAlias) -> Result<String, String> {
        let mut file_path = self.workdir.to_owned();
        for subpath in module.to_path().iter() {
            file_path.push(subpath);
        }
        file_path.set_extension("frisbee");

        std::fs::read_to_string(&file_path).map_err(|err| format!("{}", err))
    }
}

// TODO: color output?
pub fn entry_path_to_loader_and_main_module(
    entry_file_path: &String,
) -> (FileSystemLoader, alias::ModuleAlias) {
    // TODO: check file exist
    let entry_file_path = Path::new(entry_file_path);
    if entry_file_path.extension().unwrap() != "frisbee" {
        panic!(
            "Only *.frisbee files are allowed, but got {:?}!",
            entry_file_path.extension()
        );
    };

    let workdir = entry_file_path.parent().unwrap();
    let loader = FileSystemLoader { workdir: workdir.to_owned() };

    // TODO: check how this works under windows/macos
    let main_module = entry_file_path.file_stem().unwrap().to_str().unwrap();
    let main_alias = alias::ModuleAlias::new(&[main_module.to_owned()]);

    (loader, main_alias)
}
