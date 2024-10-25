use codecrafters_http_server::{Response, ThreadPool};
use std::io::Write;
use std::net::TcpListener;
use std::{
    collections::HashMap,
    env,
    io::{BufRead, BufReader, BufWriter},
    net::TcpStream,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    println!("Server started listening at 127.0.0.1:4221");

    let thread_pool = ThreadPool::new();

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                println!("accepting stream connection");
                thread_pool.execute(|| handle_connection(s));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);

    let request: Vec<String> = buf_reader
        .lines()
        .map(|line| line.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let request_line = &request[0];
    let mut request_headers = HashMap::new();

    for header in &request[1..] {
        let (key, value) = header.split_once(":").unwrap();
        request_headers.insert(key, value.trim());
    }

    let path: Vec<&str> = request_line.split(" ").collect();
    let path = path[1];

    let response = if path == "/" {
        Response::new(
            String::from("200"),
            String::from("OK"),
            String::from("Content-Length: 0"),
            String::new(),
        )
    } else if path.starts_with("/echo/") {
        Response::create_response_with_body(&path[6..])
    } else if path == "/user-agent" {
        let user_agent = request_headers.get("User-Agent").unwrap();
        Response::create_response_with_body(&user_agent)
    } else if path.starts_with("/files/") {
        let mut file_dir = env::args()
            .skip_while(|arg| arg != "--directory")
            .nth(1)
            .expect("No Directory Argument provided!");
        if !file_dir.ends_with("/") {
            file_dir.push_str("/");
        }
        let path = format!("{file_dir}{}", &path[7..]);
        Response::create_file_response(&path)
    } else {
        Response::not_found()
    };

    stream
        .write_all(&response.create_http_response().unwrap())
        .unwrap();
}
