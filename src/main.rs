use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

#[derive(Debug)]
enum HttpRequestMethod {
    Get,
    Post,
}

impl HttpRequestMethod {
    fn from_string(input: &str) -> Option<HttpRequestMethod> {
        match input {
            "GET" => Some(HttpRequestMethod::Get),
            "POST" => Some(HttpRequestMethod::Post),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct HttpRequest<'a> {
    method: HttpRequestMethod,
    version: &'a str,
    path: &'a str,
}

impl<'a> HttpRequest<'a> {
    pub fn new(request_line: &'a str) -> Self {
        let split_line: Vec<_> = request_line.split_whitespace().collect();
        println!("New HTTP Request {:?}", split_line.clone());
        return Self {
            method: HttpRequestMethod::from_string(split_line[0])
                .expect("Unable to parse http method"),
            version: split_line[2],
            path: split_line[1],
        };
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(stream_error) => {
                println!("stream error: {}", stream_error);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);

    let request_line: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // let response = match request_line.as_str() {
    //     "GET / HTTP/1.1" => "HTTP/1.1 200 OK\r\n\r\n",
    //     _ => "HTTP/1.1 404 Not Found\r\n\r\n",
    // };

    println!("Request line {:#?}", request_line.clone());

    let http_request = HttpRequest::new(&request_line[0]);
    let response: String;
    let mut path_segments = http_request.path.split("/");
    if let Some(endpoint) = path_segments.nth(1) {
        println!("Response: {:#?}", endpoint);
        response = match endpoint {
            "" => String::from("HTTP/1.1 200 OK\r\n\r\n"),
            "echo" => {
                if let Some(param) = path_segments.nth(0) {
                    let length = param.len();
                    format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length:{length}\r\n\r\n{param}")
                } else {
                    String::from("HTTP/1.1 200 OK\r\n\r\n")
                }
            }
            _ => String::from("HTTP/1.1 404 Not Found\r\n\r\n"),
        };
    } else {
        response = String::from("HTTP/1.1 404 Not Found\r\n\r\n");
    }

    let _ = stream.write_all(response.as_bytes()).unwrap();
}
