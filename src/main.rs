use std::{fs::{self, read_to_string}, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}};
use threadpool::ThreadPool;

use serde_json::Value;


fn main() {
	println!("Checking for json files...");

	if !fs::metadata("config.json").is_ok() {
		println!("Copying config.json");
		copy_default_file("config.json");
	}

	if !fs::metadata("routes.json").is_ok() {
		println!("Copying routes.json");
		copy_default_file("routes.json");
	}

	println!("Getting config data...");

	let host: &str;
	let port: u16;

	let config_data = get_json_file("config.json".to_string());

	host = config_data["host"].as_str().unwrap();
	port = config_data["port"].as_u64().unwrap() as u16;

	println!("Binding TCP listener to {}:{}", host, port);

	let listener: TcpListener = TcpListener::bind((host, port)).unwrap();

	let pool: ThreadPool = ThreadPool::new(16);

	for stream in listener.incoming() {
		let stream = stream.unwrap();
		let routes: Value = get_json_file("routes.json".to_string());

		pool.execute(|| handle_conn(stream, routes));
	}
}

fn copy_default_file(name: &str) {
	let _ = fs::write(name, "");
	let _ = fs::copy(format!("defaults/{}", name), name);
}

fn get_json_file(path: String) -> Value {
	let content = read_to_string(path).unwrap();

	let json: Value = serde_json::from_str(&content).unwrap();

	return json;
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

	let status: String;
	let content: String;

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