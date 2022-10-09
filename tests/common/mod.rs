use std::{
    env,
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
};

pub(crate) fn get_tests_resources_directory() -> Result<PathBuf> {
    let mut current_dir: PathBuf = env::current_dir()?;
    println!("current directory: {}", current_dir.display());
    current_dir.push("tests");
    current_dir.push("resources");
    println!("tests resources directory: {}", current_dir.display());
    if Path::new(current_dir.as_os_str()).try_exists()? {
        Ok(current_dir)
    } else {
        let mut message = "tests/resources directory not found in current directory: ".to_string();
        message.push_str(current_dir.display().to_string().as_str());
        Err(Error::new(ErrorKind::NotFound, message.as_str()))
    }
}
