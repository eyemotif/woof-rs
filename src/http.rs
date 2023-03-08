use std::collections::HashMap;
use std::path::PathBuf;

use tiny_http::Response;

use crate::cli::FileOption;

pub struct Server {
    files: HashMap<String, std::fs::File>,
    args: crate::cli::Args,
}

impl Server {
    pub fn new(args: crate::cli::Args) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let raw_files = args
            .paths
            .iter()
            .map(|p| std::path::Path::new(p))
            .filter_map(|p| match p.try_exists() {
                Ok(true) => Some(Ok(p)),
                Ok(false) => None,
                Err(err) => Some(Err(err)),
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|p| {
                std::io::Result::Ok(
                    get_paths_in(p)?
                        .into_iter()
                        .filter_map(|p| Some((p.file_name()?.to_string_lossy().into_owned(), p)))
                        .map(|(name, p)| std::fs::File::open(p).map(|file| (name, file))),
                )
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Result<Vec<_>, _>>()?;

        if raw_files.len() == 0 {
            return Err("No files given.".into());
        }

        let mut files = HashMap::new();

        for (name, file) in raw_files {
            if files.insert(name.clone(), file).is_some() {
                return Err(format!("Duplicate file name \"{name}\".").into());
            }
        }

        Ok(Self { files, args })
    }

    pub fn host(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let server = tiny_http::Server::http(format!("{}:{}", self.args.address, self.args.port))?;
        let index = include_str!("download.html");

        self.args.log(format!(
            "Server up at http://{}:{}",
            if self.args.address == "0.0.0.0" {
                "localhost"
            } else {
                &self.args.address
            },
            self.args.port
        ));

        for request in server.incoming_requests() {
            print_request(&self.args, &request);

            match request.url() {
                "/" if !self.args.no_index => {
                    let index_with_links =
                        index.replace("{}", &self.get_download_links().join("\n"));
                    request.respond(Response::from_string(index_with_links).with_header(
                        tiny_http::Header::from_bytes("Content-Type", "text/html").unwrap(),
                    ))?;
                }
                "/files" if matches!(self.args.file, FileOption::Bash) => {
                    request.respond(Response::from_string(
                        self.files
                            .keys()
                            .map(|name| {
                                if name.contains(' ') {
                                    format!("'{}'", name.replace('\'', r"'\''"))
                                } else {
                                    name.replace('\'', r"\'")
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(" "),
                    ))?
                }
                "/files" if matches!(self.args.file, FileOption::Json) => request.respond(
                    Response::from_string(format!("{:?}", self.files.keys().collect::<Vec<_>>())),
                )?,
                filepath => {
                    if let Some(file) = self
                        .files
                        .get(&filepath.chars().skip(1).collect::<String>())
                    {
                        match file.try_clone() {
                            Ok(file) => request.respond(Response::from_file(file))?,
                            Err(err) => request.respond(
                                Response::from_string(format!("Error opening file: {err}"))
                                    .with_status_code(500),
                            )?,
                        }
                    } else {
                        request.respond(
                            Response::from_string("File not found.").with_status_code(404),
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    fn get_download_links(&self) -> Vec<String> {
        self.files
            .keys()
            .map(|k| format!("<a href=\"/{}\" download></a>", k))
            .collect()
    }
}

fn get_paths_in<P: Into<PathBuf>>(outer_path: P) -> std::io::Result<Vec<PathBuf>> {
    let outer_path = outer_path.into().canonicalize()?;
    if outer_path.is_dir() {
        // is there a way to do this with only one collect?
        Ok(std::fs::read_dir(outer_path)?
            .map(|entry| entry.and_then(|entry| get_paths_in(entry.path())))
            .map(|files_result| files_result.map(|paths| paths.into_iter()))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect())
    } else {
        Ok(vec![outer_path])
    }
}

fn print_request(args: &crate::cli::Args, request: &tiny_http::Request) {
    if let Some(addr) = request.remote_addr() {
        args.log(format!("{addr}>> {}", request.url()));
    } else {
        args.log(format!(">> {}", request.url()));
    }
}
