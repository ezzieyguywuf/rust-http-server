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

  let mut serve: bool = true;
  for stream in listener.incoming() {
    let stream = stream.unwrap();

    serve = handle_connection(stream, &server_name, serve);
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

fn handle_connection(mut stream: TcpStream, server_name: &str, serve: bool) -> bool {
  let buf_reader = BufReader::new(&mut stream);
  let http_request: Vec<_> = buf_reader
    .lines()
    .map(|result| result.unwrap_or_else(|error| format!("Error parsing result: {:?}", error)))
    .take_while(|line| !line.is_empty())
    .collect();

  let (HttpResponse { status, content }, should_serve) =
    generate_response(&http_request, server_name, serve);
  let length = content.len();

  let response = format!("{status}\r\nContent-Length: {length}\r\n\r\n{content}");

  if should_serve {
    stream
      .write_all(response.as_bytes())
      .unwrap_or_else(|error| {
        println!("Error writing response: {:?}", error);
      });
  }

  if !http_request.is_empty() && !http_request.iter().any(|line| is_google_health_check(line)) {
    println!("-- BEGIN SERVER MESSAGE --");
    println!("Request: {:#?}", http_request);
    if !should_serve {
      println!("NOT SENDING RESPONSE: call /on to turn server back on");
    }
    println!("-- END SERVER MESSAGE --\n");
  }

  should_serve
}

fn is_google_health_check(data: &str) -> bool {
  let lower = data.to_lowercase();
  lower.starts_with("user-agent:") && (lower.contains("googlehc") || lower.contains("uptimechecks"))
}

fn generate_response(
  http_request: &Vec<String>,
  server_name: &str,
  serve: bool,
) -> (HttpResponse, bool) {
  match generate_content(http_request, server_name, serve) {
    (Ok(content), should_serve) => (
      HttpResponse {
        status: String::from("HTTP/1.1 200 OK"),
        content,
      },
      should_serve,
    ),
    (Err(error), should_serve) => (
      HttpResponse {
        status: String::from("HTTP/1.1 500 Error"),
        content: error.msg,
      },
      should_serve,
    ),
  }
}

fn generate_content(
  http_request: &Vec<String>,
  server_name: &str,
  serve: bool,
) -> (Result<String, Error>, bool) {
  if http_request.is_empty() {
    (
      Err(Error {
        msg: String::from("Empty request, don't know what to do\n"),
      }),
      serve,
    )
  } else {
    let start_line = &http_request[0];
    match parse_start_line(start_line) {
      Ok(start_line) => {
        if start_line.target == "/" {
          (
            Ok(format!(
              "Hi there! I'm a server. My name is: {}\n",
              server_name
            )),
            serve,
          )
        } else if start_line.target == "/on" {
          (
            Ok(format!(
              "Server spinning back up, beep-boop. Hi! My name is: {}\n",
              server_name
            )),
            true,
          )
        } else if start_line.target == "/off" {
          (
            Ok(String::from(
              "Server shutting down, BEEEeeep... spin back up with /on\n",
            )),
            false,
          )
        } else {
          (
            Err(Error {
              msg: format!("invalid target: {}\n", start_line.target),
            }),
            serve,
          )
        }
      }
      Err(error) => (Err(error), serve),
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
