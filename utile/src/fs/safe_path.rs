use std::path::{Component, Path, PathBuf};

use url::Url;

const RESERVED_NAMES_WINDOWS: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];
const RESERVED_NAMES_UNIX: [&str; 2] = [".", ".."];

/// First as it's used by the others.
const PERCENT: (char, &str) = ('%', "%25");

// https://stackoverflow.com/questions/1976007/what-characters-are-forbidden-in-windows-and-linux-directory-names
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

pub trait SafePath: Sized {
    fn to_safe_path(&self) -> PathBuf;
    fn from_safe_path(path: &Path) -> Option<Self>;
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
        // Extract scheme and prepare the rest of the URL
        let scheme = self.scheme();
        let string = self.to_string();
        let rest = string
            .strip_prefix(scheme)
            .unwrap()
            .strip_prefix(':')
            .unwrap();
        if let Some(rest) = rest.strip_prefix("//") {
            to_safe_path(&format!("{scheme}/{rest}"))
        } else {
            // Technically possible, just handling it for completeness.
            // panic!("{}", &format!("{scheme}%/{rest}"));
            to_safe_path(&format!("{scheme}!/{rest}"))
        }
    }

    fn from_safe_path(path: &Path) -> Option<Self> {
        let scheme = path.components().next()?.as_os_str().to_str()?;
        let rest: PathBuf = path.components().skip(1).collect();
        let rest = from_safe_path(&rest)?;
        Self::parse(&if let Some(scheme) = scheme.strip_suffix('!') {
            format!("{scheme}:{rest}")
        } else {
            format!("{scheme}://{rest}")
        })
        .ok()
    }
}

/// Converts any string to a file-system safe path string that can be used across
/// major operating systems (Windows, macOS, Linux).
fn to_safe_path(input: &str) -> PathBuf {
    let mut result = input.replace(PERCENT.0, PERCENT.1);

    for (char, encoded) in ESCAPED_CHARS {
        result = result.replace(char, encoded);
    }

    let fragments: Vec<String> = result
        .split('/')
        .map(|s| {
            if RESERVED_NAMES_WINDOWS.contains(&s.to_uppercase().as_str())
                || RESERVED_NAMES_UNIX.contains(&s.to_uppercase().as_str())
            {
                format!("%{s}")
            } else if s.ends_with('.') || s.ends_with(' ') {
                format!("{s}%END")
            } else if s.is_empty() {
                "%END".to_string()
            } else {
                s.to_string()
            }
        })
        .collect();

    fragments.join("/").into()
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
        .try_collect()?;

    let mut path = path.join("/");

    for (char, encoded) in ESCAPED_CHARS {
        path = path.replace(encoded, &char.to_string());
    }

    let result = path.replace(PERCENT.1, &PERCENT.0.to_string());

    Some(result)
}
fn decode_component(c: &str) -> String {
    let mut c = c.replace("%END", "");

    if let Some(s) = c.strip_prefix('%') {
        if RESERVED_NAMES_WINDOWS.contains(&s.to_uppercase().as_str())
            || RESERVED_NAMES_UNIX.contains(&s.to_uppercase().as_str())
        {
            c = s.to_owned();
        }
    }

    c
}

#[cfg(test)]
mod tests {
    use rand::{rngs::SmallRng, seq::IndexedRandom, Rng, SeedableRng};

    use super::*;

    const TEST_CASES: [(&str, &str); 5] = [
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
        for (input, expected) in TEST_CASES {
            let encoded = to_safe_path(input);
            assert_eq!(encoded.to_str().unwrap(), expected);

            let round_trip = from_safe_path(&encoded).unwrap();
            assert_eq!(input, round_trip);
        }
    }

    #[test]
    fn test_fuzz_random_strings() {
        let mut rng = SmallRng::seed_from_u64(42);

        for _ in 0..100_000 {
            // Generate random string
            let random_string = random_string(&mut rng);

            let encoded = to_safe_path(&random_string);
            let decoded = from_safe_path(&encoded).unwrap();

            assert_eq!(decoded, random_string);
        }
    }
    fn random_string(rng: &mut impl Rng) -> String {
        let len = rng.random_range(0..5);
        (0..len)
            .map(|_| {
                (0..rng.random_range(0..10))
                    .map(|_| random_char(rng))
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("/")
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
            "https/example.com/%CON/%PRN/%aux/nul.txt",
        ),
        (
            "https:example",
            Some("https://example/"),
            "https/example/%END",
        ),
        ("data:text/plain,Stuff", None, "data!/text/plain,Stuff"),
        ("unix:/run/foo.socket", None, "unix!/%END/run/foo.socket"),
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
        ("file:///path/to/file", None, "file/%END/path/to/file"),
        (
            "file://localhost/path/to/file",
            Some("file:///path/to/file"),
            "file/%END/path/to/file",
        ),
        (
            "file:relative/path",
            Some("file:///relative/path"),
            "file/%END/relative/path",
        ),
        // Mailto URLs
        ("mailto:user@example.com", None, "mailto!/user@example.com"),
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

            let encoded = input.to_safe_path();
            assert_eq!(
                encoded.to_str().unwrap(),
                expected,
                "\n{raw} \n&  {input} \n-> {} \n!= {expected}",
                encoded.to_str().unwrap()
            );

            let round_trip = Url::from_safe_path(&encoded).unwrap();
            assert_eq!(input, round_trip);
        }
    }

    #[test]
    fn test_conversions() {
        for (char, encoded) in ESCAPED_CHARS {
            assert_roundtrip(char);
            assert_eq!(format!("%{:02x}", char as u8), encoded);
        }
    }
    const fn assert_roundtrip(char: char) {
        if char != (char as u8) as char {
            panic!("Invalid character");
        }
    }
}
