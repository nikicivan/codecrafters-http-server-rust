use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

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

    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let response = match request_line.as_str() {
        "GET / HTTP/1.1" => "HTTP/1.1 200 OK\r\n\r\n",
        _ => "HTTP/1.1 404 Not Found\r\n\r\n",
    };

    stream.write_all(response.as_bytes()).unwrap();
}
