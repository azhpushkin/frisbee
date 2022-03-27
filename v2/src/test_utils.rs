use std::fs::{remove_file, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

use crate::loader::{load_program, WholeProgram};

pub struct TestFilesCreator {
    temp_workdir: TempDir,
    main_path: PathBuf,
}

impl TestFilesCreator {
    pub fn new() -> TestFilesCreator {
        let workdir = tempdir().unwrap();
        let main_path = workdir.path().clone().join("main.frisbee");
        TestFilesCreator { temp_workdir: workdir, main_path }
    }

    pub fn set_mainfile<S>(&mut self, contents: S)
    where
        S: Into<String> + Clone,
    {
        self.set_file("main.frisbee", contents);
    }

    pub fn set_file<N, C>(&mut self, name: N, contents: C)
    where
        N: Into<String> + Clone,
        C: Into<String> + Clone,
    {
        let file_path = self.temp_workdir.path().join(name.into());
        if file_path.exists() {
            remove_file(file_path.as_path()).unwrap();
        }

        let mut file = File::create(file_path).unwrap();
        file.write(contents.into().as_bytes()).unwrap();
    }

    pub fn get_main_path(&self) -> &Path {
        self.main_path.as_path()
    }

    pub fn load_program(&self) -> WholeProgram {
        let p = load_program(self.get_main_path());
        assert!(p.is_some());
        p.unwrap()
    }
}
