use getopts::Options;
use std::{
  env,
  io::{prelude::*, BufReader},
  net::{TcpListener, TcpStream},
};

fn main() {
  let ServerOptions {
    server_name,
    address,
    port,
  } = match parse_options() {
    Ok(opts) => opts,
    Err(error) => {
      println!("Error parsing flags: {:?}", error.msg);
      return;
    }
  };

  println!("Server name: {server_name}");
  let listener = match TcpListener::bind(format!("{address}:{port}")) {
    Ok(listener) => listener,
    Err(error) => {
      println!("Error connecting: {:?}", error);
      return;
    }
  };

  for stream in listener.incoming() {
    let stream = stream.unwrap();

    handle_connection(stream, &server_name);
  }
}

struct ServerOptions {
  server_name: String,
  address: String,
  port: String,
}

fn parse_options() -> Result<ServerOptions, Error> {
  let args: Vec<String> = env::args().collect();
  let mut opts = Options::new();
  opts.reqopt("n", "name", "The server's name", "SERVER_NAME");
  opts.reqopt("p", "port", "The port on which to listen", "PORT");
  opts.optopt(
    "a",
    "address",
    "The address to liste to (default 127.0.0.1)",
    "ADDRESS",
  );

  let matches = match opts.parse(&args[1..]) {
    Ok(m) => m,
    Err(f) => return Err(Error { msg: f.to_string() }),
  };
  let address = match matches.opt_str("a") {
    Some(val) => val,
    None => String::from("127.0.0.1"),
  };
  let server_name = match matches.opt_str("n") {
    Some(val) => val,
    None => {
      return Err(Error {
        msg: String::from("Error parsing SERVER_NAME flag"),
      })
    }
  };
  let port = match matches.opt_str("p") {
    Some(val) => val,
    None => {
      return Err(Error {
        msg: String::from("Error parsing PORT flag"),
      })
    }
  };

  Ok(ServerOptions {
    server_name,
    address,
    port,
  })
}

fn handle_connection(mut stream: TcpStream, server_name: &str) {
  let buf_reader = BufReader::new(&mut stream);
  let http_request: Vec<_> = buf_reader
    .lines()
    .map(|result| result.unwrap_or_else(|error| format!("Error parsing result: {:?}", error)))
    .take_while(|line| !line.is_empty())
    .collect();

  if !http_request
    .iter()
    .any(|line| line.contains("User-Agent: GoogleHC"))
  {
    println!("Request: {:#?}", http_request);
  }

  let HttpResponse { status, content } = generate_response(&http_request, server_name);
  let length = content.len();

  let response = format!("{status}\r\nContent-Length: {length}\r\n\r\n{content}");

  stream
    .write_all(response.as_bytes())
    .unwrap_or_else(|error| {
      println!("Error writing response: {:?}", error);
    });
}

fn generate_response(http_request: &Vec<String>, server_name: &str) -> HttpResponse {
  match generate_content(http_request, server_name) {
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

fn generate_content(http_request: &Vec<String>, server_name: &str) -> Result<String, Error> {
  if http_request.is_empty() {
    Err(Error {
      msg: String::from("Empty request, don't know what to do\n"),
    })
  } else {
    let start_line = &http_request[0];
    match parse_start_line(start_line) {
      Ok(start_line) => {
        if start_line.target == "/" {
          Ok(format!(
            "Hi there! I'm a server. My name is: {}\n",
            server_name
          ))
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
