extern crate kv_raft;

use std::sync::Arc;
use kv_raft::kv::server::KVServer;
use grpcio::{Environment, ServerBuilder};
use kv_raft::protos::kvservice_grpc;
use futures::Future;
use futures::sync::oneshot;
use std::thread;
use std::io::{self,Read};


fn main() {
    let env = Arc::new(Environment::new(1));
    let kv = KVServer::new(String::from("./testdb"));
    let service = kvservice_grpc::create_kv_service(kv);
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1",0)
        .build()
        .unwrap();
    server.start();
    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);
    }
    let (tx, rx) = oneshot::channel();
    thread::spawn(move||{
        println!("Press ENTER to exit...");
        let _ = io::stdin().read(&mut [0]).unwrap();
        tx.send(()).unwrap();
    });
    let _ = rx.wait();
    let _ = server.shutdown().wait();
}