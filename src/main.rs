use std::{
    fs,
    io::{ prelude::*, BufReader },
    net::{ TcpListener, TcpStream },
    thread,
    time::Duration,
};
use web_server::ThreadPool;

fn main() {
    let address = "127.0.0.1:7878";
    let listener = TcpListener::bind(address).expect("Failed to bind to address");
    let pool = ThreadPool::new(4);

    println!("Server listening on {}", address);

    for stream in listener.incoming() {
        let stream = stream.expect("Failed to establish a connection");

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = match buf_reader.lines().next() {
        Some(Ok(line)) => line,
        Some(Err(e)) => {
            eprintln!("Failed to read request line: {}", e);
            return;
        }
        None => {
            eprintln!("Connection closed before request line was received");
            return;
        }
    };

    let (status_line, filename) = match request_line.as_str() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "pages/hello.html"),
        "GET /css/styles.css HTTP/1.1" => ("HTTP/1.1 200 OK", "css/styles.css"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "pages/hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "pages/404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap_or_else(|_| String::from("File not found"));
    let length = contents.len();
    let content_type = match filename {
        f if f.ends_with(".html") => "text/html",
        f if f.ends_with(".css") => "text/css",
        _ => "text/plain",
    };

    let response = format!(
        "{status_line}\r\nContent-Length: {length}\r\nContent-Type: {content_type}\r\n\r\n{contents}"
    );

    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("Failed to send response: {}", e);
    }
}
