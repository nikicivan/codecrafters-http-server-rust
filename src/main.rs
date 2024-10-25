use std::{
    io::{BufWriter, Write},
    net::TcpListener,
};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut writer = BufWriter::new(stream);
                /*
                 * Response format is
                 * ```
                 * HTTP-Version Status-Code Reason-Phrase CRLF
                 * headers CRLF
                 * message-body
                 * ```
                 * as defined in rfc9112 2.1 message format
                 */
                let message = "HTTP/1.1 200 OK\r\n\r\n";
                match writer.write_all(message.as_bytes()) {
                    Ok(_) => writer.flush().unwrap_or_else(|writer_flush_error| {
                        println!("Failed to flush writer stream: {}", writer_flush_error);
                    }),
                    Err(stream_write_error) => {
                        println!("Failed to write to stream: {}", stream_write_error);
                    }
                }
            }
            Err(stream_error) => {
                println!("stream error: {}", stream_error);
            }
        }
    }
}
