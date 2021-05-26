use std::convert::Infallible;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncWrite};
use tokio::io::AsyncRead;
use tonic::transport::server::Connected;
use tonic::{transport::Server, Request, Response, Status};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};
use std::os::unix::net::UnixStream as StdUnixStream;
use std::os::unix::io::{AsRawFd, FromRawFd};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[derive(Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let mut ct = 0;
        loop {
            println!("tick {}", ct);
            ct += 1;
            tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (keep, to_send) = StdUnixStream::pair().unwrap();
    let raw = to_send.as_raw_fd();
    let without_cloexec = nix::unistd::dup(raw).unwrap();
    let _without_cloexec_stream = unsafe { StdUnixStream::from_raw_fd(without_cloexec) };
    std::mem::drop(to_send);
    let stream = tokio::net::UnixStream::from_std(
        keep
    ).unwrap();
    // let addr = "[::1]:50051".parse().unwrap();
    let greeter = MyGreeter::default();

    println!("GreeterServer listening on {}", without_cloexec);
    
    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve_with_incoming(tokio::stream::once(Ok::<Uds, Infallible>(Uds(stream))))
        .await?;
    tokio::process::Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("helloworld-client")
        .arg(&format!("{}", without_cloexec))
        .spawn()
        .unwrap()
        .wait_with_output().await.unwrap();
    Ok(())
}
/// Wrapper around tokio's UnixStream
#[derive(Debug)]
pub struct Uds(pub tokio::net::UnixStream);

impl Connected for Uds {}

impl AsyncRead for Uds {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let pinned_inner = Pin::new(&mut self.0);
        pinned_inner.poll_read(cx, buf)
    }
}

impl AsyncWrite for Uds {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let pinned_inner = Pin::new(&mut self.0);
        pinned_inner.poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}