use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = match TcpListener::bind("127.0.0.1:7878") {
        Ok(listener) => listener,
        Err(error) => {
            println!("Error connecting: {:?}", error);
            return;
        }
    };

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap_or_else(|error| format!("Error parsing result: {:?}", error)))
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request);

    let status_line = "HTTP/1.1 200 OK";
    let contents = generate_content(&http_request);
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream
        .write_all(response.as_bytes())
        .unwrap_or_else(|error| {
            println!("Error writing response: {:?}", error);
        });
}

fn generate_content(http_request: &Vec<String>) -> String {
    if http_request.is_empty() {
        String::from("Empty request, don't know what to do\n")
    } else {
        let start_line = &http_request[0];
        match parse_start_line(start_line) {
            Ok(start_line) => {
                if start_line.target == "/" {
                    match fs::read_to_string("html/hello.html") {
                        Ok(contents) => contents,
                        Err(error) => error.to_string() + "\n",
                    }
                } else {
                    format!("invalid target: {}\n", start_line.target)
                }
            }
            Err(error) => error.msg,
        }
    }
}

fn parse_start_line(line: &str) -> Result<HttpStartLine, Error> {
    let split_line: Vec<_> = line.split_whitespace().collect();
    if split_line.len() != 3 {
        Err(Error {
            msg: format!("Invalid HTTP start line: {:?}", line),
        })
    } else if let [action, target, version] = &split_line[..] {
        Ok(HttpStartLine {
            _action: action.to_string(),
            target: target.to_string(),
            _version: version.to_string(),
        })
    } else {
        Err(Error {
            msg: format!("Internal error. Start string: {:?}", line),
        })
    }
}

#[derive(Debug)]
struct Error {
    msg: String,
}

struct HttpStartLine {
    _action: String,
    target: String,
    _version: String,
}
