use argh::FromArgs;

#[derive(Debug, FromArgs)]
/// Hosts a file.
pub struct Args {
    /// the file to host.
    #[argh(positional)]
    pub file: String,
}
