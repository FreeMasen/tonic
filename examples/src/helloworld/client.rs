use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use std::os::unix::net::UnixStream as StdUnixStream;
use std::os::unix::io::{FromRawFd};
pub mod hello_world {
    tonic::include_proto!("helloworld");
}
use tokio::net::UnixStream;
use tonic::transport::Endpoint;
use futures::future::ok;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = std::env::args().nth(1).unwrap();
    let fd: i32 = arg.parse().unwrap();
    let std_socket = unsafe { StdUnixStream::from_raw_fd(fd) };
    let socket = UnixStream::from_std(std_socket).unwrap_or_else(|e| {
        eprintln!("failed to create unix stream from fd ({}): {}", fd, e);
        // exit with EX_UNAVAILABLE
        std::process::exit(69);
    });
    let mut socket_once = Some(socket); // HACK: service_fn below needs to be FnMut
    let channel = Endpoint::from_static("null://localhost:50051")
        .connect_with_connector(tower::service_fn(move |_| -> futures::future::Ready<std::io::Result<_>> {
            ok(socket_once.take().unwrap())
        }))
        .await
        .unwrap();
    let mut client = GreeterClient::with_interceptor(channel, move|req: tonic::Request<()>| {
        Ok(req)
    });

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });
    let response = tokio::time::timeout(std::time::Duration::from_secs(2),
    client.say_hello(request)).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
