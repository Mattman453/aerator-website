use aerator_website::ThreadPool;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::available_parallelism;

fn main() {
    let listener = TcpListener::bind(("127.0.0.1", 7878)).unwrap();
    let pool = ThreadPool::new(available_parallelism().unwrap().into());
    let stop = Arc::new(AtomicBool::new(false));

    for stream in listener.incoming() {
        if stop.load(Ordering::Relaxed) == true {
            break;
        }

        let stream = stream.unwrap();
        let bool = stop.clone();
        pool.execute(|| {
            handle_connection(stream, bool);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(stream: TcpStream, stop: Arc<AtomicBool>) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next(); //.unwrap().unwrap();
    let request_line = unwrap_line(request_line);
    if request_line.is_empty() {
        return;
    }

    if request_line.contains("q7w8e9r0") {
        handle_closing(&stop, stream);
        return;
    }

    let (status_line, filename) = process_request(request_line);
    // println!("{filename}");

    if filename.contains(".jpg") || filename.contains(".jpeg") || filename.contains(".png") {
        handle_images(filename, stream);
        return;
    }

    handle_html(filename, status_line, stream);
}

fn handle_html(filename: String, status_line: String, mut stream: TcpStream) {
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn unwrap_line(request_line: Option<Result<String, Error>>) -> String {
    if request_line.is_none() {
        // println!("No lines to process");
        return "".to_string();
    }
    let request_line = request_line.unwrap();

    if request_line.is_err() {
        return "".to_string();
    }

    request_line.unwrap()
}

fn handle_closing(stop: &Arc<AtomicBool>, mut stream: TcpStream) {
    stop.store(true, Ordering::Relaxed);
    let status_line = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string("resources/html/closed.html").unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn handle_images(filename: String, mut stream: TcpStream) {
    let mut file = File::open(&filename).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    let length = contents.len();
    let response: String;

    if filename.contains(".jpg") {
        response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: image/jpg\r\nContent-Length: {length}\r\n\r\n"
        );
    } else if filename.contains(".jpeg") {
        response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: {length}\r\n\r\n"
        );
    } else {
        response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {length}\r\n\r\n"
        );
    }

    stream.write_all(&response.as_bytes()).unwrap();
    stream.write_all(&contents).unwrap();
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
    // println!("{}", holder);
    let request_line = holder.as_str();

    if request_line.is_empty() {
        return (
            "HTTP/1.1 200 OK".to_string(),
            "resources/html/hello.html".to_string(),
        );
    }

    if request_line.contains(".css") {
        return (
            "HTTP/1.1 200 OK".to_string(),
            "resources/".to_owned() + request_line,
        );
    };

    if request_line.contains(".jpg")
        || request_line.contains(".jpeg")
        || request_line.contains(".png")
    {
        return if request_line.contains("background") {
            (
                "HTTP/1.1 200 OK".to_string(),
                "resources/".to_owned() + request_line,
            )
        } else {
            (
                "HTTP/1.1 200 OK".to_string(),
                "resources/html/".to_owned() + request_line,
            )
        };
    }

    let possible_requests = fs::read_to_string("resources/possible_requests.txt").unwrap();

    if !possible_requests.contains(request_line) {
        return (
            "HTTP/1.1 404 NOT FOUND".to_string(),
            "resources/html/404.html".to_string(),
        );
    }

    if request_line.contains(".html") {
        return (
            "HTTP/1.1 200 OK".to_string(),
            "resources/html/".to_owned() + request_line,
        );
    }

    (
        "HTTP/1.1 200 OK".to_string(),
        "resources/html/".to_owned() + request_line + ".html",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Ensure an empty is returned
    fn test_trim_request1() {
        let request_line = "GET / HTTP/1.1";
        let result = trim_request(request_line.to_string());
        assert_eq!(result, "");
    }

    #[test]
    /// Ensure the correct css filename is returned
    fn test_trim_request2() {
        let request_line = "GET /hello.css HTTP/1.1";
        let result = trim_request(request_line.to_string());
        assert_eq!(result, "hello.css");
    }

    #[test]
    /// Ensure the correct phrase is found "warm_grass"
    fn test_trim_request3() {
        let request_line = "GET /warm_grass HTTP/1.1";
        let result = trim_request(request_line.to_string());
        assert_eq!(result, "warm_grass");
    }

    #[test]
    /// Ensure proper trimming down to types
    fn test_trim_request4() {
        let request_line = "GET /types HTTP/1.1";
        let result = trim_request(request_line.to_string());
        assert_eq!(result, "types");
    }

    #[test]
    /// Ensure the proper response is generated and the correct file is added
    fn test_process_request1() {
        let request_line = "GET / HTTP/1.1";
        let (status_line, filename) = process_request(request_line.to_string());
        assert_eq!(status_line, "HTTP/1.1 200 OK");
        assert_eq!(filename, "resources/html/hello.html");
    }

    #[test]
    /// Ensure the proper response is generated and the correct file is generated
    fn test_process_request2() {
        let request_line = "GET /css/hello.css HTTP/1.1";
        let (status_line, filename) = process_request(request_line.to_string());
        assert_eq!(status_line, "HTTP/1.1 200 OK");
        assert_eq!(filename, "resources/css/hello.css");
    }

    #[test]
    /// Ensure the proper response is generated and the correct file is requested
    fn test_process_request3() {
        let request_line = "GET /warm_grass HTTP/1.1";
        let (status_line, filename) = process_request(request_line.to_string());
        assert_eq!(status_line, "HTTP/1.1 200 OK");
        assert_eq!(filename, "resources/html/warm_grass.html");
    }

    #[test]
    /// Ensure the error response is generated and the not found file is requested
    fn test_process_request4() {
        let request_line = "GET /types HTTP/1.1";
        let (status_line, filename) = process_request(request_line.to_string());
        assert_eq!(status_line, "HTTP/1.1 404 NOT FOUND");
        assert_eq!(filename, "resources/html/404.html");
    }
}
