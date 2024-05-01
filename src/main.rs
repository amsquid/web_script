use std::{fs::{self, read_to_string}, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}};
use threadpool::ThreadPool;

use serde_json::Value;

const HOST: &str = "127.0.0.1";
const PORT: u16  = 8080;

fn main() {
	println!("Binding TCP listener to {}:{}", HOST, PORT);

	let listener: TcpListener = TcpListener::bind((HOST, PORT)).unwrap();

	let pool: ThreadPool = ThreadPool::new(16);

	for stream in listener.incoming() {
		let stream = stream.unwrap();
		let routes: Value = get_routes();

		pool.execute(|| handle_conn(stream, routes));
	}
}

fn get_routes() -> Value {
	let routes_content = read_to_string("routes.json").unwrap();

	let routes: Value = serde_json::from_str(&routes_content).unwrap();

	return routes;
}

fn format_response(status: String, content: String) -> String {
	let length = content.len();

	return format!("{status}\r\nContent-Length: {length}\r\n\r\n{content}").to_string();
}

fn handle_conn(mut stream: TcpStream, routes: Value) {
	// Setting up reader
	let buf_reader = BufReader::new(&mut stream);
	let _http_request: Vec<_> = buf_reader
		.lines()
		.map(|result| result.unwrap())
		.take_while(|line| !line.is_empty())
		.collect();
	
	let request_line: Vec<&str> = _http_request[0].split(" ").collect();
	let request_url = request_line.get(1).unwrap();

	let mut status: String = "".to_string();
	let mut content: String = "".to_string();

	// WWW routes
	if routes["www"][request_url].is_null() {
		println!("WWW resource not found for {}", request_url);

		status = "HTTP/1.1 404 Not Found".to_string();
		
		let mut route_path = "www/".to_string();
		route_path.push_str(routes["www"]["404"].as_str().unwrap());

		content = fs::read_to_string(route_path).unwrap();


	} else {
		status = "HTTP/1.1 200 OK".to_string();

		let mut route_path = "www/".to_string();
		route_path.push_str(routes["www"][request_url].as_str().unwrap());

		content = fs::read_to_string(route_path).unwrap();
	}

	// Script routes
	if routes["scripts"][request_url].is_null() {
		println!("Script not found for {}", request_url);
	} else {
		let mut route_path = "scripts/".to_string();
		route_path.push_str(routes["scripts"][request_url].as_str().unwrap());

		let script_content = fs::read_to_string(route_path).unwrap();

		let lua = rlua::Lua::new();

		let _ = lua.load(script_content).exec();
	}

	let response = format_response(status, content);

	stream.write_all(response.as_bytes()).unwrap();
}