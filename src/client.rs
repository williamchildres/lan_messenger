use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::thread;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").expect("Failed to connect to server");
    println!("Connected to server at 127.0.0.1:7878");

    // Spawn a thread to handle incoming messages from the server
    let mut stream_clone = stream.try_clone().expect("Failed to clone stream");
    thread::spawn(move || {
        let mut buffer = [0; 512];
        loop {
            let bytes_read = stream_clone.read(&mut buffer).expect("Failed to read from server");
            if bytes_read > 0 {
                print!("{}", String::from_utf8_lossy(&buffer[..bytes_read]));
                io::stdout().flush().unwrap();
            }
        }
    });

    // Handle sending messages to the server
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        stream.write_all(input.as_bytes()).expect("Failed to write to server");
        stream.flush().expect("Failed to flush stream");
    }
}

