use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::{fs, thread};
use std::time::Duration;
use hello::ThreadPool;

#[derive(Debug, Clone)]
struct Route {
    method: &'static str,
    path: &'static str,
    status: i32,
}

fn handle_response(route: Option<Route>) -> (String, i32) {
    match route {
        Some(route) => match (route.method, route.path) {
            ("GET", "/") => (fs::read_to_string("hello.html").unwrap(), route.status),
            ("GET", "/hellos/2") => (fs::read_to_string("hello2.html").unwrap(), route.status),
            ("GET", "/sleep") => {
                thread::sleep(Duration::from_secs(5));
                (fs::read_to_string("hello2.html").unwrap(), route.status)
            },
            _ => (fs::read_to_string("404.html").unwrap(), 404),
        },
        None => (fs::read_to_string("404.html").unwrap(), 404),
    }
}

trait ByteRouteNotation {
    fn get_html_notation(&self) -> String;
}

impl ByteRouteNotation for Route {
    fn get_html_notation(&self) -> String {
        format!("{} {} HTTP/1.1\r\n", self.method, self.path)
    }
}

const HOME: Route = Route{
    method: "GET",
    path: "/",
    status: 200
};

const HELLO2: Route = Route{
    method: "GET",
    path: "/hellos/2",
    status: 200
};

const SLEEP: Route = Route{
    method: "GET",
    path: "/sleep",
    status: 200
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4).unwrap();
    for stream in listener.incoming().take(10) {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream)
        });
    }
    println!("Shutting down");
}

type Buffer = [u8; 1024];

fn get_route(routes: Vec<Route>, buffer: &Buffer) -> Option<Route> {
    routes.into_iter().find(|route| buffer.starts_with(route.get_html_notation().as_bytes()))
}

fn get_status_as_text(status: i32) -> String {
    if status == 404 {
        return String::from("NOT FOUND");
    }
    String::from("OK")
}

fn get_response(route: Option<Route>) -> String {
    let (contents, status) = handle_response(route);
    format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n{}",
        status.to_string(),
        get_status_as_text(status),
        contents.len(),
        contents
    )
}

fn handle_connection(mut stream: TcpStream) {
    let routes = vec![HOME, HELLO2, SLEEP];
    let mut buffer: Buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let route: Option<Route> = get_route(routes, &buffer);
    let response = get_response(route);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
