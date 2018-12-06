extern crate kv_raft;

use std::sync::Arc;
use kv_raft::kv::server::KVServer;
use grpcio::{Environment, ServerBuilder};
use kv_raft::protos::kvservice_grpc;
use futures::Future;
use futures::sync::oneshot;
use std::thread;
use std::io::{self,Read};
use raft::storage::MemStorage;
use std::env;


fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 4 {
        panic!();
    }

    let num_servers = args[1].parse::<u64>().unwrap();
    let server_id = args[2].parse::<u64>().unwrap();
    let base_port = args[3].parse::<u64>().unwrap();
    let mut addresses = vec![];
    for i in 1..num_servers+1 {
        addresses.push(generate_address(base_port,i));
    }

    let env = Arc::new(Environment::new(1));
    let kv = KVServer::new(format!("testdb{}",server_id),MemStorage::new(),server_id,num_servers,addresses);
    let service = kvservice_grpc::create_kv_service(kv);
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1",(base_port+server_id) as u16)
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

fn generate_address(base_port:u64, id:u64) -> String {
    format!("127.0.0.1:{}",base_port+id)
}