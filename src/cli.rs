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

    /// disable all non-error printing.
    #[argh(switch, short = 'q')]
    pub quiet: bool,

    /// disable serving an index that automagically downloads all the files when
    /// opened by a browser.
    #[argh(switch)]
    pub no_index: bool,

    /// the files to host.
    #[argh(positional, greedy)]
    pub files: Vec<String>,
}

impl Args {
    pub fn log<S: Into<String>>(&self, message: S) {
        if !self.quiet {
            println!("{}", message.into())
        }
    }
}
