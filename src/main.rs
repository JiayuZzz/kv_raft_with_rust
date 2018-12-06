extern crate kv_raft;

use std::sync::Arc;
use kv_raft::kv::server::KVServer;
use kv_raft::kv::client;
use kv_raft::protos::service::{ChangeReply,State};
use kv_raft::protos::service_grpc::KvServiceClient;
use grpcio::{Environment, ServerBuilder};
use kv_raft::protos::{service_grpc,};
use futures::Future;
use futures::sync::oneshot;
use std::thread;
use std::io::{self,Read};
use raft::storage::MemStorage;
use std::env;
use grpcio::{ChannelBuilder, EnvBuilder};
use raft::prelude::*;
use std::collections::HashMap;
use bincode::{serialize,deserialize};

fn main() {
    let args = env::args().collect::<Vec<_>>();

    let command = args[1].clone();
    if command == "new" {
        let id = args[2].parse::<u64>().unwrap();
        let port = args[3].parse::<u16>().unwrap();
        new_server(port, id,HashMap::new());
    } else if command == "add" {
        let id = args[2].parse::<u64>().unwrap();
        let port = args[3].parse::<u16>().unwrap();
        let leader_port = args[4].parse::<u16>().unwrap();
        let address_map = add_server(port,id,leader_port);
        new_server(port,id, address_map);
    }
}

fn new_server(port:u16, server_id:u64, addresses:HashMap<u64,String>) {
    let raft_address = format!(
        "127.0.0.1:{}",port+1000
    );
    let env_kv = Arc::new(Environment::new(2));
    let env_raft = Arc::new(Environment::new(2));
    let (kv,raft) = KVServer::new(format!("testdb{}",server_id),MemStorage::new(),server_id,raft_address,addresses);
    let kv_service = service_grpc::create_kv_service(kv);
    let raft_service = service_grpc::create_raft_service(raft);
    let mut kv_server = ServerBuilder::new(env_kv)
        .register_service(kv_service)
        .bind("127.0.0.1",port)
        .build()
        .unwrap();
    let mut raft_server = ServerBuilder::new(env_raft)
        .register_service(raft_service)
        .bind("127.0.0.1",port+1000)
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

fn add_server(port:u16, server_id:u64, leader_port:u16) -> HashMap<u64,String> {
    let raft_address = format!(
        "127.0.0.1:{}",port+1000
    );

    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(format!("127.0.0.1:{}",leader_port).as_str());
    let client = KvServiceClient::new(ch);
    let mut change = ConfChange::new();
    change.set_node_id(server_id);
    change.set_change_type(ConfChangeType::AddNode);
    change.set_context(serialize(&raft_address).unwrap());

    let mut reply = ChangeReply::new();
    let mut reply = match client.change_config(&change){
        Ok(r) => r,
        _ => {panic!("add error rpc!")}
    };
    match reply.get_state() {
        State::OK => {},
        _ => {panic!("add error reply!")},
    };
    if reply.get_address_map()!=""{
        let address_map:HashMap<u64,String> = deserialize(&reply.get_address_map().as_bytes()).unwrap();
        println!("not null");
        return address_map;
    }
    HashMap::new()
}