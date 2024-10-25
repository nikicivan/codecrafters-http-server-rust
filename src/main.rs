use codecrafters_http_server::{Response, ThreadPool};
use std::io::{Read, Write};
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
    let mut buf_reader = BufReader::new(&stream);

    let request: Vec<String> = buf_reader
        .by_ref()
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
    let http_method = path[0];
    let path = path[1];

    let content_length = if let Some(length) = request_headers.get("Content-Length") {
        length.parse().unwrap_or(0)
    } else {
        0
    };

    let request_body = if http_method == "POST" && content_length > 0 {
        let mut body = vec![0u8; content_length];
        buf_reader.read_exact(&mut body).unwrap();
        String::from_utf8_lossy(&body).to_string()
    } else {
        String::new()
    };

    let response = if path == "/" {
        Response::new(
            String::from("200"),
            String::from("OK"),
            String::from("Content-Length: 0"),
            String::new(),
        )
    } else if path.starts_with("/echo/") {
        Response::create_response_with_body(&path[6..], &request_headers)
    } else if path == "/user-agent" {
        let user_agent = request_headers.get("User-Agent").unwrap();
        Response::create_response_with_body(&user_agent, &request_headers)
    } else if path.starts_with("/files/") {
        println!("FILE {:#?}", &path[7..]);
        let file_path = get_path_arg(&path[7..]);

        match http_method {
            "GET" => Response::create_file_response(&file_path),
            "POST" => Response::create_file(&file_path, &request_body),
            _ => Response::not_found(),
        }
    } else {
        Response::not_found()
    };

    stream
        .write_all(&response.create_http_response().unwrap())
        .unwrap();
}

fn get_path_arg(file_name: &str) -> String {
    let mut file_dir = env::args()
        .skip_while(|arg| arg != "--directory")
        .nth(1)
        .expect("Directory argument not provided");

    if !file_dir.ends_with("/") {
        file_dir.push_str("/");
    }

    println!("File name {}", &file_name);

    let file_path = format!("{file_dir}{}", &file_name);
    println!("File Path {}", file_path);
    file_path
}
