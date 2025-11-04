# puv

A lightweight Rust crate for executing Python code using [PEP 723](https://peps.python.org/pep-0723/) inline script metadata. Built for scenarios where the convenience of Python outweighs the overhead of process spawning and JSON serialization.

## Quick rundown

- Uses Python's standardized inline metadata format for dependencies and version requirements.
- Uses [`uv`](https://github.com/astral-sh/uv) for Python version and dependency management.
- Automatic JSON serialization/deserialization with Serde/Pydantic validation.
- Use `PythonFunction::into_map` to reuse the process and initialisation.
- Captures and returns stdout, stderr, and Python exceptions for debugging.

## Usage

This crate allows you to run plain PEP 723 scripts, or scripts that expose a `process` function like this:

```python
# /// script
# requires-python = "==3.10"
# dependencies = ["numpy"]
# ///

import numpy as np
from pydantic import BaseModel

class Input(BaseModel):
    value: int

# 💡 Define a `process` function with a single input and a single output.
# Both input and output must be typed.
def process(x: Input) -> int:
    return x.value
```

### Example

```rust
let function = PythonFunction {
    function: r#"
# /// script
# requires-python = "==3.10"
# dependencies = ["numpy"]
# ///

import numpy as np

def process(x: int) -> list[int]:
    return np.array([x, x * 2, x * 3]).tolist()
"#.to_string(),
};

let (result, _raw_output) = function.run_typed::<i32, Vec<i32>>(10).await?;

match result.value {
    Ok(values) => println!("Result: {:?}", values),  // [10, 20, 30]
    Err(e) => eprintln!("Python error: {}", e), // If an exception is thrown
}

println!("stdout: {}", result.stdout);
println!("stderr: {}", result.stderr);
```

### Stateful example

For repeated calls to the same function, use `PythonMap` to keep the Python process alive:

```rust
let function = PythonFunction {
    function: "...".to_string(),
};

let mut map = function.into_map().await?;

// Multiple calls reuse the same Python process
let result1 = map.run_typed::<i32, i32>(40).await?;
let result2 = map.run_typed::<i32, i32>(100).await?;

println!("{:?}", result1.value);  // Ok(42)
println!("{:?}", result2.value);  // Ok(102)
```

### Complex Types with Pydantic

The crate automatically injects Pydantic for validation. Use Python type hints for structured I/O:

```rust
#[derive(Serialize)]
struct Input {
    values: Vec<i32>,
}

#[derive(Deserialize, Debug)]
struct Output {
    mean: f64,
    sum: i32,
}

let function = PythonFunction {
    function: r#"
# /// script
# requires-python = "==3.10"
# dependencies = ["pydantic"]
# ///

from pydantic import BaseModel

class Input(BaseModel):
    values: list[int]

class Output(BaseModel):
    mean: float
    sum: int

def process(input: Input) -> Output:
    return Output(
        mean=sum(input.values) / len(input.values),
        sum=sum(input.values)
    )
"#.to_string(),
};

let input = Input { values: vec![1, 2, 3, 4, 5] };
let (result, _) = function.run_typed::<Input, Output>(input).await?;

println!("{:?}", result.value);  // Ok(Output { mean: 3.0, sum: 15 })
```

## How It Works

1. Parses your Python code to extract type annotations from the `process` function.
2. Automatically adds Pydantic to dependencies if not present (used by the glue code).
3. Creates a wrapper script that:
   - Reads JSON from stdin,
   - Validates input with Pydantic,
   - Redirects stdout/stderr during execution,
   - Returns structured JSON with results and captured output.
4. Runs the script via `uv run`, which handles Python installation and dependency management.

## Requirements

- [uv](https://github.com/astral-sh/uv) must be installed and available in PATH.

## When to Use

**Good for:**
- Prototyping with Python libraries from Rust.
- Calls to Python for specific tasks, where the inputs/outputs are small compared to work done, or overhead doesn't matter (some forms of data processing, ML inference, ...).
- Scripts where Python's ecosystem advantage outweighs other problems.

**Bad for:**
- Hot paths where process/stdio overhead is a problem.
- Large inputs or outputs (serialization overhead).
- Situations requiring deep Python/Rust integration.
- `uv` is not available.