use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

fn handle_client(mut stream: TcpStream, clients: Arc<Mutex<Vec<TcpStream>>>) {
    let mut buffer = [0; 512];
    
    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(0) => {
                // Client disconnected
                println!("Client disconnected");
                break;
            }
            Ok(n) => n,
            Err(_) => {
                println!("Error reading from client");
                break;
            }
        };

        let message = &buffer[..bytes_read];
        let mut clients_guard = clients.lock().unwrap();
        for client in clients_guard.iter_mut() {
            if client.peer_addr().unwrap() != stream.peer_addr().unwrap() {
                client.write_all(message).expect("Failed to write to client");
                client.flush().expect("Failed to flush client stream");
            }
        }
    }

    // Remove the client from the list when disconnected
    let mut clients_guard = clients.lock().unwrap();
    clients_guard.retain(|client| client.peer_addr().unwrap() != stream.peer_addr().unwrap());
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Could not bind to port 7878");
    let clients: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
    println!("Server listening on port 7878");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let client_addr = stream.peer_addr().unwrap();
                println!("Client connected: {}", client_addr);

                let clients = Arc::clone(&clients);
                {
                    let mut clients_guard = clients.lock().unwrap();
                    clients_guard.push(stream.try_clone().expect("Failed to clone stream"));
                }

                thread::spawn(move || {
                    handle_client(stream, clients);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}
