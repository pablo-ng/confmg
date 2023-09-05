use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, Context, Result};

fn expand_tilde<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    // from https://stackoverflow.com/a/54306906/9616820
    let path = path.as_ref();
    if !path.starts_with("~") {
        return Ok(path.to_path_buf());
    }
    let mut home_dir = dirs::home_dir().ok_or(anyhow!(
        "Failed to find home directory for path '{}'",
        path.display()
    ))?;
    if path == Path::new("~") {
        return Ok(home_dir);
    }
    if home_dir == Path::new("/") {
        // Corner case: don't prepend extra `/`, just drop the tilde.
        Ok(path.strip_prefix("~").unwrap().to_path_buf())
    } else {
        home_dir.push(path.strip_prefix("~/").unwrap());
        Ok(home_dir)
    }
}

pub fn is_file<P: AsRef<Path>>(path: P) -> Result<bool> {
    match fs::metadata(expand_tilde(&path)?) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => Ok(false),
            // io::ErrorKind::IsADirectory TODO need?
            _ => Err(err).context(format!(
                "Failed to read file at '{}'",
                path.as_ref().display()
            ))?,
        },
    }
}

pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
    fs::read_to_string(expand_tilde(&path)?).context(format!(
        "Failed to read file at '{}'",
        path.as_ref().display()
    ))
}

pub fn _write_file<P: AsRef<Path>>(path: P, contents: String) -> Result<()> {
    fs::write(expand_tilde(&path)?, contents).context(format!(
        "Failed to write file at '{}'",
        path.as_ref().display()
    ))
}

pub fn open_file<P: AsRef<OsStr> + AsRef<Path>, E: AsRef<OsStr>>(editor: E, path: P) -> Result<()> {
    let _ = Command::new(editor)
        .arg(expand_tilde(&path)?)
        .status()
        .context(format!(
            "Failed to open file at '{}'",
            <P as AsRef<Path>>::as_ref(&path).display()
        ))?;
    Ok(())
}
