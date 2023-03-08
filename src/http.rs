use std::collections::HashMap;

pub struct Server {
    files: HashMap<String, std::fs::File>,
}

impl Server {
    pub fn new(args: crate::cli::Args) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let path = std::path::Path::new(&args.file);

        if !path.try_exists()? {
            return Err(format!("File {:?} not found.", args.file).into());
        }

        Ok(Self {
            files: HashMap::from_iter([(args.file.clone(), std::fs::File::open(path)?)]),
        })
    }

    pub fn host(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let server = tiny_http::Server::http("0.0.0.0:0")?;
        let index = include_str!("download.html");

        println!("Server up at {}", server.server_addr());

        for request in server.incoming_requests() {
            print_request(&request);

            match request.url() {
                "/" => {
                    let index_with_links =
                        index.replace("{}", &self.get_download_links().join("\n"));
                    request.respond(
                        tiny_http::Response::from_string(index_with_links).with_header(
                            tiny_http::Header::from_bytes("Content-Type", "text/html").unwrap(),
                        ),
                    )?;
                }
                "/favicon.ico" => request.respond(tiny_http::Response::from_data([]))?,
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

fn print_request(request: &tiny_http::Request) {
    if let Some(addr) = request.remote_addr() {
        println!("{addr}>> {}", request.url())
    } else {
        println!(">> {}", request.url())
    }
}