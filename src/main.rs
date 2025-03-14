extern crate chunked_transfer;
use aerator_website::ThreadPool;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::available_parallelism;
use chunked_transfer::Encoder;

fn main() {
    let listener = TcpListener::bind(("127.0.0.1", 7878)).unwrap();
    let pool = ThreadPool::new(available_parallelism().unwrap().into());
    let stop = Arc::new(AtomicBool::new(false));

    for stream in listener.incoming() {
        if stop.load(Ordering::Relaxed) == true {
            let status_line = "HTTP/1.1 200 OK";
            let contents = fs::read_to_string("resources/html/closed.html").unwrap();
            let length = contents.len();
            let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

            let mut stream = stream.unwrap();
            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();

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

fn handle_connection(mut stream: TcpStream, stop: Arc<AtomicBool>) {
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
    if request_line.contains("q7w8e9r0") {
        stop.store(true, Ordering::Relaxed);
        return;
    }

    let (status_line, filename) = process_request(request_line);
    // println!("{filename}");

    // if filename.contains(".jpg") {
    //     let mut file = File::open(filename).unwrap();
    //     let mut buf = Vec::new();
    //     file.read_to_end(&mut buf).unwrap();
    //
    //     // println!("File Read.");
    //
    //     let mut encoded = Vec::new();
    //     {
    //         let mut encoder = Encoder::with_chunks_size(&mut encoded, 8);
    //         // println!("Encoder Created");
    //         // encoder.write(&buf).unwrap();
    //     }
    //     // println!("Encoded");
    //
    //     let headers = [
    //         "HTTP/1.1 200 OK",
    //         "Content-type: image/jpeg",
    //         "Transfer-Encoding: chunked",
    //         "\r\n"
    //     ];
    //     let mut response = headers.join("\r\n")
    //         .to_string()
    //         .into_bytes();
    //     response.extend(encoded);
    //
    //     // match stream.write(&response) {
    //     //     Ok(_) => println!("Response sent"),
    //     //     Err(e) => println!("Failed sending response: {e}"),
    //     // }
    //     // stream.flush().unwrap();
    //     return
    // }

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

    if request_line.contains(".jpg") {
        return (
            "HTTP/1.1 200 OK".to_string(),
            "resources/".to_owned() + request_line,
            )
    }

    let possible_requests = fs::read_to_string("resources/possible_requests.txt").unwrap();
    if !possible_requests.contains(request_line) {
        return (
            "HTTP/1.1 404 NOT FOUND".to_string(),
            "resources/html/404.html".to_string(),
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
