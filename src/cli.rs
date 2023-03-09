use argh::FromArgs;

#[derive(Debug, FromArgs)]
/// Hosts or receives some files.
pub struct Args {
    /// the address to host on. defaults to "0.0.0.0"
    #[argh(option, default = "String::from(\"0.0.0.0\")", short = 'a')]
    pub address: String,
    /// the port to host on. defaults to "8080". set to 0 to automatically assign a port
    #[argh(option, default = "8080", short = 'p')]
    pub port: u16,

    /// disable all non-error printing
    #[argh(switch, short = 'q')]
    pub quiet: bool,

    /// in default mode, sets /file to respond with a bash or json list
    #[argh(option, default = "FileOption::Off", short = 'f')]
    pub file: FileOption,

    /// enables upload mode. [paths] becomes a filter on incoming filenames
    #[argh(switch, short = 'U')]
    pub upload: bool,

    /// in upload mode, the amount of files -U will accept before closing. 0
    /// means it will never close
    #[argh(option, default = "0", short = 'c')]
    pub count: usize,

    /// in default mode, disable serving an index that automagically downloads
    /// all the files when opened by a browser
    #[argh(switch)]
    pub no_index: bool,

    /// the files and folders to host. will host all the files if the path is a
    /// folder. follows all symlinks
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
    pub fn pretty_address(&self) -> String {
        format!(
            "http://{}:{}",
            if self.address == "0.0.0.0" {
                "localhost"
            } else {
                &self.address
            },
            self.port
        )
    }
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
