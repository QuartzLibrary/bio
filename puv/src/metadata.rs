use std::str::FromStr;

use uv_pep440::VersionSpecifier;
use uv_pep508::{Pep508Error, Requirement, VerbatimUrl, VersionOrUrl};
use uv_workspace::pyproject_mut;

use self::uv_scripts_vendor::Pep723Script;

#[derive(Debug, thiserror::Error)]
pub enum ScriptMetadataError {
    #[error("Script is missing PEP 723 metadata")]
    MissingMetadata,
    #[error("{0}")]
    InvalidPythonVersion(String),
    #[error("PEP 508 error: {0}")]
    InvalidDependency(Pep508Error),
    #[error("PEP 723 error: {0}")]
    Pep723(uv_scripts::Pep723Error),
    #[error("TOML edit error: {0}")]
    TomlEdit(pyproject_mut::Error),
}

pub fn script_python_version(
    script_content: &str,
) -> Result<VersionSpecifier, ScriptMetadataError> {
    let script = Pep723Script::parse(script_content.as_bytes())?
        .ok_or(ScriptMetadataError::MissingMetadata)?;
    let specifiers =
        script
            .metadata
            .requires_python
            .ok_or(ScriptMetadataError::InvalidPythonVersion(
                "Script is missing Python version".to_string(),
            ))?;

    if specifiers.len() != 1 {
        return Err(ScriptMetadataError::InvalidPythonVersion(format!(
            "Script has multiple Python versions: {specifiers}",
        )));
    }

    specifiers
        .into_iter()
        .next()
        .ok_or(ScriptMetadataError::InvalidPythonVersion(
            "Script has no Python version".to_string(),
        ))
}

pub fn script_dependencies(
    script_content: &str,
) -> Result<Vec<Requirement<VerbatimUrl>>, ScriptMetadataError> {
    let script = Pep723Script::parse(script_content.as_bytes())?
        .ok_or(ScriptMetadataError::MissingMetadata)?;
    Ok(script
        .metadata
        .dependencies
        .unwrap_or_default()
        .into_iter()
        // We drop the parsed url to avoid needing more crates,
        // most of the time the user will want to just print it anyway.
        .map(
            |Requirement {
                 name,
                 extras,
                 version_or_url,
                 marker,
                 origin,
             }| Requirement {
                name,
                extras,
                version_or_url: version_or_url.map(|v| match v {
                    VersionOrUrl::VersionSpecifier(version_specifiers) => {
                        VersionOrUrl::VersionSpecifier(version_specifiers)
                    }
                    VersionOrUrl::Url(url) => VersionOrUrl::Url(url.verbatim),
                }),
                marker,
                origin,
            },
        )
        .collect())
}

/// Injects a dependency into a Python script with PEP 723 inline metadata.
///
/// TODO: preserve whitespace and line count.
pub fn inject_dependency(
    script_content: &str,
    dependency: &str,
) -> Result<String, ScriptMetadataError> {
    let mut script = Pep723Script::parse(script_content.as_bytes())?
        .ok_or(ScriptMetadataError::MissingMetadata)?;

    let req = Requirement::from_str(dependency).map_err(ScriptMetadataError::InvalidDependency)?;

    if let Some(deps) = &script.metadata.dependencies
        && deps.iter().any(|dep| dep.name == req.name)
    {
        return Ok(script_content.to_string());
    }

    script.add_dependency(&req)?;

    Ok(script.content())
}

impl Pep723Script {
    fn add_dependency(&mut self, req: &Requirement) -> Result<(), pyproject_mut::Error> {
        let mut toml = pyproject_mut::PyProjectTomlMut::from_toml(
            &self.metadata.raw,
            pyproject_mut::DependencyTarget::Script,
        )?;

        toml.add_dependency(req, None, false)?;

        self.metadata = toml.to_string().parse().expect("valid metadata");

        Ok(())
    }
}

/// A modified version of the `uv-scripts` crate to manipulate the script directly without file access.
///
/// https://github.com/astral-sh/uv/blob/0adb444806e8bcea7e7a5e9ae90d1288778a0b54/crates/uv-scripts/src/lib.rs
mod uv_scripts_vendor {
    use std::{str::FromStr, sync::LazyLock};

    use memchr::memmem::Finder;
    use uv_scripts::{Pep723Error, Pep723Metadata};

    static FINDER: LazyLock<Finder> = LazyLock::new(|| Finder::new(b"# /// script"));

    /// A PEP 723 script, including its [`Pep723Metadata`].
    #[derive(Debug, Clone)]
    pub(super) struct Pep723Script {
        /// The parsed [`Pep723Metadata`] table from the script.
        pub metadata: Pep723Metadata,
        /// The content of the script before the metadata table.
        pub prelude: String,
        /// The content of the script after the metadata table.
        pub postlude: String,
    }

    impl Pep723Script {
        /// Parse the PEP 723 `script` metadata from a Python file, if it exists.
        ///
        /// Returns `None` if the file is missing a PEP 723 metadata block.
        ///
        /// See: <https://peps.python.org/pep-0723/>
        pub fn parse(contents: &[u8]) -> Result<Option<Self>, Pep723Error> {
            let ScriptTag {
                prelude,
                metadata,
                postlude,
            } = match ScriptTag::parse(contents)? {
                Some(tag) => tag,
                None => return Ok(None),
            };

            let metadata = Pep723Metadata::from_str(&metadata)?;

            Ok(Some(Self {
                metadata,
                prelude,
                postlude,
            }))
        }

        /// Replace the existing metadata in the file with new metadata and write the updated content.
        pub fn content(&self) -> String {
            format!(
                "{}{}{}",
                self.prelude,
                serialize_metadata(&self.metadata.raw),
                self.postlude
            )
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct ScriptTag {
        /// The content of the script before the metadata block.
        pub prelude: String,
        /// The metadata block.
        pub metadata: String,
        /// The content of the script after the metadata block.
        pub postlude: String,
    }

    impl ScriptTag {
        /// Given the contents of a Python file, extract the `script` metadata block with leading
        /// comment hashes removed, any preceding shebang or content (prelude), and the remaining Python
        /// script.
        ///
        /// Given the following input string representing the contents of a Python script:
        ///
        /// ```python
        /// #!/usr/bin/env python3
        /// # /// script
        /// # requires-python = '>=3.11'
        /// # dependencies = [
        /// #   'requests<3',
        /// #   'rich',
        /// # ]
        /// # ///
        ///
        /// import requests
        ///
        /// print("Hello, World!")
        /// ```
        ///
        /// This function would return:
        ///
        /// - Preamble: `#!/usr/bin/env python3\n`
        /// - Metadata: `requires-python = '>=3.11'\ndependencies = [\n  'requests<3',\n  'rich',\n]`
        /// - Postlude: `import requests\n\nprint("Hello, World!")\n`
        ///
        /// See: <https://peps.python.org/pep-0723/>
        pub fn parse(contents: &[u8]) -> Result<Option<Self>, Pep723Error> {
            // Identify the opening pragma.
            let Some(index) = FINDER.find(contents) else {
                return Ok(None);
            };

            // The opening pragma must be the first line, or immediately preceded by a newline.
            if !(index == 0 || matches!(contents[index - 1], b'\r' | b'\n')) {
                return Ok(None);
            }

            // Extract the preceding content.
            let prelude = std::str::from_utf8(&contents[..index])?;

            // Decode as UTF-8.
            let contents = &contents[index..];
            let contents = std::str::from_utf8(contents)?;

            let mut lines = contents.lines();

            // Ensure that the first line is exactly `# /// script`.
            if lines.next().is_none_or(|line| line != "# /// script") {
                return Ok(None);
            }

            // > Every line between these two lines (# /// TYPE and # ///) MUST be a comment starting
            // > with #. If there are characters after the # then the first character MUST be a space. The
            // > embedded content is formed by taking away the first two characters of each line if the
            // > second character is a space, otherwise just the first character (which means the line
            // > consists of only a single #).
            let mut toml = vec![];

            for line in lines {
                // Remove the leading `#`.
                let Some(line) = line.strip_prefix('#') else {
                    break;
                };

                // If the line is empty, continue.
                if line.is_empty() {
                    toml.push("");
                    continue;
                }

                // Otherwise, the line _must_ start with ` `.
                let Some(line) = line.strip_prefix(' ') else {
                    break;
                };

                toml.push(line);
            }

            // Find the closing `# ///`. The precedence is such that we need to identify the _last_ such
            // line.
            //
            // For example, given:
            // ```python
            // # /// script
            // #
            // # ///
            // #
            // # ///
            // ```
            //
            // The latter `///` is the closing pragma
            let Some(index) = toml.iter().rev().position(|line| *line == "///") else {
                return Err(Pep723Error::UnclosedBlock);
            };
            let index = toml.len() - index;

            // Discard any lines after the closing `# ///`.
            //
            // For example, given:
            // ```python
            // # /// script
            // #
            // # ///
            // #
            // #
            // ```
            //
            // We need to discard the last two lines.
            toml.truncate(index - 1);

            // Join the lines into a single string.
            let prelude = prelude.to_string();
            let metadata = toml.join("\n") + "\n";
            let postlude = contents
                .lines()
                .skip(index + 1)
                .collect::<Vec<_>>()
                .join("\n")
                + "\n";

            Ok(Some(Self {
                prelude,
                metadata,
                postlude,
            }))
        }
    }

    /// Formats the provided metadata by prefixing each line with `#` and wrapping it with script markers.
    fn serialize_metadata(metadata: &str) -> String {
        let mut output = String::with_capacity(metadata.len() + 32);

        output.push_str("# /// script");
        output.push('\n');

        for line in metadata.lines() {
            output.push('#');
            if !line.is_empty() {
                output.push(' ');
                output.push_str(line);
            }
            output.push('\n');
        }

        output.push_str("# ///");
        output.push('\n');

        output
    }
}

impl From<uv_scripts::Pep723Error> for ScriptMetadataError {
    fn from(e: uv_scripts::Pep723Error) -> Self {
        Self::Pep723(e)
    }
}
impl From<pyproject_mut::Error> for ScriptMetadataError {
    fn from(e: pyproject_mut::Error) -> Self {
        Self::TomlEdit(e)
    }
}

// TODO: remove once other modules have error types.
impl From<ScriptMetadataError> for std::io::Error {
    fn from(e: ScriptMetadataError) -> Self {
        std::io::Error::other(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TESTS: &[(&str, &str, &str)] = &[
        (
            r#"# /// script
# requires-python = ">=3.8"
# dependencies = [
#     "numpy",
# ]
# ///
"#,
            "pandas>=2.0.0",
            r#"# /// script
# requires-python = ">=3.8"
# dependencies = [
#     "numpy",
#     "pandas>=2.0.0",
# ]
# ///

"#,
        ),
        // Empty dependencies array
        (
            r#"# /// script
# requires-python = ">=3.8"
# dependencies = []
# ///
"#,
            "requests",
            r#"# /// script
# requires-python = ">=3.8"
# dependencies = [
#     "requests",
# ]
# ///

"#,
        ),
        // Script with shebang and code
        (
            r#"#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "flask",
# ]
# ///
import flask

print("Hello")
"#,
            "jinja2>=3.0",
            r#"#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "flask",
#     "jinja2>=3.0",
# ]
# ///
import flask

print("Hello")
"#,
        ),
        // No dependencies field yet
        (
            r#"# /// script
# requires-python = ">=3.9"
# ///
"#,
            "click",
            r#"# /// script
# requires-python = ">=3.9"
# dependencies = [
#     "click",
# ]
# ///

"#,
        ),
        // Different version specifier
        (
            r#"# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "django==4.2",
# ]
# ///
"#,
            "psycopg2-binary~=2.9.0",
            r#"# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "django==4.2",
#     "psycopg2-binary~=2.9.0",
# ]
# ///

"#,
        ),
    ];

    #[test]
    fn test_inject_dependency_simple() {
        for (script, dependency, expected) in TESTS {
            println!("script: {script}");
            println!("dependency: {dependency}");
            println!("expected: {expected}");
            let result = inject_dependency(script, dependency).unwrap();
            assert_eq!(&result, expected);
        }
    }

    #[test]
    fn test_inject_duplicate_dependency() {
        let script = r#"# /// script
# requires-python = ">=3.8"
# dependencies = [
#     "numpy>=1.20",
# ]
# ///
"#;
        // Try to add numpy again (different version spec, same package name)
        let result = inject_dependency(script, "numpy>=2.0").unwrap();
        // Should return unchanged script
        assert_eq!(result, script);
    }
}
