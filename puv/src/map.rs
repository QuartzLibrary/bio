use std::{
    fmt,
    io::{self, Write as _},
    pin::pin,
    process::Stdio,
};

use futures::FutureExt;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWriteExt as _};

use super::function::{MaybeWithStdoutStderr, PythonFunction, WithStdoutStderr};

const SCRIPT_START_MARKER: &str = "c5b70a4e-69e8-4af2-ae50-2c392e6e2132";

/// See [PythonFunction] for more information.
///
/// This is a 'live' version that allows mapping multiple values.
#[derive(Debug)]
pub struct PythonMap {
    _tempdir: TempDir,
    #[expect(dead_code)]
    child: tokio::process::Child,
    stdin: tokio::process::ChildStdin,
    stdout: tokio::io::BufReader<tokio::process::ChildStdout>,
    #[expect(dead_code)]
    stderr: tokio::io::BufReader<tokio::process::ChildStderr>,
}
impl PythonMap {
    #[tracing::instrument(level = "debug")]
    pub(super) async fn new(function: &PythonFunction) -> io::Result<Self> {
        super::script::install_python(&function.python_version()?).await?;

        let dir = tempfile::Builder::new().suffix("python_exec").tempdir()?;

        let script_path = dir.path().join("script.py");

        {
            let script = function.stream_script()?;
            let mut temp = std::fs::File::create(script_path.clone())?;
            temp.write_all(script.as_bytes())?;
            temp.flush()?;

            log::debug!("Running script:\n{script}");
        }

        let mut child = tokio::process::Command::new("uv")
            .arg("run")
            .arg(&script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        log::debug!("Script spawned.");

        let stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();

        wait_for_setup(&mut child, &mut stdout, &mut stderr).await?;

        Ok(Self {
            _tempdir: dir,
            child,
            stdin,
            stdout: tokio::io::BufReader::new(stdout),
            stderr: tokio::io::BufReader::new(stderr),
        })
    }
    #[tracing::instrument(level = "debug", skip(input))]
    pub async fn run(
        &mut self,
        input: impl AsRef<[u8]>,
    ) -> io::Result<WithStdoutStderr<Result<Value, String>>> {
        log::debug!("Running function.");

        if input.as_ref().contains(&b'\n') {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Input contains newlines",
            ))?
        }

        let output = self.raw_run(input).await?;
        let structured: MaybeWithStdoutStderr<Value> = serde_json::from_str(&output)?;
        structured.unpack()
    }
    #[tracing::instrument(level = "debug", skip(input))]
    pub async fn run_typed<In, Out>(
        &mut self,
        input: In,
    ) -> io::Result<WithStdoutStderr<Result<Out, String>>>
    where
        In: Serialize,
        Out: DeserializeOwned + fmt::Debug,
    {
        log::debug!("Running typed function.");
        let input = serde_json::to_vec(&input)?;
        let output = self.raw_run(input).await?;
        let value: MaybeWithStdoutStderr<Out> = serde_json::from_str(&output)?;
        value.unpack()
    }
    async fn raw_run(&mut self, input: impl AsRef<[u8]>) -> io::Result<String> {
        self.stdin.write_all(br#"{"input":"#).await?;
        self.stdin.write_all(input.as_ref()).await?;
        self.stdin.write_all(b"}\n").await?;
        self.stdin.flush().await?;

        let mut output = String::new();
        self.stdout.read_line(&mut output).await?; // TODO: catch exit

        Ok(output)
    }
}
#[tracing::instrument(level = "debug", skip(child, stdout, stderr))]
async fn wait_for_setup(
    child: &mut tokio::process::Child,
    mut stdout: &mut tokio::process::ChildStdout,
    mut stderr: &mut tokio::process::ChildStderr,
) -> io::Result<()> {
    let mut stdout_buf = Vec::with_capacity(1_000);
    let mut stderr_buf = Vec::with_capacity(1_000);

    let result = {
        let std_io = Box::pin(futures::future::join(
            wait_for_marker("stdout", &mut stdout, &mut stdout_buf, SCRIPT_START_MARKER),
            wait_for_marker("stderr", &mut stderr, &mut stderr_buf, SCRIPT_START_MARKER),
        ));
        let wait = pin!(child.wait());

        match futures::future::select(std_io, wait).await {
            futures::future::Either::Left(((out, err), wait)) => {
                if let Some(wait) = wait.now_or_never() {
                    log::warn!("Python process exited with code {wait:?}");
                }
                futures::future::Either::Left((out, err))
            }
            futures::future::Either::Right((wait, std_io)) => {
                drop(std_io.now_or_never());
                futures::future::Either::Right(wait)
            }
        }
    };

    let stdout_buf = String::from_utf8_lossy(&stdout_buf);
    let stderr_buf = String::from_utf8_lossy(&stderr_buf);

    match result {
        futures::future::Either::Left((out, err)) => {
            match (out, err) {
                (Ok(()), Ok(())) => {
                    log::trace!(
                        "Python process started.\n\
                        STDOUT:\n\
                        {stdout_buf}\n\
                        STDERR:\n\
                        {stderr_buf}"
                    );
                }
                (Err(e), Ok(())) => {
                    Err(io::Error::other(format!(
                        "Stdout error: {e:?}\n\
                        STDOUT:\n\
                        {stdout_buf}\n\
                        STDERR:\n\
                        {stderr_buf}"
                    )))?;
                }
                (Ok(()), Err(e)) => {
                    Err(io::Error::other(format!(
                        "Stderr error: {e:?}\n\
                        STDOUT:\n\
                        {stdout_buf}\n\
                        STDERR:\n\
                        {stderr_buf}"
                    )))?;
                }
                (Err(e), Err(e2)) => {
                    Err(io::Error::other(format!(
                        "Stdout error: {e:?}\n\
                        Stderr error: {e2:?}\n\
                        STDOUT:\n\
                        {stdout_buf}\n\
                        STDERR:\n\
                        {stderr_buf}"
                    )))?;
                }
            };
        }
        futures::future::Either::Right(wait) => {
            let code = wait?;
            Err(io::Error::other(format!(
                "Python process unexpectedly exited with code {code}.\n\
                STDOUT:\n\
                {stdout_buf}\n\
                STDERR:\n\
                {stderr_buf}"
            )))?;
        }
    };

    Ok(())
}

#[tracing::instrument(level = "debug", skip(reader, data))]
async fn wait_for_marker(
    kind: &str,
    reader: impl AsyncRead,
    data: &mut Vec<u8>,
    marker: &str,
) -> io::Result<()> {
    fn check_marker(data: &[u8], marker: &str) -> bool {
        data.split(|b| *b == b'\n')
            .rev()
            .take(2) // Ends in a newline
            .any(|line| {
                line.windows(marker.len())
                    .any(|window| window == marker.as_bytes())
            })
    }

    data.clear();

    let mut reader = pin!(reader);

    let mut buf = [0; 1_000];
    loop {
        let read = reader.read(&mut buf).await?;
        data.extend_from_slice(&buf[..read]);
        if check_marker(data, marker) {
            return Ok(());
        } else if read == 0 {
            return Err(io::Error::other(format!(
                "Reached EOF on {kind} before seeing start marker."
            )));
        }
    }
}
impl PythonFunction {
    fn stream_script(&self) -> io::Result<String> {
        let (input, output) = self.output_and_parse_input()?;

        let function = crate::metadata::inject_dependency(&self.function, "pydantic")?;

        let content = format!(
            r##"{function}

def main():
    import sys
    import os
    import io
    import traceback
    import json
    from contextlib import redirect_stdout, redirect_stderr
    from pydantic import BaseModel

    class __InternalInputModel(BaseModel):
        input: {input}

    class __InternalOutputModel(BaseModel):
        value: {output} | None
        error: str | None
        stdout: str
        stderr: str

    def clean_stacktrace(stacktrace: str) -> str:
        return stacktrace.replace(os.path.dirname(os.path.abspath(__file__)), "/temp_folder")

    print({SCRIPT_START_MARKER:?}, flush=True)
    print({SCRIPT_START_MARKER:?}, flush=True, file=sys.stderr)

    for raw_input in sys.stdin:
        raw_input = raw_input.strip()

        # Parse input
        try:
            input = __InternalInputModel.model_validate_json(raw_input).input
        except Exception:
            output = __InternalOutputModel(
                value=None,
                error=str(traceback.format_exc()),
                stdout="",
                stderr=raw_input,
            )
            print(output.model_dump_json(), flush=True)
            return

        # Run function with stdout protection
        stdout = io.StringIO()
        stderr = io.StringIO()

        result = None
        exception = None

        with redirect_stdout(stdout), redirect_stderr(stderr):
            try:
                result = process(input)
            except Exception:
                exception = traceback.format_exc()

        output = __InternalOutputModel(
            value=result,
            error=clean_stacktrace(str(exception)) if exception else None,
            stdout=stdout.getvalue(),
            stderr=stderr.getvalue(),
        )
        print(output.model_dump_json(), flush=True)

if __name__ == "__main__":
    main()
"##
        );

        Ok(content)
    }
}

type TestValue = (Value, WithStdoutStderr<Result<Value, String>>);
impl PythonMap {
    pub fn test_values() -> Vec<(PythonFunction, Vec<TestValue>)> {
        vec![Self::simple(), Self::dep(), Self::exception()]
    }

    fn simple() -> (PythonFunction, Vec<TestValue>) {
        PythonFunction::simple()
    }

    fn dep() -> (PythonFunction, Vec<TestValue>) {
        PythonFunction::dep()
    }

    fn exception() -> (PythonFunction, Vec<TestValue>) {
        const ERROR: &str = r#"Traceback (most recent call last):
  File "/temp_folder/script.py", line 63, in main
    result = process(input)
  File "/temp_folder/script.py", line 11, in process
    raise Exception('This is a test')
Exception: This is a test
"#;

        let (function, mut values) = PythonFunction::exception();

        for value in &mut values {
            value.1.value = Err(ERROR.to_string());
        }

        (function, values)
    }

    #[cfg(test)]
    fn invalid_function() -> (PythonFunction, Vec<TestValue>) {
        PythonFunction::invalid_function()
    }

    #[cfg(test)]
    fn invalid_import() -> (PythonFunction, Vec<TestValue>) {
        PythonFunction::invalid_import()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn basic() {
        for (map, values) in PythonMap::test_values() {
            let mut map = map.into_map().await.unwrap();
            for (input, expected) in values {
                let structured = map.run(serde_json::to_vec(&input).unwrap()).await.unwrap();
                assert_eq!(structured, expected);
            }
        }
    }

    #[tokio::test]
    async fn test_map() {
        let (map, _) = PythonMap::simple();
        let mut map = map.into_map().await.unwrap();

        let structured = map.run(b"40").await.unwrap();
        assert_eq!(
            structured,
            WithStdoutStderr {
                value: Ok(Value::Number(42.into())),
                stdout: "hello\n".to_string(),
                stderr: "".to_string(),
            }
        );

        let structured = map.run(b"1").await.unwrap();
        assert_eq!(
            structured,
            WithStdoutStderr {
                value: Ok(Value::Number(3.into())),
                stdout: "hello\n".to_string(),
                stderr: "".to_string(),
            }
        );
    }
    #[test]
    #[ignore]
    fn test_map_blocking() {
        todo!()
    }

    #[tokio::test]
    async fn test_map_typed() {
        let (map, _) = PythonMap::simple();
        let mut map = map.into_map().await.unwrap();

        let structured = map.run_typed::<_, i32>(40).await.unwrap();
        assert_eq!(
            structured,
            WithStdoutStderr {
                value: Ok(42),
                stdout: "hello\n".to_string(),
                stderr: "".to_string(),
            }
        );
        let structured = map.run_typed::<_, i32>(1).await.unwrap();
        assert_eq!(
            structured,
            WithStdoutStderr {
                value: Ok(3),
                stdout: "hello\n".to_string(),
                stderr: "".to_string(),
            }
        );
    }
    #[test]
    #[ignore]
    fn test_map_typed_blocking() {
        todo!()
    }
}

#[cfg(test)]
mod exception_tests {
    use super::*;

    const ERROR: &str = r#"Traceback (most recent call last):
  File "/temp_folder/script.py", line 63, in main
    result = process(input)
  File "/temp_folder/script.py", line 11, in process
    raise Exception('This is a test')
Exception: This is a test
"#;

    #[tokio::test]
    async fn test_map_exception() {
        let (map, _) = PythonMap::exception();
        let mut map = map.into_map().await.unwrap();

        let structured = map.run(b"40").await.unwrap();
        assert_eq!(
            structured,
            WithStdoutStderr {
                value: Err(ERROR.to_string()),
                stdout: "hello\n".to_string(),
                stderr: "".to_string(),
            }
        );

        let structured = map.run(b"1").await.unwrap();
        assert_eq!(
            structured,
            WithStdoutStderr {
                value: Err(ERROR.to_string()),
                stdout: "hello\n".to_string(),
                stderr: "".to_string(),
            }
        );
    }
    #[test]
    #[ignore]
    fn test_map_blocking_exception() {
        todo!()
    }
}

#[cfg(test)]
mod invalid_tests {
    use super::*;

    #[tokio::test]
    async fn test_map_invalid() {
        let (map, _) = PythonMap::invalid_function();
        match map.into_map().await {
            Ok(output) => {
                println!("{output:?}");
                unreachable!()
            }
            Err(e) => {
                println!("{e:?}")
            }
        }
    }

    #[tokio::test]
    async fn test_map_invalid_import() {
        let (map, _) = PythonMap::invalid_import();
        match map.into_map().await {
            Ok(output) => {
                println!("{output:?}");
                unreachable!()
            }
            Err(e) => {
                println!("{e:?}")
            }
        }
    }
}
