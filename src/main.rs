use std::{
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
        .map(|result| match result {
            Ok(result) => result,
            Err(error) => format!("Error parsing result: {:?}", error),
        })
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {:#?}", http_request);

    let response = "HTTP/1.1 200 OK \r\n\r\n";

    stream
        .write_all(response.as_bytes())
        .unwrap_or_else(|error| {
            println!("Error writing response: {:?}", error);
        });
}
