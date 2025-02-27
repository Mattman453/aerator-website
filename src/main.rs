use aerator_website::ThreadPool;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::available_parallelism;

fn main() {
    let listener = TcpListener::bind(("127.0.0.1", 7878)).unwrap();
    let pool = ThreadPool::new(available_parallelism().unwrap().into());

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
        // println!("No lines to process");
        return;
    }
    let request_line = request_line.unwrap();

    if request_line.is_err() {
        return;
    }
    let request_line = request_line.unwrap();

    let (status_line, filename) = process_request(request_line);

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

/// Takes the request line containing the desired page or file and return just the filename
///
/// The request_line is the string to be processed
fn trim_request(request_line: String) -> String {
    let request_line = request_line.trim();
    let mut request_line = request_line.split(" ");
    let request_line = request_line.nth(1).unwrap();
    // println!("{}", request_line);
    let request_line = request_line.get(1..).unwrap();
    // println!("{}", request_line);

    request_line.to_string()
}

fn process_request(request_line: String) -> (String, String) {
    let holder = trim_request(request_line);
    let request_line = holder.as_str();

    if request_line.is_empty() {
        return (
            "HTTP/1.1 200 OK".to_string(),
            "resources/hello.html".to_string(),
        );
    }

    if request_line.contains(".css") {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_request1() {
        let request_line = "GET / HTTP/1.1";
        let result = trim_request(request_line.to_string());
        assert_eq!(result, "");
    }

    #[test]
    fn test_trim_request2() {
        let request_line = "GET /hello.css HTTP/1.1";
        let result = trim_request(request_line.to_string());
        assert_eq!(result, "hello.css");
    }

    #[test]
    fn test_trim_request3() {
        let request_line = "GET /warm_grass HTTP/1.1";
        let result = trim_request(request_line.to_string());
        assert_eq!(result, "warm_grass");
    }

    #[test]
    fn test_trim_request4() {
        let request_line = "GET /types HTTP/1.1";
        let result = trim_request(request_line.to_string());
        assert_eq!(result, "types");
    }

    #[test]
    fn test_process_request1() {
        let request_line = "GET / HTTP/1.1";
        let (status_line, filename) = process_request(request_line.to_string());
        assert_eq!(status_line, "HTTP/1.1 200 OK");
        assert_eq!(filename, "resources/hello.html");
    }

    #[test]
    fn test_process_request2() {
        let request_line = "GET /hello.css HTTP/1.1";
        let (status_line, filename) = process_request(request_line.to_string());
        assert_eq!(status_line, "HTTP/1.1 200 OK");
        assert_eq!(filename, "resources/hello.css");
    }

    #[test]
    fn test_process_request3() {
        let request_line = "GET /warm_grass HTTP/1.1";
        let (status_line, filename) = process_request(request_line.to_string());
        assert_eq!(status_line, "HTTP/1.1 200 OK");
        assert_eq!(filename, "resources/warm_grass.html");
    }

    #[test]
    fn test_process_request4() {
        let request_line = "GET /types HTTP/1.1";
        let (status_line, filename) = process_request(request_line.to_string());
        assert_eq!(status_line, "HTTP/1.1 404 NOT FOUND");
        assert_eq!(filename, "resources/404.html");
    }
}
