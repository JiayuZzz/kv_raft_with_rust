extern crate kv_raft;

use kv_raft::kv::server;
use kv_raft::protos::kvservice::{PutReq, PutReply, GetReq, GetReply, State};
use kv_raft::protos::kvservice_grpc::KvServiceClient;
use grpcio::{ChannelBuilder, EnvBuilder};
use std::env;
use std::sync::Arc;
use std::thread;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        panic!();
    }

    let port = args[1].parse::<u16>().unwrap();

    let env = Arc::new(EnvBuilder::new().build());
    let ch  = ChannelBuilder::new(env).connect(format!("localhost:{}",port).as_str());
    let client = KvServiceClient::new(ch);

    let mut put = PutReq::new();
    put.set_key(String::from("hello"));
    put.set_value(String::from("world"));
    let reply = client.put(&put).expect("PUT Failed!");
    match reply.get_state() {
        State::OK => { println!("put success!");},
        _ => {println!("put error!")},
    };
    let mut get = GetReq::new();
    get.set_key(String::from("hello"));
    let reply = client.get(&get).expect("GET Failed!");
    match reply.get_state() {
        State::OK => {
            println!("get {}",reply.get_value());
        },
        State::NOT_FOUND => { println!("Not found");},
        _ => { println!("get error!");},
    }
}