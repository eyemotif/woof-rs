use argh::FromArgs;

#[derive(Debug, FromArgs)]
/// Hosts some files.
pub struct Args {
    /// the address to host on. defaults to "0.0.0.0".
    #[argh(option, default = "String::from(\"0.0.0.0\")", short = 'a')]
    pub address: String,
    /// the port to host on. defaults to "8080". set to 0 to automatically assign a port.
    #[argh(option, default = "8080", short = 'p')]
    pub port: u16,

    /// sets /file to respond with a bash or json list.
    #[argh(option, default = "FileOption::Off", short = 'f')]
    pub file: FileOption,

    /// disable all non-error printing.
    #[argh(switch, short = 'q')]
    pub quiet: bool,

    /// disable serving an index that automagically downloads all the files when
    /// opened by a browser.
    #[argh(switch)]
    pub no_index: bool,

    /// the files and folders to host. will host all the files if the path is a
    /// folder. follows all symlinks.
    #[argh(positional, greedy)]
    pub paths: Vec<String>,
}

#[derive(Debug)]
pub enum FileOption {
    Off,
    Bash,
    Json,
}

impl Args {
    pub fn log<S: Into<String>>(&self, message: S) {
        if !self.quiet {
            println!("{}", message.into())
        }
    }
}

impl std::str::FromStr for FileOption {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" => Ok(Self::Off),
            "bash" => Ok(Self::Bash),
            "json" => Ok(Self::Json),
            _ => Err(format!("Invalid /file option {:?}", s)),
        }
    }
}
