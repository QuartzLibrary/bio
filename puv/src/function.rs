use std::{fmt, io, process::Output, sync::LazyLock};

use regex::Regex;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
#[cfg(not(target_arch = "wasm32"))]
use uv_pep440::VersionSpecifier;
#[cfg(not(target_arch = "wasm32"))]
use uv_pep508::{Requirement, VerbatimUrl};

#[cfg(not(target_arch = "wasm32"))]
use crate::{
    metadata::script_dependencies,
    metadata::{ScriptMetadataError, script_python_version},
    script::PythonScript,
};

/// A Python function.
///
/// It must:
/// - be valid Python code with PEP 723 metadata.
/// - expose a `process` function
///
/// ```python
/// # Both input and output must be typed and JSON serializable.
/// def process(input: int) -> int:
///     return input + 1
/// ```
///
/// You can use Pydantic models for complex input and output types.
/// ```python
/// from pydantic import BaseModel
///
/// class Input(BaseModel):
///     input: int
///
/// class Output(BaseModel):
///     output: int
///
/// def process(input: Input) -> Output:
///     return Output(output=input.input + 1)
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PythonFunction {
    pub function: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WithStdoutStderr<T> {
    pub value: T,
    pub stdout: String,
    pub stderr: String,
}
pub type PyFnOutput<T> = (WithStdoutStderr<Result<T, String>>, Output);

#[cfg(not(target_arch = "wasm32"))]
impl PythonFunction {
    pub async fn run(&self, input: impl AsRef<[u8]>) -> io::Result<PyFnOutput<Value>> {
        // TODO: better pass-through of input
        let input = format!("{{\"input\":{}}}", str::from_utf8(input.as_ref()).unwrap());
        let output = self.script()?.run(&input).await?;
        parse_and_unpack(output)
    }
    pub async fn run_typed<In, Out>(&self, input: In) -> io::Result<PyFnOutput<Out>>
    where
        In: Serialize,
        Out: DeserializeOwned + fmt::Debug,
    {
        // TODO: better pass-through of input
        let input = format!("{{\"input\":{}}}", serde_json::to_string(&input)?);
        let output = self.script()?.run(&input).await?;
        parse_and_unpack(output)
    }

    pub fn run_blocking(&self, input: impl AsRef<[u8]>) -> io::Result<PyFnOutput<Value>> {
        // TODO: better pass-through of input
        let input = format!("{{\"input\":{}}}", str::from_utf8(input.as_ref()).unwrap());
        let output = self.script()?.run_blocking(input)?;
        parse_and_unpack(output)
    }
    pub fn run_typed_blocking<In, Out>(&self, input: In) -> io::Result<PyFnOutput<Out>>
    where
        In: Serialize,
        Out: DeserializeOwned + fmt::Debug,
    {
        // TODO: better pass-through of input
        let input = format!("{{\"input\":{}}}", serde_json::to_string(&input)?);
        let output = self.script()?.run_blocking(&input)?;
        parse_and_unpack(output)
    }

    pub fn python_version(&self) -> Result<VersionSpecifier, ScriptMetadataError> {
        script_python_version(&self.function)
    }
    pub fn dependencies(&self) -> Result<Vec<Requirement<VerbatimUrl>>, ScriptMetadataError> {
        script_dependencies(&self.function)
    }

    pub async fn into_map(self) -> io::Result<super::map::PythonMap> {
        super::map::PythonMap::new(&self).await
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl PythonFunction {
    fn script(&self) -> io::Result<PythonScript> {
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

    # Buffer all stdin
    raw_input = sys.stdin.read()

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

        Ok(PythonScript { content })
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl PythonFunction {
    pub(super) fn output_and_parse_input(&self) -> io::Result<(&str, &str)> {
        static FN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            let var = "[a-zA-Z0-9_-]+";
            let type_ = "[a-zA-Z0-9\\[\\]_-]+";
            Regex::new(&format!(
                r"def\s+process\({var}\s*:\s*(?<input>{type_})\)\s*->\s*(?<output>{type_}):\s*\n",
            ))
            .unwrap()
        });

        let capture = {
            let mut captures = FN_REGEX.captures_iter(&self.function);
            let Some(capture) = captures.next() else {
                return Err(io::Error::other(format!(
                    "# No matches for the `process` function:\n{}",
                    self.function
                )));
            };
            assert!(captures.next().is_none());
            capture
        };

        let input = capture.name("input").unwrap().as_str();
        let output = capture.name("output").unwrap().as_str();

        Ok((input, output))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_and_unpack<T>(output: Output) -> io::Result<PyFnOutput<T>>
where
    T: DeserializeOwned + fmt::Debug,
{
    let value: MaybeWithStdoutStderr<T> = match serde_json::from_slice(&output.stdout) {
        Ok(value) => value,
        Err(e) => {
            let Output {
                status,
                stdout,
                stderr,
            } = output;
            let stdout = String::from_utf8_lossy(&stdout);
            let stderr = String::from_utf8_lossy(&stderr);
            return Err(io::Error::other(format!(
                "Failed to parse output: {e:?}.\n\
                Exit status: {status:?}.\n\
                stdout:\n{stdout:?}.\n\
                stderr:\n{stderr:?}.\n"
            )));
        }
    };
    let value = value.unpack()?;
    Ok((value, output))
}

#[cfg(not(target_arch = "wasm32"))]
/// Intermediate struct to parse the output of the Python script.
#[derive(Debug, Deserialize)]
pub(super) struct MaybeWithStdoutStderr<T> {
    value: Option<T>,
    error: Option<String>,
    stdout: String,
    stderr: String,
}
#[cfg(not(target_arch = "wasm32"))]
impl<T: fmt::Debug> MaybeWithStdoutStderr<T> {
    pub(super) fn unpack(self) -> io::Result<WithStdoutStderr<Result<T, String>>> {
        static FOLDER_NAME: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"/\.[a-zA-Z0-9_-]*python_exec/script\.py").unwrap());

        let Self {
            value,
            error,
            stdout,
            stderr,
        } = self;

        match (value, error) {
            (None, None) => Err(io::Error::other("No value or error")),
            (None, Some(error)) => Ok(WithStdoutStderr {
                value: Err(FOLDER_NAME
                    .replace_all(&error, "/temp_folder/script.py")
                    .into_owned()),
                stdout,
                stderr,
            }),
            (Some(value), None) => Ok(WithStdoutStderr {
                value: Ok(value),
                stdout,
                stderr,
            }),
            (Some(value), Some(error)) => Err(io::Error::other(format!(
                "Got both value and error:\nvalue: {value:?}\nerror: {error:?}"
            ))),
        }
    }
}

type TestValue = (Value, WithStdoutStderr<Result<Value, String>>);
impl PythonFunction {
    pub fn test_values() -> Vec<(Self, Vec<TestValue>)> {
        vec![Self::simple(), Self::dep(), Self::exception()]
    }

    pub(super) fn simple() -> (Self, Vec<TestValue>) {
        const FUNCTION: &str = r#"
# /// script
# requires-python = "==3.10"
# dependencies = []
# ///

def process(input: int) -> int:
    print("hello")
    return input + 2
"#;

        let values = vec![(
            Value::Number(40.into()),
            WithStdoutStderr {
                value: Ok(Value::Number(42.into())),
                stdout: "hello\n".to_string(),
                stderr: "".to_string(),
            },
        )];

        (
            Self {
                function: FUNCTION.to_string(),
            },
            values,
        )
    }

    pub(super) fn dep() -> (Self, Vec<TestValue>) {
        const FUNCTION: &str = r#"
# /// script
# requires-python = "==3.10"
# dependencies = ["numpy", "pandas"]
# ///

import numpy as np
def process(x: int) -> list[int]:
    return np.array([x]).tolist()
"#;

        let values = vec![(
            Value::Number(40.into()),
            WithStdoutStderr {
                value: Ok(Value::Array(vec![Value::Number(40.into())])),
                stdout: "".to_string(),
                stderr: "".to_string(),
            },
        )];

        (
            Self {
                function: FUNCTION.to_string(),
            },
            values,
        )
    }

    pub(super) fn exception() -> (Self, Vec<TestValue>) {
        const FUNCTION: &str = r#"
# /// script
# requires-python = "==3.10"
# dependencies = []
# ///

def process(input: int) -> int:
    print("hello")
    raise Exception('This is a test')
"#;
        const ERROR: &str = r#"Traceback (most recent call last):
  File "/temp_folder/script.py", line 60, in main
    result = process(input)
  File "/temp_folder/script.py", line 11, in process
    raise Exception('This is a test')
Exception: This is a test
"#;

        let values = vec![(
            Value::Number(40.into()),
            WithStdoutStderr {
                value: Err(ERROR.to_string()),
                stdout: "hello\n".to_string(),
                stderr: "".to_string(),
            },
        )];

        (
            Self {
                function: FUNCTION.to_string(),
            },
            values,
        )
    }

    #[cfg(test)]
    pub(super) fn invalid_function() -> (Self, Vec<TestValue>) {
        const FUNCTION: &str = r#"
# /// script
# requires-python = "==3.10"
# dependencies = []
# ///

def pro cess(input: int) -> int:
    print("hello")
    raise Exception('This is a test')
"#;

        let values = vec![(
            Value::Number(40.into()),
            WithStdoutStderr {
                value: Err("".to_string()),
                stdout: "".to_string(),
                stderr: "".to_string(),
            },
        )];

        (
            Self {
                function: FUNCTION.to_string(),
            },
            values,
        )
    }

    #[cfg(test)]
    pub(super) fn invalid_import() -> (Self, Vec<TestValue>) {
        const FUNCTION: &str = r#"
# /// script
# requires-python = "==3.10"
# dependencies = []
# ///

import numpy as np # Fails

def process(input: int) -> int:
    return 42
"#;

        let values = vec![(
            Value::Number(40.into()),
            WithStdoutStderr {
                value: Err("".to_string()),
                stdout: "".to_string(),
                stderr: "".to_string(),
            },
        )];

        (
            Self {
                function: FUNCTION.to_string(),
            },
            values,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn basic() {
        for (function, values) in PythonFunction::test_values() {
            for (input, expected) in values {
                let (structured, _output) = function
                    .run(serde_json::to_vec(&input).unwrap())
                    .await
                    .unwrap();
                assert_eq!(structured, expected);
            }
        }
    }

    #[tokio::test]
    async fn test_run_function() {
        let (function, _) = PythonFunction::simple();
        let (structured, output) = function.run(b"40").await.unwrap();
        println!("{output:?}");
        let Output {
            status,
            stdout,
            stderr,
        } = &output;
        assert!(status.success());
        assert_eq!(
            stdout,
            b"{\"value\":42,\"error\":null,\"stdout\":\"hello\\n\",\"stderr\":\"\"}\n",
            "{:?}",
            String::from_utf8_lossy(stdout)
        );
        // assert_eq!(stderr, b"Installed 5 packages in 3ms\n");
        assert!(
            stderr
                .windows(b"Installed 5 packages in ".len())
                .any(|w| w == b"Installed 5 packages in ")
        );
        assert_eq!(
            structured,
            WithStdoutStderr {
                value: Ok(Value::Number(42.into())),
                stdout: "hello\n".to_string(),
                stderr: "".to_string(),
            }
        );
    }
    #[test]
    fn test_run_function_blocking() {
        let (function, _) = PythonFunction::simple();
        let (structured, output) = function.run_blocking(b"40").unwrap();
        println!("{output:?}");
        let Output {
            status,
            stdout,
            stderr,
        } = &output;
        assert!(status.success());
        assert_eq!(
            stdout,
            b"{\"value\":42,\"error\":null,\"stdout\":\"hello\\n\",\"stderr\":\"\"}\n",
            "{:?}",
            String::from_utf8_lossy(stdout)
        );
        // assert_eq!(stderr, b"Installed 5 packages in 3ms\n");
        assert!(
            stderr
                .windows(b"Installed 5 packages in ".len())
                .any(|w| w == b"Installed 5 packages in ")
        );
        assert_eq!(
            structured,
            WithStdoutStderr {
                value: Ok(Value::Number(42.into())),
                stdout: "hello\n".to_string(),
                stderr: "".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_run_typed_function() {
        let (function, _) = PythonFunction::simple();
        let (structured, output) = function.run_typed::<_, i32>(40).await.unwrap();
        println!("{output:?}");
        let Output {
            status,
            stdout,
            stderr,
        } = &output;
        assert!(status.success());
        assert_eq!(
            stdout,
            b"{\"value\":42,\"error\":null,\"stdout\":\"hello\\n\",\"stderr\":\"\"}\n",
            "{:?}",
            String::from_utf8_lossy(stdout)
        );
        // assert_eq!(stderr, b"Installed 5 packages in 3ms\n");
        assert!(
            stderr
                .windows(b"Installed 5 packages in ".len())
                .any(|w| w == b"Installed 5 packages in ")
        );
        assert_eq!(
            structured,
            WithStdoutStderr {
                value: Ok(42),
                stdout: "hello\n".to_string(),
                stderr: "".to_string(),
            }
        );
    }
    #[test]
    fn test_run_typed_function_blocking() {
        let (function, _) = PythonFunction::simple();
        let (structured, output) = function.run_typed_blocking::<_, i32>(40).unwrap();
        println!("{output:?}");
        let Output {
            status,
            stdout,
            stderr,
        } = &output;
        assert!(status.success());
        assert_eq!(
            stdout,
            b"{\"value\":42,\"error\":null,\"stdout\":\"hello\\n\",\"stderr\":\"\"}\n",
            "{:?}",
            String::from_utf8_lossy(stdout)
        );
        // assert_eq!(stderr, b"Installed 5 packages in 3ms\n");
        assert!(
            stderr
                .windows(b"Installed 5 packages in ".len())
                .any(|w| w == b"Installed 5 packages in ")
        );
        assert_eq!(
            structured,
            WithStdoutStderr {
                value: Ok(42),
                stdout: "hello\n".to_string(),
                stderr: "".to_string(),
            }
        );
    }
}

#[cfg(test)]
mod exception_tests {
    use super::*;

    const STDOUT: &str = "{\"value\":null,\"error\":\"Traceback (most recent call last):\\n  File \\\"/temp_folder/script.py\\\", line 60, in main\\n    result = process(input)\\n  File \\\"/temp_folder/script.py\\\", line 11, in process\\n    raise Exception('This is a test')\\nException: This is a test\\n\",\"stdout\":\"hello\\n\",\"stderr\":\"\"}\n";
    const ERROR: &str = r#"Traceback (most recent call last):
  File "/temp_folder/script.py", line 60, in main
    result = process(input)
  File "/temp_folder/script.py", line 11, in process
    raise Exception('This is a test')
Exception: This is a test
"#;

    #[tokio::test]
    async fn test_run_function_exception() {
        let (function, _) = PythonFunction::exception();
        let (structured, output) = function.run(b"40").await.unwrap();
        println!("{output:?}");
        let Output {
            status,
            stdout,
            stderr,
        } = &output;
        assert!(status.success());
        assert_eq!(
            stdout,
            STDOUT.as_bytes(),
            "{:?}",
            String::from_utf8_lossy(stdout)
        );
        // assert_eq!(stderr, b"Installed 5 packages in 3ms\n");
        assert!(
            stderr
                .windows(b"Installed 5 packages in ".len())
                .any(|w| w == b"Installed 5 packages in ")
        );
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
    fn test_run_function_exception_blocking() {
        let (function, _) = PythonFunction::exception();
        let (structured, output) = function.run_blocking(b"40").unwrap();
        println!("{output:?}");
        let Output {
            status,
            stdout,
            stderr,
        } = &output;
        assert!(status.success());
        assert_eq!(
            stdout,
            STDOUT.as_bytes(),
            "{:?}",
            String::from_utf8_lossy(stdout)
        );
        // assert_eq!(stderr, b"Installed 5 packages in 3ms\n");
        assert!(
            stderr
                .windows(b"Installed 5 packages in ".len())
                .any(|w| w == b"Installed 5 packages in ")
        );
        assert_eq!(
            structured,
            WithStdoutStderr {
                value: Err(ERROR.to_string()),
                stdout: "hello\n".to_string(),
                stderr: "".to_string(),
            }
        );
    }
}

#[cfg(test)]
mod invalid_tests {
    use super::*;

    #[tokio::test]
    async fn test_run_function_invalid() {
        let (function, _) = PythonFunction::invalid_function();
        match function.run(b"40").await {
            Ok((structured, output)) => {
                println!("{output:?}");
                println!("{structured:?}");
                unreachable!()
            }
            Err(e) => {
                println!("{e:?}")
            }
        }
    }

    #[test]
    fn test_run_function_invalid_import() {
        let (function, _) = PythonFunction::invalid_import();
        match function.run_blocking(b"40") {
            Ok((structured, output)) => {
                println!("{output:?}");
                println!("{structured:?}");
                unreachable!()
            }
            Err(e) => {
                println!("{e:?}")
            }
        }
    }
}
