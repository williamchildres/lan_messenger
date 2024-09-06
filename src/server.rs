use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

type Clients = Arc<Mutex<HashMap<String, TcpStream>>>;

fn handle_client(mut stream: TcpStream, clients: Clients, user_name_map: Arc<Mutex<HashMap<String, String>>>) {
    let mut buffer = [0; 512];
    let socket_addr = stream.peer_addr().unwrap().to_string();
    let mut user_name = socket_addr.clone();

    {
        // Add client to the list
        let mut clients_guard = clients.lock().unwrap();
        clients_guard.insert(socket_addr.clone(), stream.try_clone().expect("Failed to clone stream"));
    }

    {
        // Add default socket address to the user_name_map
        let mut name_map_guard = user_name_map.lock().unwrap();
        name_map_guard.insert(socket_addr.clone(), socket_addr.clone());
    }

    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(0) => {
                // Client disconnected
                println!("Client {} disconnected", user_name);
                break;
            }
            Ok(n) => n,
            Err(_) => {
                println!("Error reading from client {}", user_name);
                break;
            }
        };

        let message = String::from_utf8_lossy(&buffer[..bytes_read]);

        // Check if the user is setting their name
        if message.starts_with("/name ") {
            let new_name = message[6..].trim().to_string();
            let mut name_map_guard = user_name_map.lock().unwrap();
            name_map_guard.insert(socket_addr.clone(), new_name.clone());
            user_name = new_name.clone();
            println!("Client {} set their name to {}", socket_addr, user_name);
            continue;
        }

        // Broadcast the message to all other clients
        broadcast_message(&user_name, &message, &clients, &socket_addr);
    }

    // Remove the client from the list when disconnected
    let mut clients_guard = clients.lock().unwrap();
    clients_guard.remove(&socket_addr);

    let mut name_map_guard = user_name_map.lock().unwrap();
    name_map_guard.remove(&socket_addr);
}

fn broadcast_message(user_name: &str, message: &str, clients: &Clients, sender_socket: &str) {
    let mut clients_guard = clients.lock().unwrap();
    for (client_socket, client_stream) in clients_guard.iter_mut() {
        if client_socket != sender_socket {
            let full_message = format!("{}: {}", user_name, message);
            client_stream.write_all(full_message.as_bytes()).expect("Failed to write to client");
            client_stream.flush().expect("Failed to flush stream");
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Could not bind to port 7878");
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let user_name_map = Arc::new(Mutex::new(HashMap::new()));

    println!("Server listening on port 7878");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let clients = Arc::clone(&clients);
                let user_name_map = Arc::clone(&user_name_map);
                thread::spawn(move || {
                    handle_client(stream, clients, user_name_map);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

