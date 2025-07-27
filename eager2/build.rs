use std::{
    borrow::Cow,
    collections::VecDeque,
    env,
    error::Error as StdError,
    fmt::{self, Debug, Formatter, Write},
    ops::{Deref, DerefMut},
    process::{Command, ExitCode, Termination},
    rc::Rc,
};

#[repr(transparent)]
struct Error(Rc<dyn StdError>);

impl<T: StdError + 'static> From<T> for Error {
    fn from(value: T) -> Self {
        Self(Rc::new(value))
    }
}

impl Deref for Error {
    type Target = Rc<dyn StdError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Error {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_char('\n')?;
        let mut errors = VecDeque::from([&*self.0]);
        while let Some(src) = self.0.source() {
            errors.push_front(src);
        }
        for (i, err) in errors.iter().enumerate() {
            writeln!(f, "\t#{i}: {err}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ErrString(Cow<'static, str>);

impl StdError for ErrString {}

impl fmt::Display for ErrString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.to_string().as_str())
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum RustcChannel {
    Stable,
    Beta,
    Nightly,
}

impl fmt::Display for RustcChannel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
            Self::Nightly => "nightly",
        })
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct RustcVersion {
    major: u32,
    minor: u32,
    patch: u32,
    channel: RustcChannel,
}

impl TryFrom<(&str, &str, &str, &str)> for RustcVersion {
    type Error = Error;
    fn try_from(value: (&str, &str, &str, &str)) -> Result<Self, Self::Error> {
        Ok(Self {
            major: value.0.parse()?,
            minor: value.1.parse()?,
            patch: value.2.parse()?,
            channel: match value.3.to_lowercase().as_str() {
                "stable" => Ok(RustcChannel::Stable),
                "beta" => Ok(RustcChannel::Beta),
                "nightly" => Ok(RustcChannel::Nightly),
                s => Err(ErrString(format!("unknown rust channel: {s}").into())),
            }?,
        })
    }
}

impl fmt::Display for RustcVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "rustc {}.{}.{}-{}",
            self.major, self.minor, self.patch, self.channel
        )
    }
}

fn main() -> Result<impl Termination, Error> {
    let rustc = env::var("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output()?;
    if !output.status.success() {
        return Ok(ExitCode::from(
            output
                .status
                .code()
                .expect("could not retrieve status code") as u8,
        ));
    }
    let stdout = output
        .stdout
        .trim_ascii_start()
        .strip_prefix(b"rustc ")
        .expect("rustc version output does not start with \"rustc\"");

    let next_sep = stdout
        .iter()
        .position(|byte| *byte == b'.')
        .expect("could not retrieve rustc major version");
    let maj = &stdout[0..next_sep];
    let stdout = unsafe {
        stdout
            .strip_prefix(maj)
            .unwrap_unchecked()
            .strip_prefix(b".")
            .unwrap_unchecked()
    };

    let next_sep = stdout
        .iter()
        .position(|byte| *byte == b'.')
        .expect("could not retrieve rustc minor version");
    let min = &stdout[0..next_sep];
    let stdout = unsafe {
        stdout
            .strip_prefix(min)
            .unwrap_unchecked()
            .strip_prefix(b".")
            .unwrap_unchecked()
    };

    let mut have_channel_info = false;
    let next_sep = stdout
        .iter()
        .position(|byte| {
            if *byte == b'-' {
                have_channel_info = true;
            }
            byte.is_ascii_whitespace() || *byte == b'-'
        })
        .expect("could not retrieve rustc major version");
    let patch = &stdout[0..next_sep];
    let stdout = unsafe {
        stdout
            .strip_prefix(patch)
            .unwrap_unchecked()
            .strip_prefix(if have_channel_info {
                b"-".as_slice()
            } else {
                b""
            })
            .unwrap_unchecked()
            .trim_ascii_start()
    };

    let channel = if have_channel_info {
        stdout
            .iter()
            .take_while(|byte| !byte.is_ascii_whitespace())
            .copied()
            .collect()
    } else {
        b"stable".to_vec()
    };

    let rustversion: RustcVersion = (
        str::from_utf8(maj)?,
        str::from_utf8(min)?,
        str::from_utf8(patch)?,
        str::from_utf8(&channel)?,
    )
        .try_into()?;

    println!("cargo::rustc-check-cfg=cfg(rustchan, values(\"stable\", \"beta\", \"nightly\"))");
    println!("cargo::rustc-cfg=rustchan=\"{}\"", rustversion.channel);

    Ok(ExitCode::SUCCESS)
}
