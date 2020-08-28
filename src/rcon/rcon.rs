/*
    Flow:
        Establish network connection
        Log in
        packet loop
*/

use super::{
    packet::{read, write, Packet},
    PacketType,
};
use crate::error::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct Connection<S: AsyncReadExt + AsyncWriteExt + Unpin> {
    stream: S,
    next_request_id: i32,
}

pub async fn connect(addr: &str, password: String) -> Result<Connection<TcpStream>, Error> {
    let stream = TcpStream::connect(addr).await?;
    let mut conn = Connection {
        stream,
        next_request_id: 100,
    };
    conn.login(password).await?;
    Ok(conn)
}

impl<S: AsyncReadExt + AsyncWriteExt + Unpin> Connection<S> {
    pub async fn run_command(&mut self, command: &str) -> Result<String, Error> {
        // TODO: Fix ownership rules for strings in packets
        let cmd = Packet::new(self.gen_request_id(), PacketType::Command, command.into())?;
        let followup = Packet::new(self.gen_request_id(), PacketType::Invalid, "".into())?;
        let mut responses = vec![];
        self.send_packet(&cmd).await?;
        self.send_packet(&followup).await?;

        loop {
            match self.receive_packet().await? {
                Packet {
                    request_id: id,
                    payload,
                    ..
                } if id == cmd.request_id => responses.push(payload),
                Packet { request_id: id, .. } if id == followup.request_id => break,
                _ => return Err("Bad packet id received".into()),
            }
        }
        Ok(responses.join(""))
    }

    async fn login(&mut self, password: String) -> Result<(), Error> {
        // Precondition: this is the first packet sent
        // Therefore, we do not expect to get an unrelated packet back after sending this login
        let request_id = self.gen_request_id();
        let login_packet = Packet::new(request_id, PacketType::Login, password)?;
        self.send_packet(&login_packet).await?;
        let response = self.receive_packet().await?;
        match response.request_id {
            id if id == request_id => Ok(()),
            -1 => Err("Invalid RCon password".into()),
            other_id => Err(format!("Unknown response packet with id {}", other_id).into()),
        }
    }

    async fn send_packet(&mut self, packet: &Packet) -> Result<(), Error> {
        Ok(write(packet, &mut self.stream).await?)
    }

    async fn receive_packet(&mut self) -> Result<Packet, Error> {
        Ok(read(&mut self.stream).await?)
    }

    fn gen_request_id(&mut self) -> i32 {
        let curr_id = self.next_request_id;
        self.next_request_id = curr_id + 1;
        curr_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pin_project_lite::pin_project;
    use std::{
        io::Cursor,
        pin::Pin,
        task::{Context, Poll},
    };
    use tokio::io::{AsyncRead, AsyncWrite};

    #[tokio::test]
    async fn test_successful_login() -> Result<(), Error> {
        let mut conn = fake_connection();
        let login_response =
            Packet::new(conn.next_request_id, PacketType::Login, "password".into())?;
        write(&login_response, &mut conn.stream.input).await?;
        conn.stream.input.set_position(0);
        conn.login("password".into()).await?; // Would error if the login failed
        Ok(())
    }

    #[tokio::test]
    async fn test_bad_password_login() -> Result<(), Error> {
         let mut conn = fake_connection();
        let login_response =
            Packet::new(-1, PacketType::Login, "".into())?;
        write(&login_response, &mut conn.stream.input).await?;
        conn.stream.input.set_position(0);
        let login_result = conn.login("password".into()).await;
        assert!(login_result.is_err()); // TODO: This could be a more general error but error handling is bad right now
        Ok(())
    }

    fn fake_connection() -> Connection<FakeReadWrite> {
        Connection {
            next_request_id: 100,
            stream: FakeReadWrite::new(),
        }
    }

    // Fake struct over 2 buffers so we can pretend to be a tcp socket for tests
    pin_project! {
        struct FakeReadWrite {
            #[pin]
            input: Cursor<Vec<u8>>,
            #[pin]
            output: Cursor<Vec<u8>>
        }
    }

    impl FakeReadWrite {
        fn new() -> Self {
            FakeReadWrite {
                input: Cursor::new(Vec::new()),
                output: Cursor::new(Vec::new()),
            }
        }
    }

    impl AsyncRead for FakeReadWrite {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<std::io::Result<usize>> {
            self.project().input.poll_read(cx, buf)
        }
    }

    impl AsyncWrite for FakeReadWrite {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize, std::io::Error>> {
            self.project().output.poll_write(cx, buf)
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), std::io::Error>> {
            self.project().output.poll_flush(cx)
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), std::io::Error>> {
            self.project().output.poll_shutdown(cx)
        }
    }
}
