use std::{
    ffi::OsStr,
    path::{Path, PathBuf, Prefix},
    process::Command,
};

pub fn new_wsl_command(command: impl AsRef<OsStr>) -> Command {
    if cfg!(target_os = "windows") {
        let mut cmd = Command::new("wsl");
        cmd.arg(command);
        cmd
    } else {
        Command::new(command)
    }
}

pub fn to_wsl_path(path: impl AsRef<Path>) -> Result<PathBuf, std::io::Error> {
    fn windows_drive(path: impl AsRef<Path>) -> Result<String, std::io::Error> {
        fn error(s: &'static str) -> std::io::Error {
            std::io::Error::new(std::io::ErrorKind::Unsupported, s)
        }
        assert!(path.as_ref().is_absolute());
        let first_component = path.as_ref().components().next().unwrap();
        Ok(match first_component {
            std::path::Component::Prefix(prefix_component) => match prefix_component.kind() {
                Prefix::Verbatim(s) => {
                    Ok(s.to_str().unwrap().trim_start_matches(r"\\?\").to_owned())
                }
                Prefix::VerbatimDisk(d) => Ok(d.as_ascii().unwrap().to_string()),
                Prefix::Disk(d) => Ok(d.as_ascii().unwrap().to_string()),

                Prefix::VerbatimUNC(_, _) => Err(error("UNC paths are not supported.")),
                Prefix::DeviceNS(_) => Err(error("Device namespace paths are not supported.")),
                Prefix::UNC(_, _) => Err(error("UNC paths are not supported.")),
            },
            _ => Err(error(
                "Absolute path does not contain a Windows drive letter.",
            )),
        }?
        .to_lowercase())
    }

    if cfg!(target_os = "windows") {
        let absolute_path = path.as_ref().canonicalize()?;
        let drive = windows_drive(&absolute_path)?;

        let rest = absolute_path.components().skip(1).collect::<PathBuf>();
        Ok(PathBuf::from(format!(
            "/mnt/{}/{}",
            drive,
            rest.display().to_string().replace("\\", "/")
        )))
    } else {
        Ok(path.as_ref().to_path_buf())
    }
}
