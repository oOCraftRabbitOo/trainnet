use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use truinlag::api::{error::Error, insert_connection, InactiveRecvConnection, SendConnection};

pub async fn connect(
    passphrase: String,
) -> Result<(SendConnection, InactiveRecvConnection), Error> {
    let mut stream = TcpStream::connect("127.0.0.1:41315").await?;
    let passphrase = passphrase.as_bytes();
    let length: u8 =
        passphrase
            .len()
            .try_into()
            .or(Err(Error::Connection(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "The passphrase can't exceed a size of 255 bytes.",
            ))))?;
    stream.write_u8(length).await?;
    stream.write_all(passphrase).await?;
    let response = stream.read_u8().await?;
    match response {
        0 => {
            let (tcp_read, tcp_write) = stream.into_split();
            insert_connection(tcp_read, tcp_write).await
        }
        _ => Err(Error::Connection(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Incorrect passphrase",
        ))),
    }
}
