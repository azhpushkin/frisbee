use std::fs::File;
use std::io::Write;
use tempfile::{tempdir, TempDir};

pub struct TestFilesCreator {
    temp_workdir: TempDir,
    main_file: String
}

impl TestFilesCreator {
    pub fn new() ->TestFilesCreator {
        let workdir = tempdir().unwrap();
        TestFilesCreator{temp_workdir: workdir, main_file: String::from("")}
    }

    pub fn add_mainfile<S>(&mut self, name: S, contents: S) 
        where S: Into<String> + Clone
    {
        self.main_file = name.clone().into();
        self.add_file(name, contents);
    }

    pub fn add_file<S>(&mut self, name: S, contents: S) 
        where S: Into<String> + Clone
    {
        let mut file = File::create(
            self.temp_workdir.path().join(name.into())
        ).unwrap();
        file.write(contents.into().as_bytes()).unwrap();
    }
}