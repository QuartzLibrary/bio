use std::{
    fmt,
    io::{self, Write as _},
    pin::pin,
    process::Stdio,
};

use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use tempfile::TempDir;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt as _};

use super::{
    function::{MaybeWithStdoutStderr, PythonFunction, WithStdoutStderr},
    script::PythonScript,
};

const SCRIPT_START_MARKER: &str = "c5b70a4e-69e8-4af2-ae50-2c392e6e2132";

#[derive(Debug)]
pub struct PythonMap {
    _tempdir: TempDir,
    child: tokio::process::Child,
    stdout: tokio::io::BufReader<tokio::process::ChildStdout>,
    stderr: tokio::io::BufReader<tokio::process::ChildStderr>,
    first: bool,
}
impl PythonMap {
    pub(super) fn new(function: &PythonFunction) -> io::Result<Self> {
        let dir = tempfile::Builder::new().suffix("python_exec").tempdir()?;

        let script_path = dir.path().join("script.py");

        {
            let mut temp = std::fs::File::create(script_path.clone())?;
            temp.write_all(function.stream_script().script().as_bytes())?;
            temp.flush()?;
        }

        let mut child = tokio::process::Command::new("uv")
            .arg("run")
            .arg(&script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = tokio::io::BufReader::new(child.stdout.take().unwrap());
        let stderr = tokio::io::BufReader::new(child.stderr.take().unwrap());

        Ok(Self {
            _tempdir: dir,
            child,
            stdout,
            stderr,
            first: true,
        })
    }
    pub async fn run(
        &mut self,
        input: impl AsRef<[u8]>,
    ) -> io::Result<WithStdoutStderr<Result<Value, String>>> {
        let output = self.raw_run(input).await?;
        let structured: MaybeWithStdoutStderr<Value> = serde_json::from_str(&output)?;
        structured.unpack()
    }
    pub async fn run_typed<In, Out>(
        &mut self,
        input: In,
    ) -> io::Result<WithStdoutStderr<Result<Out, String>>>
    where
        In: Serialize,
        Out: DeserializeOwned + fmt::Debug,
    {
        let input = serde_json::to_vec(&input)?;
        let output = self.raw_run(input).await?;
        let value: MaybeWithStdoutStderr<Out> = serde_json::from_str(&output)?;
        value.unpack()
    }
    async fn raw_run(&mut self, input: impl AsRef<[u8]>) -> io::Result<String> {
        if input.as_ref().contains(&b'\n') {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Input contains newlines",
            ))?
        }

        if self.first {
            self.first = false;
            wait_for_marker(&mut self.stdout, SCRIPT_START_MARKER).await?;
            wait_for_marker(&mut self.stderr, SCRIPT_START_MARKER).await?;
        }

        let stdin = self.child.stdin.as_mut().unwrap();
        stdin.write_all(input.as_ref()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;

        let mut output = String::new();
        self.stdout.read_line(&mut output).await?;

        Ok(output)
    }
}
async fn wait_for_marker(reader: impl AsyncBufRead, marker: &str) -> io::Result<()> {
    let mut reader = pin!(reader);

    let mut buf = String::new();
    loop {
        buf.clear();
        reader.read_line(&mut buf).await?;
        if buf.contains(marker) {
            return Ok(());
        }
    }
}
impl PythonFunction {
    fn stream_script(&self) -> PythonScript {
        let (output, parse_input) = self.output_and_parse_input();

        let function = &self.function;

        let content = format!(
            r##"
import sys
import os
import io
import traceback
import json
from contextlib import redirect_stdout, redirect_stderr
from pydantic import BaseModel

{function}

def main():
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

        if not raw_input:
            continue

        # Parse input
        input = {parse_input}

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

        print(__InternalOutputModel(
            value=result,
            error=clean_stacktrace(str(exception)) if exception else None,
            stdout=stdout.getvalue(),
            stderr=stderr.getvalue(),
        ).model_dump_json(), flush=True)

if __name__ == "__main__":
    main()
"##
        );

        PythonScript {
            python_version: self.python_version.clone(),
            dependencies: self.deps(),
            content,
        }
    }
}

type TestValue = (Value, WithStdoutStderr<Result<Value, String>>);
impl PythonMap {
    pub fn test_values() -> Vec<(Self, Vec<TestValue>)> {
        vec![Self::simple(), Self::dep(), Self::exception()]
    }

    fn simple() -> (Self, Vec<TestValue>) {
        const PYTHON_VERSION: &str = "==3.10";
        const FUNCTION: &str = "
def process(input: int) -> int:
    print(\"hello\")
    return input + 2
";

        let values = vec![(
            Value::Number(40.into()),
            WithStdoutStderr {
                value: Ok(Value::Number(42.into())),
                stdout: "hello\n".to_string(),
                stderr: "".to_string(),
            },
        )];

        (
            PythonFunction {
                python_version: PYTHON_VERSION.to_string(),
                dependencies: vec![],
                function: FUNCTION.to_string(),
            }
            .into_map()
            .unwrap(),
            values,
        )
    }

    fn dep() -> (Self, Vec<TestValue>) {
        const PYTHON_VERSION: &str = "==3.10";
        const FUNCTION: &str = "
import numpy as np
def process(x: int) -> list[int]:
    return np.array(x).tolist()
";

        let values = vec![(
            Value::Array(vec![Value::Number(40.into())]),
            WithStdoutStderr {
                value: Ok(Value::Array(vec![Value::Number(40.into())])),
                stdout: "".to_string(),
                stderr: "".to_string(),
            },
        )];

        (
            PythonFunction {
                python_version: PYTHON_VERSION.to_string(),
                dependencies: vec!["numpy".to_owned(), "pandas".to_owned()],
                function: FUNCTION.to_string(),
            }
            .into_map()
            .unwrap(),
            values,
        )
    }

    fn exception() -> (Self, Vec<TestValue>) {
        const PYTHON_VERSION: &str = "==3.10";
        const FUNCTION: &str = "
def process(input: int) -> int:
    print(\"hello\")
    raise Exception('This is a test')
";
        const ERROR: &str = "Traceback (most recent call last):\n  File \"/temp_folder/script.py\", line 53, in main\n    result = process(input)\n  File \"/temp_folder/script.py\", line 19, in process\n    raise Exception('This is a test')\nException: This is a test\n";

        let values = vec![(
            Value::Number(40.into()),
            WithStdoutStderr {
                value: Err(ERROR.to_string()),
                stdout: "hello\n".to_string(),
                stderr: "".to_string(),
            },
        )];

        (
            PythonFunction {
                python_version: PYTHON_VERSION.to_string(),
                dependencies: vec![],
                function: FUNCTION.to_string(),
            }
            .into_map()
            .unwrap(),
            values,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn basic() {
        for (mut map, values) in PythonMap::test_values() {
            for (input, expected) in values {
                let structured = map.run(serde_json::to_vec(&input).unwrap()).await.unwrap();
                assert_eq!(structured, expected);
            }
        }
    }

    #[tokio::test]
    async fn test_map() {
        let (mut map, _) = PythonMap::simple();

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
        let (mut map, _) = PythonMap::simple();
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

    const ERROR: &str = "Traceback (most recent call last):\n  File \"/temp_folder/script.py\", line 53, in main\n    result = process(input)\n  File \"/temp_folder/script.py\", line 19, in process\n    raise Exception('This is a test')\nException: This is a test\n";

    #[tokio::test]
    async fn test_map_exception() {
        let (mut map, _) = PythonMap::exception();

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
