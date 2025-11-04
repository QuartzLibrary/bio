use std::{
    io::{self, Write as _},
    path::Path,
    process::{Output, Stdio},
};

use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt as _;

#[cfg(not(target_arch = "wasm32"))]
use uv_pep440::VersionSpecifier;
#[cfg(not(target_arch = "wasm32"))]
use uv_pep508::{Requirement, VerbatimUrl};

#[cfg(not(target_arch = "wasm32"))]
use crate::metadata::{ScriptMetadataError, script_dependencies, script_python_version};

/// A python script that uses the PEP 723 metadata format.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PythonScript {
    pub content: String,
}
#[cfg(not(target_arch = "wasm32"))]
impl PythonScript {
    #[tracing::instrument(level = "debug", skip(input))]
    pub async fn run(&self, input: impl AsRef<[u8]>) -> io::Result<Output> {
        install_python(&self.python_version()?).await?;

        let dir = tempfile::Builder::new().suffix("python_exec").tempdir()?;

        let script_path = dir.path().join("script.py");

        {
            let mut temp = std::fs::File::create(script_path.clone())?;
            temp.write_all(self.content.as_bytes())?;
            temp.flush()?;
        }

        let mut child = tokio::process::Command::new("uv")
            .arg("run")
            .arg(&script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(input.as_ref()).await?;
        stdin.flush().await?;
        stdin.shutdown().await?;
        drop(stdin);

        let output = child.wait_with_output().await?;
        Ok(clean_output(output, dir.path()))
    }
    #[tracing::instrument(level = "debug", skip(input))]
    pub fn run_blocking(&self, input: impl AsRef<[u8]>) -> io::Result<Output> {
        install_python_blocking(&self.python_version()?)?;

        let dir = tempfile::Builder::new().suffix("python_exec").tempdir()?;

        let script_path = dir.path().join("script.py");

        {
            let mut temp = std::fs::File::create(script_path.clone())?;
            temp.write_all(self.content.as_bytes())?;
            temp.flush()?;
        }

        let mut child = std::process::Command::new("uv")
            .arg("run")
            .arg(&script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(input.as_ref())?;
        stdin.flush()?;
        drop(stdin);

        let output = child.wait_with_output()?;
        Ok(clean_output(output, dir.path()))
    }
    pub fn python_version(&self) -> Result<VersionSpecifier, ScriptMetadataError> {
        script_python_version(&self.content)
    }
    pub fn dependencies(&self) -> Result<Vec<Requirement<VerbatimUrl>>, ScriptMetadataError> {
        script_dependencies(&self.content)
    }
}
#[cfg(not(target_arch = "wasm32"))]
fn clean_output(mut output: Output, path: &Path) -> Output {
    output.stderr = clean_temp_path(output.stderr, path);
    output
}
#[cfg(not(target_arch = "wasm32"))]
fn clean_temp_path(data: Vec<u8>, path: &Path) -> Vec<u8> {
    let dir_name = path.file_name().unwrap().to_str().unwrap();
    let Ok(data) = std::str::from_utf8(&data) else {
        return data;
    };
    regex::Regex::new(&regex::escape(dir_name))
        .unwrap()
        .replace(data, "temp_folder")
        .into_owned()
        .into_bytes()
}

#[cfg(not(target_arch = "wasm32"))]
#[tracing::instrument(level = "debug")]
pub(super) fn install_python_blocking(version: &VersionSpecifier) -> io::Result<()> {
    let child = std::process::Command::new("uv")
        .arg("python")
        .arg("install")
        .arg(version.to_string())
        .spawn()?;
    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "Failed to install Python {version}.\n{output:?}"
        )))
    }
}

// uv python install
#[cfg(not(target_arch = "wasm32"))]
#[tracing::instrument(level = "debug")]
pub(super) async fn install_python(version: &VersionSpecifier) -> io::Result<()> {
    let child = tokio::process::Command::new("uv")
        .arg("python")
        .arg("install")
        .arg(version.to_string())
        .spawn()?;
    let output = child.wait_with_output().await?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "Failed to install Python {version}.\n{output:?}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SCRIPT: &str = r#"
# /// script
# requires-python = "==3.10"
# dependencies = []
# ///

print(1)
"#;

    #[tokio::test]
    async fn test_run() {
        let script = PythonScript {
            content: TEST_SCRIPT.to_string(),
        };
        let output = script.run(b"").await.unwrap();
        println!("{output:?}");
        let Output {
            status,
            stdout,
            stderr,
        } = &output;
        assert!(status.success());
        assert_eq!(stdout, b"1\n");
        assert_eq!(stderr, b"");
    }
    #[test]
    fn test_run_blocking() {
        let script = PythonScript {
            content: TEST_SCRIPT.to_string(),
        };
        let output = script.run_blocking(b"").unwrap();
        println!("{output:?}");
        let Output {
            status,
            stdout,
            stderr,
        } = &output;
        assert!(status.success());
        assert_eq!(stdout, b"1\n");
        assert_eq!(stderr, b"");
    }
}

#[cfg(test)]
mod exception_tests {
    use super::*;

    const TEST_EXCEPTION: &[u8] = b"Traceback (most recent call last):\n  File \"/tmp/temp_folder/script.py\", line 7, in <module>\n    raise Exception('This is a test')\nException: This is a test\n";

    const TEST_EXCEPTION_SCRIPT: &str = r#"
# /// script
# requires-python = "==3.10"
# dependencies = []
# ///

raise Exception('This is a test')
"#;

    #[tokio::test]
    async fn test_run_exception() {
        let script = PythonScript {
            content: TEST_EXCEPTION_SCRIPT.to_string(),
        };
        let output = script.run(b"").await.unwrap();
        println!("{output:?}");
        let Output {
            status,
            stdout,
            stderr,
        } = &output;
        assert!(!status.success());
        assert_eq!(stdout, b"");
        assert_eq!(stderr, TEST_EXCEPTION);
    }
    #[test]
    fn test_run_exception_blocking() {
        let script = PythonScript {
            content: TEST_EXCEPTION_SCRIPT.to_string(),
        };
        let output = script.run_blocking(b"").unwrap();
        println!("{output:?}");
        let Output {
            status,
            stdout,
            stderr,
        } = &output;
        assert!(!status.success());
        assert_eq!(stdout, b"");
        assert_eq!(stderr, TEST_EXCEPTION);
    }
}
