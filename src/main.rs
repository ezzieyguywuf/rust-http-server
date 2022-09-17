use std::{
  env,
  io::{prelude::*, BufReader},
  net::{TcpListener, TcpStream},
};

fn main() {
  for arg in env::args() {
    println!("got arg: {arg}");
  }

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

  let HttpResponse { status, content } = generate_response(&http_request);
  let length = content.len();

  let response = format!("{status}\r\nContent-Length: {length}\r\n\r\n{content}");

  stream
    .write_all(response.as_bytes())
    .unwrap_or_else(|error| {
      println!("Error writing response: {:?}", error);
    });
}

fn generate_response(http_request: &Vec<String>) -> HttpResponse {
  match generate_content(http_request) {
    Ok(content) => HttpResponse {
      status: String::from("HTTP/1.1 200 OK"),
      content,
    },
    Err(error) => HttpResponse {
      status: String::from("HTTP/1.1 500 Error"),
      content: format!("{:?}", error.msg),
    },
  }
}

fn generate_content(http_request: &Vec<String>) -> Result<String, Error> {
  if http_request.is_empty() {
    Err(Error {
      msg: String::from("Empty request, don't know what to do\n"),
    })
  } else {
    let start_line = &http_request[0];
    match parse_start_line(start_line) {
      Ok(start_line) => {
        if start_line.target == "/" {
          Ok(String::from("Hi there! I'm server X\n"))
        } else {
          Err(Error {
            msg: format!("invalid target: {}\n", start_line.target),
          })
        }
      }
      Err(error) => Err(error),
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

struct HttpResponse {
  status: String,
  content: String,
}

struct HttpStartLine {
  _action: String,
  target: String,
  _version: String,
}
