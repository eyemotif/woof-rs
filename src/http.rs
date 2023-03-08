use std::collections::HashMap;

pub struct Server {
    files: HashMap<String, std::fs::File>,
    args: crate::cli::Args,
}

impl Server {
    pub fn new(args: crate::cli::Args) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let raw_files = args
            .files
            .iter()
            .map(|path| std::path::Path::new(path))
            .filter_map(|p| Some((p.file_name()?, p)))
            .filter_map(|(name, p)| match p.try_exists() {
                Ok(true) => Some(Ok((name, p))),
                Ok(false) => None,
                Err(err) => Some(Err(err)),
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|(name, p)| {
                std::fs::File::open(p).map(|f| (name.to_string_lossy().into_owned(), f))
            })
            .collect::<Result<Vec<_>, _>>()?;

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
                    request.respond(
                        tiny_http::Response::from_string(index_with_links).with_header(
                            tiny_http::Header::from_bytes("Content-Type", "text/html").unwrap(),
                        ),
                    )?;
                }
                filepath => {
                    if let Some(file) = self
                        .files
                        .get(&filepath.chars().skip(1).collect::<String>())
                    {
                        match file.try_clone() {
                            Ok(file) => request.respond(tiny_http::Response::from_file(file))?,
                            Err(err) => request.respond(
                                tiny_http::Response::from_string(format!(
                                    "Error opening file: {err}"
                                ))
                                .with_status_code(500),
                            )?,
                        }
                    } else {
                        request.respond(
                            tiny_http::Response::from_string("File not found.")
                                .with_status_code(404),
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
            .map(|k| format!("<a class=\"dl\" href=\"/{}\" download></a>", k))
            .collect()
    }
}

fn print_request(args: &crate::cli::Args, request: &tiny_http::Request) {
    if let Some(addr) = request.remote_addr() {
        args.log(format!("{addr}>> {}", request.url()));
    } else {
        args.log(format!(">> {}", request.url()));
    }
}
