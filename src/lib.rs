use flate2::{write::GzEncoder, Compression};
use std::{
    collections::HashMap,
    fs,
    io::Write,
    sync::{mpsc, Arc, Mutex},
    thread,
};

#[derive(Debug)]
pub struct Response {
    status_code: String,
    status_message: String,
    headers: String,
    body: String,
}

impl Response {
    pub fn new(
        status_code: String,
        status_message: String,
        headers: String,
        body: String,
    ) -> Response {
        let headers = if !headers.is_empty() && !headers.ends_with("\r\n") {
            format!("{headers}\r\n")
        } else {
            headers
        };
        Response {
            status_code,
            status_message,
            body,
            headers,
        }
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.push_str(&format!("{key}: {value}\r\n"));
    }

    pub fn set_body(&mut self, body: String) {
        self.body = String::from(body);
    }

    pub fn create_http_response(&self) -> Result<Vec<u8>, ()> {
        if self.status_code.is_empty() || self.status_message.is_empty() {
            ()
        }
        let response = format!(
            "HTTP/1.1 {} {}\r\n{}\r\n{}",
            self.status_code, self.status_message, self.headers, self.body
        );
        let result = response.into_bytes();

        Ok(result)
    }

    pub fn not_found() -> Response {
        Response::new(
            String::from("404"),
            String::from("Not Found"),
            String::from("Content-Length: 0"),
            String::new(),
        )
    }

    pub fn create_response_with_body(body: &str) -> Response {
        let mut response = Response::new(
            String::from("200"),
            String::from("OK"),
            String::from(""),
            String::from(body),
        );
        response.add_header("Content-Type", "text/plain");
        response.add_header("Content-Length", &body.len().to_string());
        response
    }

    pub fn create_gzip_response(body: &str) -> Vec<u8> {
        let mut response = Response::new(
            String::from("200"),
            String::from("OK"),
            String::new(),
            String::from(body),
        );

        response.add_header("Content-Encoding", "gzip");
        let mut gz = GzEncoder::new(vec![], Compression::default());
        _ = gz.write_all(&body.as_bytes()).unwrap();
        let comp_body = gz.finish().unwrap();
        response.add_header("Content-Length", &comp_body.len().to_string());

        let mut response = format!(
            "HTTP/1.1 {} {}\r\n{}\r\n",
            response.status_code, response.status_message, response.headers
        )
        .into_bytes();

        response.extend_from_slice(&comp_body);
        response
    }

    pub fn create_file(file_path: &str, content: &str) -> Response {
        let _ = fs::write(file_path, content);
        Response::new(
            String::from("201"),
            String::from("Created"),
            String::from("Content-Length: 0"),
            String::new(),
        )
    }

    pub fn create_file_response(path: &str) -> Response {
        let content = fs::read_to_string(&path);
        match content {
            Ok(c) => {
                let size = fs::metadata(path).unwrap().len();
                let mut response =
                    Response::new(String::from("200"), String::from("OK"), String::new(), c);
                response.add_header("Content-Type", "application/octet-stream");
                response.add_header("Content-Length", &size.to_string());
                response
            }
            Err(_) => Response::not_found(),
        }
    }
}

static INITIAL_WORKER_AMOUNT: usize = 4;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("Worker {id} got a job. Executing...");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected. Shuting down...");
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new() -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(INITIAL_WORKER_AMOUNT);
        for i in 0..INITIAL_WORKER_AMOUNT {
            let worker = Worker::new(i, Arc::clone(&receiver));
            workers.push(worker);
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}
