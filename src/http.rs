use crate::cli::FileOption;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tiny_http::Response;

pub struct Server<F = DefaultMode> {
    files: F,
    args: crate::cli::Args,
}

pub struct DefaultMode(HashMap<String, std::fs::File>);
pub struct UploadMode {
    filter: HashSet<String>,
    left: usize,
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

        Ok(Self {
            files: DefaultMode(files),
            args,
        })
    }

    pub fn new_upload(args: crate::cli::Args) -> Server<UploadMode> {
        Server {
            files: UploadMode {
                filter: HashSet::from_iter(args.paths.clone()),
                left: args.count,
            },
            args,
        }
    }

    pub fn host(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let server = tiny_http::Server::http(format!("{}:{}", self.args.address, self.args.port))?;
        let index = include_str!("download.html");

        self.args
            .log(format!("Server up at {}", self.args.pretty_address()));

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
                            .0
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
                    Response::from_string(format!("{:?}", self.files.0.keys().collect::<Vec<_>>())),
                )?,
                filepath => {
                    if let Some(file) = self
                        .files
                        .0
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
            .0
            .keys()
            .map(|k| format!("<a href=\"/{}\" download></a>", k))
            .collect()
    }
}

impl Server<UploadMode> {
    pub fn receive(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let server = tiny_http::Server::http(format!("{}:{}", self.args.address, self.args.port))?;
        let index = include_str!("upload.html");

        self.args
            .log(format!("Server up at {}", self.args.pretty_address()));

        for mut request in server.incoming_requests() {
            print_request(&self.args, &request);

            match request.url() {
                "/" => request.respond(Response::from_string(index).with_header(
                    tiny_http::Header::from_bytes("Content-Type", "text/html").unwrap(),
                ))?,
                "/upload" if matches!(request.method(), &tiny_http::Method::Put) => {
                    let mut buf = Vec::new();
                    request.as_reader().read_to_end(&mut buf)?;

                    match request
                        .headers()
                        .iter()
                        .find(|header| header.field.equiv("File-Name"))
                    {
                        Some(file_name_header) => {
                            if !self.files.filter.is_empty()
                                && !self.files.filter.contains(file_name_header.value.as_str())
                            {
                                request.respond(Response::empty(200))?;
                                continue;
                            }

                            match std::fs::write(file_name_header.value.as_str(), buf) {
                                Ok(_) => request.respond(Response::empty(201))?,
                                Err(err) => {
                                    request.respond(Response::empty(500))?;
                                    return Err(err.into());
                                }
                            }
                        }
                        None => request.respond(
                            Response::from_string("Could not find File-Name header")
                                .with_status_code(400),
                        )?,
                    }
                }
                "/upload" => request.respond(
                    Response::from_string("Redirecting you back...")
                        .with_status_code(308)
                        .with_header(tiny_http::Header::from_bytes("Location", "/").unwrap()),
                )?,
                _ => request
                    .respond(Response::from_string("File not found.").with_status_code(404))?,
            }
        }

        self.args
            .log(format!("Server up at {}", self.args.pretty_address()));
        Ok(())
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
        args.log(format!("{addr}>> {} {}", request.method(), request.url()));
    } else {
        args.log(format!(">> {} {}", request.method(), request.url()));
    }
}
