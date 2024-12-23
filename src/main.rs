use std::net::SocketAddr;

use tokio::{
    io::{copy_bidirectional, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UnixStream},
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:41315").await.unwrap();
    loop {
        match listener.accept().await {
            Err(err) => eprintln!("Failed to accept connection: {}", err),
            Ok((stream, addr)) => {
                println!("Successfully received connection from {}", addr);
                tokio::spawn(async move {match handle_connection(stream, &addr).await {
                    Ok(()) => {},
                    Err(err) => eprintln!("There was a problem with the connection to {}: {}", addr, err)
                }});
            }
        }
    }
}

async fn handle_connection(mut stream: TcpStream, addr: &SocketAddr) -> Result<(), std::io::Error> {
    let length = stream.read_u8().await?;
    let mut buffer = vec![0; length as usize];
    match stream.read_exact(&mut buffer).await {
        Ok(0) => println!("Connection to {} was closed immediately.", addr),
        Err(e) => eprintln!("Couldn't read string from client: {}", e),
        Ok(_) => {
            if buffer == "Trainl 4g 10'000".as_bytes() {
                // (we have only the highest of security around here)
                match stream.write_u8(0).await {
                    Ok(()) => {
                        println!("Client from {} logged in successfully.", addr);
                        match make_truinbridge(stream).await {
                            Err(err) => eprintln!("Connection with client {} ended due to error: {}", addr, err),
                            Ok((to_truin, to_client)) => println!(
                                "Connection with client {} terminated after sending {} bytes to truin and {} bytes to the client.", 
                                addr, 
                                to_truin,
                                to_client
                            )
                        }},
                    Err(err) => eprintln!("Couldn't send login confirmation to client: {}", err)
                };
            } else {
                println!(
                    "Client {} tried connecting with incorrect password: {}.", 
                    addr, 
                    String::from_utf8(buffer).unwrap_or("(sent byte sequence wasn't valid UTF-8)".into()));
                if let Err(err) = stream.write_u8(1).await  {
                    eprintln!("Couldn't send login failure byte to client: {}", err)
                }
            }
        }
    };
    Ok(())
}

async fn make_truinbridge(mut stream: TcpStream) -> Result<(u64, u64), std::io::Error> {
    let mut truinstream = UnixStream::connect("/tmp/truinsocket").await.unwrap();
    copy_bidirectional(&mut stream, &mut truinstream).await
}
