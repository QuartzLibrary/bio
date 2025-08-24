use std::{fmt, io, process::Output, sync::LazyLock};

use regex::Regex;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use super::script::PythonScript;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PythonFunction {
    pub python_version: String,
    pub dependencies: Vec<String>,
    pub function: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WithStdoutStderr<T> {
    pub value: T,
    pub stdout: String,
    pub stderr: String,
}
pub type PyFnOutput<T> = (WithStdoutStderr<Result<T, String>>, Output);

#[cfg(not(target_arch = "wasm32"))]
impl PythonFunction {
    pub async fn run(&self, input: impl AsRef<[u8]>) -> io::Result<PyFnOutput<Value>> {
        let output = self.script().run(input).await?;
        let structured: MaybeWithStdoutStderr<Value> = serde_json::from_slice(&output.stdout)?;
        let structured = structured.unpack()?;
        Ok((structured, output))
    }
    pub async fn run_typed<In, Out>(&self, input: In) -> io::Result<PyFnOutput<Out>>
    where
        In: Serialize,
        Out: DeserializeOwned + fmt::Debug,
    {
        let output = self
            .script()
            .run(&serde_json::to_vec(&input).unwrap())
            .await?;
        let value: MaybeWithStdoutStderr<Out> = serde_json::from_slice(&output.stdout).unwrap();
        let value = value.unpack().unwrap();
        Ok((value, output))
    }

    pub fn run_blocking(&self, input: impl AsRef<[u8]>) -> io::Result<PyFnOutput<Value>> {
        let output = self.script().run_blocking(input)?;
        let structured: MaybeWithStdoutStderr<Value> = serde_json::from_slice(&output.stdout)?;
        let structured = structured.unpack()?;
        Ok((structured, output))
    }
    pub fn run_typed_blocking<In, Out>(&self, input: In) -> io::Result<PyFnOutput<Out>>
    where
        In: Serialize,
        Out: DeserializeOwned + fmt::Debug,
    {
        let output = self.script().run_blocking(&serde_json::to_vec(&input)?)?;
        let value: MaybeWithStdoutStderr<Out> = serde_json::from_slice(&output.stdout)?;
        let value = value.unpack()?;
        Ok((value, output))
    }

    pub fn into_map(self) -> io::Result<super::map::PythonMap> {
        super::map::PythonMap::new(&self)
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl PythonFunction {
    fn script(&self) -> PythonScript {
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

    # Buffer all stdin
    raw_input = sys.stdin.read()

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
#[cfg(not(target_arch = "wasm32"))]
impl PythonFunction {
    pub(super) fn deps(&self) -> Vec<String> {
        let mut deps = self.dependencies.clone();
        if !deps.contains(&"pydantic".to_owned()) {
            deps.push("pydantic".to_owned());
        }
        deps
    }
    pub(super) fn output_and_parse_input(&self) -> (&str, String) {
        static FN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            let var = "[a-zA-Z0-9_-]+";
            let type_ = "[a-zA-Z0-9_-]+";
            Regex::new(&format!(
                r"def\s+process\({var}\s*:\s*(?<input>{type_})\)\s*->\s*(?<output>{type_}):\s*\n",
            ))
            .unwrap()
        });

        let capture = {
            let mut captures = FN_REGEX.captures_iter(&self.function);
            let capture = captures.next().unwrap();
            assert!(captures.next().is_none());
            capture
        };

        let input = capture.name("input").unwrap().as_str();
        let output = capture.name("output").unwrap().as_str();

        let input_is_model = Regex::new(&format!("class\\s+{input}:"))
            .unwrap()
            .is_match(&self.function);

        let parse_input = if input_is_model {
            format!("{input}.model_validate_json(raw_input)")
        } else {
            "json.loads(raw_input)".to_owned()
        };

        (output, parse_input)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    const PYTHON_VERSION: &str = "==3.10";
    const FUNCTION: &str = "
def process(input: int) -> int:
    print(\"hello\")
    return input + 2
";

    #[tokio::test]
    async fn test_run_function() {
        let function = PythonFunction {
            python_version: PYTHON_VERSION.to_string(),
            dependencies: vec![],
            function: FUNCTION.to_string(),
        };
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
            b"{\"value\":42,\"error\":null,\"stdout\":\"hello\\n\",\"stderr\":\"\"}\n"
        );
        // assert_eq!(stderr, b"Installed 5 packages in 3ms\n");
        assert!(stderr
            .windows(b"Installed 5 packages in ".len())
            .any(|w| w == b"Installed 5 packages in "));
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
        let function = PythonFunction {
            python_version: PYTHON_VERSION.to_string(),
            dependencies: vec![],
            function: FUNCTION.to_string(),
        };
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
            b"{\"value\":42,\"error\":null,\"stdout\":\"hello\\n\",\"stderr\":\"\"}\n"
        );
        // assert_eq!(stderr, b"Installed 5 packages in 3ms\n");
        assert!(stderr
            .windows(b"Installed 5 packages in ".len())
            .any(|w| w == b"Installed 5 packages in "));
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
        let function = PythonFunction {
            python_version: PYTHON_VERSION.to_string(),
            dependencies: vec![],
            function: FUNCTION.to_string(),
        };
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
            b"{\"value\":42,\"error\":null,\"stdout\":\"hello\\n\",\"stderr\":\"\"}\n"
        );
        // assert_eq!(stderr, b"Installed 5 packages in 3ms\n");
        assert!(stderr
            .windows(b"Installed 5 packages in ".len())
            .any(|w| w == b"Installed 5 packages in "));
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
        let function = PythonFunction {
            python_version: PYTHON_VERSION.to_string(),
            dependencies: vec![],
            function: FUNCTION.to_string(),
        };
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
            b"{\"value\":42,\"error\":null,\"stdout\":\"hello\\n\",\"stderr\":\"\"}\n"
        );
        // assert_eq!(stderr, b"Installed 5 packages in 3ms\n");
        assert!(stderr
            .windows(b"Installed 5 packages in ".len())
            .any(|w| w == b"Installed 5 packages in "));
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

    const PYTHON_VERSION: &str = "==3.10";
    const FUNCTION: &str = "
def process(input: int) -> int:
    print(\"hello\")
    raise Exception('This is a test')
";

    const STDOUT: &str = "{\"value\":null,\"error\":\"Traceback (most recent call last):\\n  File \\\"/temp_folder/script.py\\\", line 47, in main\\n    result = process(input)\\n  File \\\"/temp_folder/script.py\\\", line 19, in process\\n    raise Exception('This is a test')\\nException: This is a test\\n\",\"stdout\":\"hello\\n\",\"stderr\":\"\"}\n";
    const ERROR: &str = "Traceback (most recent call last):\n  File \"/temp_folder/script.py\", line 47, in main\n    result = process(input)\n  File \"/temp_folder/script.py\", line 19, in process\n    raise Exception('This is a test')\nException: This is a test\n";

    #[tokio::test]
    async fn test_run_function_exception() {
        let function = PythonFunction {
            python_version: PYTHON_VERSION.to_string(),
            dependencies: vec![],
            function: FUNCTION.to_string(),
        };
        let (structured, output) = function.run(b"40").await.unwrap();
        println!("{output:?}");
        let Output {
            status,
            stdout,
            stderr,
        } = &output;
        assert!(status.success());
        assert_eq!(stdout, STDOUT.as_bytes(),);
        // assert_eq!(stderr, b"Installed 5 packages in 3ms\n");
        assert!(stderr
            .windows(b"Installed 5 packages in ".len())
            .any(|w| w == b"Installed 5 packages in "));
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
        let function = PythonFunction {
            python_version: PYTHON_VERSION.to_string(),
            dependencies: vec![],
            function: FUNCTION.to_string(),
        };
        let (structured, output) = function.run_blocking(b"40").unwrap();
        println!("{output:?}");
        let Output {
            status,
            stdout,
            stderr,
        } = &output;
        assert!(status.success());
        assert_eq!(stdout, STDOUT.as_bytes());
        // assert_eq!(stderr, b"Installed 5 packages in 3ms\n");
        assert!(stderr
            .windows(b"Installed 5 packages in ".len())
            .any(|w| w == b"Installed 5 packages in "));
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
