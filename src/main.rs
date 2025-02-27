use aerator_website::ThreadPool;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::available_parallelism;

fn main() {
    let listener = TcpListener::bind(("127.0.0.1", 7878)).unwrap();
    let pool = ThreadPool::new(available_parallelism().unwrap().into());
    // println!("{}", available_parallelism().unwrap().get());

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next(); //.unwrap().unwrap();

    if request_line.is_none() {
        println!("No lines to process");
        return;
    }
    let request_line = request_line.unwrap();

    if request_line.is_err() {
        return;
    }
    let request_line = request_line.unwrap();

    // let http_request: Vec<_> = buf_reader.lines().map(|result| result.unwrap()).take_while(|line| !line.is_empty()).collect();

    // println!("Request: {http_request:#?}");

    /*let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "resources/hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "resources/404.html")
    };*/

    let (status_line, filename) = process_request(request_line);

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}

fn process_request(request_line: String) -> (String, String) {
    let request_line = request_line.trim();
    let mut request_line = request_line.split(" ");
    let request_line = request_line.nth(1).unwrap();
    // println!("{}", request_line);
    let request_line = request_line.get(1..).unwrap();
    // println!("{}", request_line);
    if request_line.is_empty() {
        // println!("Empty request");
        return (
            "HTTP/1.1 200 OK".to_string(),
            "resources/hello.html".to_string(),
        );
    }

    if request_line.contains(".") {
        // println!("resources/{request_line}");
        return (
            "HTTP/1.1 200 OK".to_string(),
            "resources/".to_owned() + request_line,
        );
    };

    let possible_requests = fs::read_to_string("resources/possible_requests.txt").unwrap();
    if !possible_requests.contains(request_line) {
        return (
            "HTTP/1.1 404 NOT FOUND".to_string(),
            "resources/404.html".to_string(),
        );
    }

    (
        "HTTP/1.1 200 OK".to_string(),
        "resources/".to_owned() + request_line + ".html",
    )
}
