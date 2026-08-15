// TODO: explore encoding preserving ordering and prefix validity.

use std::path::{Component, Path, PathBuf};

use url::Url;

const RESERVED_NAMES_WINDOWS: [&str; 30] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "COM¹", "COM²", "COM³", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8",
    "LPT9", "LPT¹", "LPT²", "LPT³", "CONIN$", "CONOUT$",
];
const RESERVED_NAMES_UNIX: [&str; 2] = [".", ".."];

/// First as it's used by the others.
const PERCENT: (char, &str) = ('%', "%25");

/// Distinguishes URL slash modes. Raw control characters cannot occur in a serialized URL.
const URL_SLASH_MARKER: char = '\0';

// https://stackoverflow.com/questions/1976007/what-characters-are-forbidden-in-windows-and-linux-directory-names
// https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file
const ESCAPED_CHARS: [(char, &str); 40] = [
    ('\0', "%00"),     // NUL
    ('\u{01}', "%01"), // SOH
    ('\u{02}', "%02"), // STX
    ('\u{03}', "%03"), // ETX
    ('\u{04}', "%04"), // EOT
    ('\u{05}', "%05"), // ENQ
    ('\u{06}', "%06"), // ACK
    ('\u{07}', "%07"), // BEL
    ('\u{08}', "%08"), // BS
    ('\t', "%09"),     // HT (Tab)
    ('\n', "%0a"),     // LF
    ('\u{0B}', "%0b"), // VT
    ('\u{0C}', "%0c"), // FF
    ('\r', "%0d"),     // CR
    ('\u{0E}', "%0e"), // SO
    ('\u{0F}', "%0f"), // SI
    ('\u{10}', "%10"), // DLE
    ('\u{11}', "%11"), // DC1
    ('\u{12}', "%12"), // DC2
    ('\u{13}', "%13"), // DC3
    ('\u{14}', "%14"), // DC4
    ('\u{15}', "%15"), // NAK
    ('\u{16}', "%16"), // SYN
    ('\u{17}', "%17"), // ETB
    ('\u{18}', "%18"), // CAN
    ('\u{19}', "%19"), // EM
    ('\u{1A}', "%1a"), // SUB
    ('\u{1B}', "%1b"), // ESC
    ('\u{1C}', "%1c"), // FS
    ('\u{1D}', "%1d"), // GS
    ('\u{1E}', "%1e"), // RS
    ('\u{1F}', "%1f"), // US
    ('<', "%3c"),
    ('>', "%3e"),
    (':', "%3a"),
    ('"', "%22"),
    ('\\', "%5c"),
    ('|', "%7c"),
    ('?', "%3f"),
    ('*', "%2a"),
];

/// Encode and decode the value into a path.
///
/// - Guaranteed to round-trip.
/// - Different types might use different schemes to maximize legibility.
///
/// Note: while the implementation is guaranteed to round-trip,
/// case-insensitive filesystems might not preserve the original case.
pub trait SafePath: Sized {
    fn to_safe_path(&self) -> PathBuf;
    fn from_safe_path(path: &Path) -> Option<Self>;

    /// A prefix that is guaranteed to prefix all encoded values that the original does.
    ///
    /// This is not an exact equivalent because correctly encoding for Windows while
    /// balancing readability causes some values to be conditionally encoded.
    fn to_loose_prefix(&self) -> PathBuf {
        fn trailing_separator(path: &Path) -> bool {
            path.as_os_str()
                .as_encoded_bytes()
                .last()
                .copied()
                .is_some_and(|s| b"/\\".contains(&s))
        }
        let path = self.to_safe_path();
        if trailing_separator(&path) {
            path
        } else {
            let Some(path) = path.parent() else {
                return PathBuf::new();
            };
            if !trailing_separator(path) {
                path.join("")
            } else {
                path.to_path_buf()
            }
        }
    }
}

impl SafePath for String {
    fn to_safe_path(&self) -> PathBuf {
        to_safe_path(self)
    }

    fn from_safe_path(path: &Path) -> Option<Self> {
        from_safe_path(path)
    }
}

impl SafePath for Url {
    fn to_safe_path(&self) -> PathBuf {
        let scheme = self.scheme();
        let string = self.to_string();
        let rest = string
            .strip_prefix(scheme)
            .unwrap()
            .strip_prefix(':')
            .unwrap();
        let leading_slashes = rest.bytes().take_while(|byte| *byte == b'/').count();

        to_safe_path(&match leading_slashes {
            0 => format!("{scheme}{URL_SLASH_MARKER}{rest}"),
            1 => format!("{scheme}/{URL_SLASH_MARKER}{}", &rest[1..]),
            2 => format!("{scheme}/{}", &rest[2..]),
            _ => format!("{scheme}/{URL_SLASH_MARKER}{rest}"),
        })
    }

    fn from_safe_path(path: &Path) -> Option<Self> {
        let decoded = from_safe_path(path)?;
        let url = if let Some((before_marker, after_marker)) = decoded.split_once(URL_SLASH_MARKER)
        {
            if let Some(scheme) = before_marker.strip_suffix('/') {
                if after_marker.starts_with('/') {
                    format!("{scheme}:{after_marker}")
                } else {
                    format!("{scheme}:/{after_marker}")
                }
            } else {
                format!("{before_marker}:{after_marker}")
            }
        } else {
            let (scheme, rest) = decoded.split_once('/')?;
            format!("{scheme}://{rest}")
        };

        Self::parse(&url).ok()
    }
}

/// Converts any string to a file-system safe path string that can be used across
/// major operating systems (Windows, macOS, Linux).
fn to_safe_path(input: &str) -> PathBuf {
    let mut result = input.replace(PERCENT.0, PERCENT.1);

    for (char, encoded) in ESCAPED_CHARS {
        result = result.replace(char, encoded);
    }

    let fragments: Vec<String> = result.split('/').map(encode_component).collect();

    fragments.join("/").into()
}
fn encode_component(s: &str) -> String {
    let mut result = s.to_string();

    if is_reserved_component(s) {
        result.insert(0, '%');
    }
    if s.is_empty() || s.ends_with('.') || s.ends_with(' ') {
        result.push_str("%END");
    }

    result
}
fn is_reserved_component(s: &str) -> bool {
    let uppercase = s.to_uppercase();
    let stem = uppercase.split('.').next().unwrap();

    RESERVED_NAMES_WINDOWS.contains(&stem) || RESERVED_NAMES_UNIX.contains(&uppercase.as_str())
}
fn from_safe_path(path: &Path) -> Option<String> {
    let path: Vec<String> = path
        .components()
        .map(|c| {
            Some(match c {
                Component::Prefix(_) => return None,
                Component::RootDir => "".to_owned(),
                Component::CurDir => ".".to_owned(),
                Component::ParentDir => "..".to_owned(),
                Component::Normal(os_str) => decode_component(os_str.to_str()?),
            })
        })
        .collect::<Option<Vec<_>>>()?;

    let mut path = path.join("/");

    for (char, encoded) in ESCAPED_CHARS {
        path = path.replace(encoded, &char.to_string());
    }

    let result = path.replace(PERCENT.1, &PERCENT.0.to_string());

    Some(result)
}
fn decode_component(c: &str) -> String {
    let c = c.strip_suffix("%END").unwrap_or(c);

    if let Some(s) = c.strip_prefix('%')
        && is_reserved_component(s)
    {
        s.to_owned()
    } else {
        c.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use rand::{Rng, RngExt, SeedableRng, rngs::SmallRng, seq::IndexedRandom};

    use super::*;

    const STRING_TEST_CASES: [(&str, &str); 5] = [
        ("simple_string", "simple_string"),
        ("path/with/slashes", "path/with/slashes"),
        (
            "st//ring<with>spe/cial:chars\"\\|?*chars[]",
            "st/%END/ring%3cwith%3espe/cial%3achars%22%5c%7c%3f%2achars[]",
        ),
        (
            "path /with./ some spaces.txt",
            "path %END/with.%END/ some spaces.txt",
        ),
        (
            "AUX/CON/PRN/NUL/COM1/LPT1",
            "%AUX/%CON/%PRN/%NUL/%COM1/%LPT1",
        ),
    ];

    #[test]
    fn string_unit_tests() {
        for (input, expected) in STRING_TEST_CASES {
            assert_full_string(input);

            let encoded = to_safe_path(input);
            assert_eq!(encoded.to_str().unwrap(), expected);
        }
    }

    #[test]
    fn test_string_examples() {
        for (char, encoded) in ESCAPED_CHARS {
            assert_full_string(&char.to_string());
            assert_eq!(format!("%{:02x}", char as u8), encoded);
        }

        for reserved in RESERVED_NAMES_WINDOWS {
            for reserved in [
                reserved.to_string(),
                reserved_name_mixed_case_windows(reserved),
                reserved.to_ascii_lowercase(),
            ] {
                for suffix in ["", ".txt", ".tar.gz", ".", ".txt.", ".txt "] {
                    assert_full_string(&format!("{reserved}{suffix}"));
                }
            }
        }

        for reserved in RESERVED_NAMES_UNIX {
            assert_full_string(reserved);
        }

        for input in [
            "",
            "/",
            "//",
            "/leading",
            "trailing/",
            "two//components",
            ".",
            "..",
            "...",
            "name.",
            "name ",
            "name. ",
            "./../",
            "C:",
            "C:/path",
            "\\\\server\\share",
            "\\\\?\\C:\\path",
        ] {
            assert_full_string(input);
        }

        for marker in [PERCENT.1, "%END"]
            .into_iter()
            .chain(ESCAPED_CHARS.map(|(_, encoded)| encoded))
        {
            assert_full_string(marker);
            assert_full_string(&format!("prefix{marker}suffix"));
            assert_full_string(&format!("{marker}."));
        }

        for forbidden in (0..=31)
            .map(char::from)
            .chain(['<', '>', ':', '"', '\\', '|', '?', '*'])
        {
            assert_full_string(&format!("before{forbidden}after"));
        }
    }

    const TEST_CASES_URL: [(&str, Option<&str>, &str); 28] = [
        (
            "https://example.com/path/to/resource",
            None,
            "https/example.com/path/to/resource",
        ),
        (
            "https://example.com/path?query=value&other=value#fragment",
            None,
            "https/example.com/path%3fquery=value&other=value#fragment",
        ),
        (
            "https://example.com:8080/path",
            None,
            "https/example.com%3a8080/path",
        ),
        (
            "https://example.com/path/to/resource?query=value#fragment",
            None,
            "https/example.com/path/to/resource%3fquery=value#fragment",
        ),
        (
            "https://example.com/CON/PRN/aux/nul.txt",
            None,
            "https/example.com/%CON/%PRN/%aux/%nul.txt",
        ),
        (
            "https:example",
            Some("https://example/"),
            "https/example/%END",
        ),
        ("data:text/plain,Stuff", None, "data%00text/plain,Stuff"),
        ("unix:/run/foo.socket", None, "unix/%00run/foo.socket"),
        // URLs with authentication
        (
            "https://user:password@example.com/path",
            None,
            "https/user%3apassword@example.com/path",
        ),
        // IPv4 and IPv6 addresses
        ("https://192.168.1.1/path", None, "https/192.168.1.1/path"),
        (
            "https://[2001:db8::1]/path",
            None,
            "https/[2001%3adb8%3a%3a1]/path",
        ),
        // Internationalized domain names
        (
            "https://例子.测试/path",
            Some("https://xn--fsqu00a.xn--0zwm56d/path"),
            "https/xn--fsqu00a.xn--0zwm56d/path",
        ),
        // URLs with Unicode characters
        (
            "https://example.com/ünicode/path",
            Some("https://example.com/%C3%BCnicode/path"),
            "https/example.com/%25C3%25BCnicode/path",
        ),
        // URLs with percent-encoded sequences already
        (
            "https://example.com/path%20with%20spaces",
            None,
            "https/example.com/path%2520with%2520spaces",
        ),
        // Edge case: empty path segments
        (
            "https://example.com//empty//segments",
            None,
            "https/example.com/%END/empty/%END/segments",
        ),
        // File URLs (absolute and relative)
        (
            "file:///path/to/file",
            None,
            "file/%00/%END/%END/path/to/file",
        ),
        (
            "file://localhost/path/to/file",
            Some("file:///path/to/file"),
            "file/%00/%END/%END/path/to/file",
        ),
        (
            "file:relative/path",
            Some("file:///relative/path"),
            "file/%00/%END/%END/relative/path",
        ),
        // Mailto URLs
        ("mailto:user@example.com", None, "mailto%00user@example.com"),
        // URLs with fragments only
        (
            "https://example.com#fragment",
            Some("https://example.com/#fragment"),
            "https/example.com/#fragment",
        ),
        // URLs with query only
        (
            "https://example.com?query=value",
            Some("https://example.com/?query=value"),
            "https/example.com/%3fquery=value",
        ),
        // URLs with special characters in different parts
        (
            "https://example.com/path?q=a:b&c=d*e",
            None,
            "https/example.com/path%3fq=a%3ab&c=d%2ae",
        ),
        // URLs with reserved filenames in different parts
        (
            "https://example.com?file=CON&type=device",
            Some("https://example.com/?file=CON&type=device"),
            "https/example.com/%3ffile=CON&type=device",
        ),
        // Edge cases with '.' and '..'
        (
            "https://example.com/./path/../resource",
            Some("https://example.com/resource"),
            "https/example.com/resource",
        ),
        // URLs ending with spaces or dots
        (
            "https://example.com/file. ",
            Some("https://example.com/file."),
            "https/example.com/file.%END",
        ),
        // URL with percent character
        (
            "https://example.com/percent%value",
            None,
            "https/example.com/percent%25value",
        ),
        // URLs with unusual schemes
        (
            "git+ssh://git@github.com/user/repo.git",
            None,
            "git+ssh/git@github.com/user/repo.git",
        ),
        // URLs with very long paths
        (
            "https://example.com/very/long/path/with/many/segments/to/test/handling/of/long/paths",
            None,
            "https/example.com/very/long/path/with/many/segments/to/test/handling/of/long/paths",
        ),
    ];

    #[test]
    fn url_unit_tests() {
        for (raw, parsed, expected) in TEST_CASES_URL {
            let input = Url::parse(raw).unwrap();

            if let Some(parsed) = parsed {
                assert_eq!(input.as_str(), parsed);
            } else {
                assert_eq!(input.as_str(), raw);
            }

            assert_full_url(&input);

            let encoded = input.to_safe_path();
            assert_eq!(encoded.to_str().unwrap(), expected);
        }
    }

    #[test]
    fn test_url_examples() {
        for scheme in RESERVED_NAMES_WINDOWS
            .into_iter()
            .filter(|name| name.chars().all(|char| char.is_ascii_alphanumeric()))
        {
            for suffix in ["", ".device", "."] {
                let input = Url::parse(&format!(
                    "{}{suffix}://example.com/path",
                    scheme.to_ascii_lowercase()
                ))
                .unwrap();
                assert_full_url(&input);
            }
        }

        #[expect(clippy::single_element_loop)]
        for url in [Url::parse("custom.://example.com/path").unwrap()] {
            assert_full_url(&url);
        }

        for (raw_prefix, raw_value) in [
            ("custom:", "custom://example.com/path"),
            ("custom:/", "custom://example.com/path"),
            ("m://", "m://example.com/path"),
            ("m://", "m:///path"),
        ] {
            let prefix = Url::parse(raw_prefix).unwrap();
            let value = Url::parse(raw_value).unwrap();
            assert!(value.as_str().starts_with(prefix.as_str()));

            assert_full_url(&prefix);
            assert_full_url(&value);

            let loose_prefix = prefix
                .to_loose_prefix()
                .as_os_str()
                .as_encoded_bytes()
                .to_vec();
            let encoded = value.to_safe_path().as_os_str().as_encoded_bytes().to_vec();

            assert!(
                encoded.starts_with(&loose_prefix),
                "{value} encoded as {encoded:?}, which does not start with {prefix}'s loose prefix {loose_prefix:?}"
            );
        }

        for (raw, expected) in [
            ("custom:value", "custom%00value"),
            ("custom:/value", "custom/%00value"),
            ("custom://example.com/value", "custom/example.com/value"),
            ("custom:///value", "custom/%00/%END/%END/value"),
            ("custom:////value", "custom/%00/%END/%END/%END/value"),
        ] {
            let url = Url::parse(raw).unwrap();
            assert_eq!(url.as_str(), raw);

            assert_full_url(&url);

            let encoded = url.to_safe_path();
            assert_eq!(encoded.to_str(), Some(expected));
            assert_eq!(Url::from_safe_path(&encoded), Some(url));
        }
    }

    // Fuzz

    #[test]
    fn test_fuzz_random_strings() {
        let mut rng = SmallRng::seed_from_u64(42);

        for _ in 0..100_000 {
            let random_string = random_string(&mut rng);
            assert_full_string(&random_string);
        }
    }

    #[test]
    fn test_fuzz_random_urls() {
        let mut rng = SmallRng::seed_from_u64(42);

        for _ in 0..10_000 {
            let value = random_url(&mut rng);
            assert_full_url(&value);
        }
    }

    // Random generation

    fn random_string(rng: &mut impl Rng) -> String {
        let len = rng.random_range(0..5);
        (0..len)
            .map(|_| random_component(rng))
            .collect::<Vec<String>>()
            .join("/")
    }
    fn random_component(rng: &mut impl Rng) -> String {
        match rng.random_range(0..6) {
            0 => (0..rng.random_range(0..10))
                .map(|_| random_char(rng))
                .collect(),
            1 => {
                let reserved = RESERVED_NAMES_WINDOWS.choose(rng).unwrap();
                let mut component = if rng.random_bool(0.5) {
                    reserved.to_ascii_lowercase()
                } else {
                    reserved.to_string()
                };
                component.push_str(["", ".txt", ".", " "].choose(rng).unwrap());
                component
            }
            2 => RESERVED_NAMES_UNIX.choose(rng).unwrap().to_string(),
            3 => {
                let mut component: String = (0..rng.random_range(0..10))
                    .map(|_| random_char(rng))
                    .collect();
                component.push(*['.', ' '].choose(rng).unwrap());
                component
            }
            4 => match rng.random_range(0..ESCAPED_CHARS.len() + 2) {
                0 => PERCENT.1.to_string(),
                1 => "%END".to_string(),
                index => ESCAPED_CHARS[index - 2].1.to_string(),
            },
            5 => String::new(),
            _ => unreachable!(),
        }
    }
    fn random_char(rng: &mut impl Rng) -> char {
        match rng.random_range(0..5) {
            0 => rng.random(),
            1 => '%',
            2 => ' ',
            3 => '.',
            4 => ESCAPED_CHARS.choose(rng).unwrap().0,
            _ => unreachable!(),
        }
    }

    fn random_url(rng: &mut impl Rng) -> Url {
        let tail = random_url_path(rng);
        let scheme = match rng.random_range(0..2) {
            0 => "custom",
            1 => simple_reserved_names_windows().choose(rng).unwrap(),
            2 => RESERVED_NAMES_UNIX.choose(rng).unwrap(),
            _ => unreachable!(),
        };
        let raw_value = match rng.random_range(0..5) {
            0 => format!("{scheme}:{tail}"),
            1 => format!("{scheme}:/{tail}"),
            2 => format!("{scheme}://example.com/{tail}"),
            3 => format!("{scheme}:///{tail}"),
            4 => format!("{scheme}:////{tail}"),
            _ => unreachable!(),
        };
        Url::parse(&raw_value).unwrap()
    }

    fn random_url_path(rng: &mut impl Rng) -> String {
        (0..rng.random_range(0..5))
            .map(|_| {
                url::form_urlencoded::byte_serialize(random_component(rng).as_bytes()).collect()
            })
            .collect::<Vec<String>>()
            .join("/")
    }

    fn reserved_name_mixed_case_windows(name: &str) -> String {
        name.chars()
            .enumerate()
            .map(|(index, char)| {
                if index % 2 == 0 {
                    char.to_ascii_lowercase()
                } else {
                    char
                }
            })
            .collect()
    }

    // Assert helpers

    fn assert_full_url(value: &Url) {
        assert_safe_round_trip(value.clone());

        let encoded = value.to_safe_path();
        let encoded_bytes = encoded.as_os_str().as_encoded_bytes();

        let mut raw_prefix = value.as_str();

        loop {
            if let Ok(prefix) = Url::parse(raw_prefix)
                && value.as_str().starts_with(prefix.as_str())
            {
                let loose_prefix = prefix
                    .to_loose_prefix()
                    .as_os_str()
                    .as_encoded_bytes()
                    .to_vec();

                assert!(
                    encoded_bytes.starts_with(&loose_prefix),
                    "{value} encoded as {encoded:?}, which does not start with {prefix}'s loose prefix {loose_prefix:?}"
                );
            }

            let Some((index, _)) = raw_prefix.char_indices().next_back() else {
                break;
            };
            raw_prefix = &raw_prefix[..index];
        }
    }

    fn assert_full_string(value: &str) {
        let value = value.to_string();

        assert_safe_round_trip(value.clone());

        let encoded = value.to_safe_path();
        let encoded = encoded.as_os_str().as_encoded_bytes();

        let mut prefix = value.as_str();
        loop {
            let loose_prefix = prefix
                .to_string()
                .to_loose_prefix()
                .as_os_str()
                .as_encoded_bytes()
                .to_vec();

            assert!(
                encoded.starts_with(&loose_prefix),
                "encoded value does not start with loose prefix: {prefix:?} of {value:?} -> {encoded:?}, expected {loose_prefix:?}"
            );

            let Some((index, _)) = prefix.char_indices().next_back() else {
                break;
            };
            prefix = &prefix[..index];
        }
    }

    fn assert_safe_round_trip(input: impl SafePath + Eq + std::fmt::Debug) {
        let encoded = input.to_safe_path();
        assert_safe_path(&encoded);
        assert_eq!(SafePath::from_safe_path(&encoded), Some(input));
    }

    fn assert_safe_path(path: &Path) {
        fn is_known_windows_reserved_name(component: &str) -> bool {
            let stem = component.split('.').next().unwrap().to_uppercase();
            RESERVED_NAMES_WINDOWS.contains(&stem.as_str())
        }

        fn is_forbidden_in_windows_component(char: char) -> bool {
            char <= '\u{1f}' || matches!(char, '<' | '>' | ':' | '"' | '\\' | '|' | '?' | '*')
        }

        for component in path.to_str().unwrap().split('/') {
            assert!(!component.is_empty(), "empty component in {path:?}");
            assert!(
                !matches!(component.chars().last(), Some('.' | ' ')),
                "component ends in a dot or space: {component:?} in {path:?}"
            );
            assert!(
                component != "." && component != "..",
                "relative component {component:?} in {path:?}"
            );
            assert!(
                !is_known_windows_reserved_name(component),
                "reserved Windows component {component:?} in {path:?}"
            );
            assert!(
                !component.chars().any(is_forbidden_in_windows_component),
                "forbidden character in {component:?} in {path:?}"
            );
        }
    }

    // Other helpers

    fn simple_reserved_names_windows() -> Vec<&'static str> {
        RESERVED_NAMES_WINDOWS
            .iter()
            .filter(|name| name.chars().all(|char| char.is_ascii_alphanumeric()))
            .copied()
            .collect()
    }
}
