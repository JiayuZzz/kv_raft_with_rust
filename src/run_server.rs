extern crate kv_raft;

use std::sync::Arc;
use kv_raft::kv::server::KVServer;
use grpcio::{Environment, ServerBuilder};
use kv_raft::protos::{service_grpc,};
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
    let mut addresses = vec![]; //raft service addresses
    for i in 1..num_servers+1 {
        addresses.push(generate_address(base_port+num_servers,i));
    }

    let env_kv = Arc::new(Environment::new(1));
    let env_raft = Arc::new(Environment::new(1));
    let (kv,raft) = KVServer::new(format!("testdb{}",server_id),MemStorage::new(),server_id,num_servers,addresses);
    let kv_service = service_grpc::create_kv_service(kv);
    let raft_service = service_grpc::create_raft_service(raft);
    let mut kv_server = ServerBuilder::new(env_kv)
        .register_service(kv_service)
        .bind("127.0.0.1",(base_port+server_id) as u16)
        .build()
        .unwrap();
    let mut raft_server = ServerBuilder::new(env_raft)
        .register_service(raft_service)
        .bind("127.0.0.1",(base_port+server_id+num_servers) as u16)
        .build()
        .unwrap();
    kv_server.start();
    raft_server.start();

    for &(ref host, port) in kv_server.bind_addrs() {
        println!("listening on {}:{}", host, port);
    }
    let (tx, rx) = oneshot::channel();
    thread::spawn(move||{
        println!("Press ENTER to exit...");
        let _ = io::stdin().read(&mut [0]).unwrap();
        tx.send(()).unwrap();
    });
    let _ = rx.wait();
    let _ = kv_server.shutdown().wait();
    let _ = raft_server.shutdown().wait();
}

fn generate_address(base_port:u64, id:u64) -> String {
    format!("127.0.0.1:{}",base_port+id)
}