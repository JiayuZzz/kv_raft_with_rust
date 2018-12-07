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
    let id = args[2].parse::<u64>().unwrap();
    let port = args[3].parse::<u16>().unwrap();
    if command == "new" {
        new_server(port, id,HashMap::new());
    } else if command == "add" {
        let leader_port = args[4].parse::<u16>().unwrap();
        let address_map = change_server("add",port,id,leader_port);
        new_server(port,id, address_map);
    } else if command == "remove" {
        change_server("remove",0,id,port);
    } else if command == "client" {
        run_client(id,port);
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

fn change_server(change_type:&str, port:u16, server_id:u64, leader_port:u16) -> HashMap<u64,String> { // add or remove server
    let raft_address = format!(
        "127.0.0.1:{}",port+1000
    );
    println!("{}",change_type);

    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(format!("127.0.0.1:{}",leader_port).as_str());
    let client = KvServiceClient::new(ch);
    let mut change = ConfChange::new();
    change.set_node_id(server_id);
    change.set_change_type(if change_type=="add"{ConfChangeType::AddNode} else {ConfChangeType::RemoveNode});
    change.set_context(if change_type=="add"{serialize(&raft_address).unwrap()}else{vec![]});

    let mut reply = ChangeReply::new();
    let mut reply = match client.change_config(&change){
        Ok(r) => r,
        _ => {panic!("add error rpc!")}
    };
    match reply.get_state() {
        State::OK => {},
        _ => {panic!("add error reply!")},
    };
    if reply.get_address_map().len()>0{
        let address_map:HashMap<u64,String> = deserialize(&reply.get_address_map()).unwrap();
        println!("not null");
        return address_map;
    }
    HashMap::new()
}

fn run_client(server_id:u64,server_port:u16) {
    let address = format!("127.0.0.1:{}",server_port);


    let mut kv_client = client::KVClient::new(server_id,address);

    for i in 3001..7000 {
        println!("put begin");
        kv_client.put(format!("key{}", i), format!("value{}", i));
        println!("put done");
        let value = kv_client.get(format!("key{}", i));
        println!("get value:{}", value);
        thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("3\n");

    thread::sleep(std::time::Duration::from_secs(60));
}