extern crate kv_raft;

use kv_raft::kv::{client,server};
use kv_raft::protos::kvservice::{PutReq, PutReply, GetReq, GetReply, State};
use kv_raft::protos::kvservice_grpc::KvServiceClient;
use grpcio::{ChannelBuilder, EnvBuilder};
use std::env;
use std::sync::Arc;
use std::thread;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        panic!();
    }

    let num_servers = args[1].parse::<u64>().unwrap();
    let base_port = args[2].parse::<u64>().unwrap();
    let mut ids = vec![];
    let mut addresses = vec![];
    for i in 1..num_servers+1 {
        addresses.push(generate_address(base_port,i));
        ids.push(i);
    }


    let mut kv_client = client::KVClient::new(ids,addresses);

    for i in 3001..4000 {
        kv_client.put(format!("key{}", i), format!("value{}", i));
        println!("put done");
        let value = kv_client.get(format!("key{}", i));
        println!("get value:{}",value);
        thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("3\n");

//    for i in 3001..4000 {
//        let key = format!("key{}",i);
//        let value = kv_client.get(key);
//        println!("get value:{}",value);
//    };

    thread::sleep(std::time::Duration::from_secs(60));
}

fn generate_address(base_port:u64, id:u64) -> String {
    format!("127.0.0.1:{}",base_port+id)
}