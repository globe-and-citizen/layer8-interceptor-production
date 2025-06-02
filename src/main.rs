use crate::ntor::{client, server};
mod ntor;

fn main() {
    // Create a new client
    let mut client = client::Client::new();

    // Spin up a server
    let server_id = String::from("my server id");
    let mut server   = server::Server::new(server_id);

    // Client initializes session with the server
    let init_session_msg = client.initialise_session();

    // Server accepts the connection request and processes it
    let init_session_response = server.accept_init_session_request(&init_session_msg);

    // Client processes response from the server and verifies it authenticity
    let success_flag = client.handle_response_from_server(&server.get_certificate(), &init_session_response);

    println!("{success_flag}");
}