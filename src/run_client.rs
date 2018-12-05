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

    let client= Arc::new(client);
    let client1 = client.clone();
    thread::spawn( move|| {
        let kv_client = KVClient{client};
        for i in 600..701 {
            kv_client.put(format!("key{}", i), format!("value{}", i));
            thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    thread::spawn(move || {
        let kv_client = KVClient{client:client1};
        for i in 600..701 {
            let key = format!("key{}",i);
            let value = kv_client.get(key);
            println!("get value:{}",value);
            thread::sleep(std::time::Duration::from_millis(10));
        }
    });
    thread::sleep(std::time::Duration::from_secs(5));
}

struct KVClient {
    client:Arc<KvServiceClient>,
}

impl KVClient {
    pub fn get(&self, key:String) -> String {
        let mut req = GetReq::new();
        req.set_key(key);
        let reply = self.client.get(&req).expect("Get Failed!");
        match reply.get_state() {
            State::OK => {
                return String::from(reply.get_value());
            },
            State::NOT_FOUND => {
                println!("Not found");
                return String::from("");
            },
            _ => {
                println!("get error!");
                String::from("")
            },
        }
    }

    pub fn put(&self, key:String, value:String) {
        let mut req = PutReq::new();
        req.set_key(key);
        req.set_value(value);
        let reply = self.client.put(&req).expect("PUT Failed!");
        match reply.get_state() {
            State::OK => { println!("put success!");},
            _ => {println!("put error!")},
        };
    }
}