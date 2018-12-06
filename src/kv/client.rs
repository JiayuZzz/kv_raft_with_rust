use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::thread;

use grpcio::{ChannelBuilder, EnvBuilder};

use super::server;

use super::super::protos::service::{GetReply, GetReq, PutReply, PutReq, State};
use super::super::protos::service_grpc::KvServiceClient;

pub struct KVClient {
    leader: usize,
    // leader's index in server_ids
    num_servers: usize,
    server_ids: Vec<u64>,
    clients: HashMap<u64, Arc<KvServiceClient>>,
}

impl KVClient {
    pub fn new(server_ids:Vec<u64>, addresses:Vec<String>) ->KVClient {
        let num_servers = server_ids.len();
        if num_servers!=addresses.len(){
            panic!("parameters error");
        }

        let mut clients = HashMap::new();
        let mut i = 0;

        for i in 0..addresses.len() {
            let env = Arc::new(EnvBuilder::new().build());
            let ch = ChannelBuilder::new(env).connect(addresses[i].as_str());
            let client = KvServiceClient::new(ch);
            clients.insert(server_ids[i],Arc::new(client));
        }

        KVClient{leader:0, num_servers, server_ids, clients}
    }

    pub fn get(&mut self, key: String) -> String {
        let mut req = GetReq::new();
        req.set_key(key);
        loop {
            let leader_index = self.leader;
            let client = match self.clients.get(&self.server_ids[leader_index]) {
                Some(c) => c,
                None => {
                    self.leader = 0;
                    continue;
                }
            };

            let reply = match client.get(&req) {
                Ok(r) => r,
                _ => { // maybe leader error
                    let num_servers = self.num_servers;
                    self.leader = (leader_index + 1) % num_servers;
                    continue;
                }
            };

            match reply.get_state() {
                State::OK => {
                    return String::from(reply.get_value());
                }
                State::NOT_FOUND => {
                    println!("Not found");
                    return String::from("");
                }
                State::WRONG_LEADER => {
                    let num_servers = self.num_servers;
                    self.leader = (leader_index + 1) % num_servers;  // try next server
                }
                _ => {
                    println!("get error!");
                    continue;
                }
            }
        }
    }

    pub fn put(&mut self, key: String, value: String) {
        let mut req = PutReq::new();
        req.set_key(key);
        req.set_value(value);
        loop {
            let leader_index = self.leader;
            println!("try {}",leader_index);
            let client = match self.clients.get(&self.server_ids[leader_index]) {
                Some(c) => c,
                None => {
                    self.leader = 0;
                    continue;
                }
            };
            println!("1");
            let reply = match client.put(&req) {
                Ok(r) => r,
                _ => { // maybe leader error
                    let num_servers = self.num_servers;
                    self.leader = (leader_index + 1) % num_servers;
                    continue;
                }
            };
            println!("2");
            match reply.get_state() {
                State::OK => {
                    println!("put {} success!", req.get_key());
                    return;
                }
                State::WRONG_LEADER => {
                    let num_servers = self.num_servers;
                    self.leader = (leader_index + 1) % num_servers;  // try next server
                }
                _ => {
                    println!("put error! retry");
                    continue ;
                }
            };
        }
    }
}