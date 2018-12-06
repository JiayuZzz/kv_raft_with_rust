use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::thread;

use grpcio::{ChannelBuilder, EnvBuilder};

use super::server;

use super::super::protos::service::{GetReply, GetReq, PutReply, PutReq, State};
use super::super::protos::service_grpc::KvServiceClient;
use bincode::deserialize;

pub struct KVClient {
    leader: u64, // leader's index in server_ids
    server_ids: Vec<u64>,
    clients: HashMap<u64, Arc<KvServiceClient>>,
    try_id_index : usize,
}

fn create_client(address:String) -> KvServiceClient{
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(&address);
    let client = KvServiceClient::new(ch);
    client
}

fn trans_addr(raft_addr:&String) -> String {
    let prefix_len = raft_addr.len()-4;
    let port = raft_addr[prefix_len..].parse::<u16>().unwrap();
    let prefix= &raft_addr[0..prefix_len];
    let new_addr = format!("{}{}",prefix,port-1000);
    println!("{}",new_addr);
    new_addr
}

impl KVClient {
    pub fn new(server_id:u64,address:String) ->KVClient {
        let mut clients = HashMap::new();

        let client = create_client(address);
        clients.insert(server_id.clone(),Arc::new(client));

        KVClient{leader:server_id.clone(), server_ids:vec![server_id], clients, try_id_index:0}
    }

    pub fn get(&mut self, key: String) -> String {
        let mut req = GetReq::new();
        req.set_key(key);
        loop {
            let leader = self.leader;
            let client = match self.clients.get(&leader) {
                Some(c) => c,
                None => {
                    self.try_id_index = (self.try_id_index+1)%self.server_ids.len();
                    self.leader = self.server_ids[self.try_id_index];
                    continue;
                }
            };

            let reply = match client.get(&req) {
                Ok(r) => r,
                _ => { // maybe leader error
                    self.try_id_index = (self.try_id_index+1)%self.server_ids.len();
                    self.leader = self.server_ids[self.try_id_index];
                    continue;
                }
            };

            // try add new node
            if reply.get_address_map()!=""{
                let address_map:HashMap<u64,String> = deserialize(&reply.get_address_map().as_bytes()).unwrap();
                for (id,address) in &address_map {
                    if let Some(v) = self.clients.get(id) {
                        continue;
                    }
                    println!("insert {} client",id);
                    let client = create_client(trans_addr(address));
                    self.clients.insert(id.clone(),Arc::new(client));
                    self.server_ids.push(id.clone());
                }
            }

            match reply.get_state() {
                State::OK => {
                    return String::from(reply.get_value());
                }
                State::NOT_FOUND => {
                    println!("Not found");
                    return String::from("");
                }
                State::WRONG_LEADER => {
                    self.leader = reply.get_leader_id();
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
            let leader = self.leader;
            let client = match self.clients.get(&leader) {
                Some(c) => c,
                None => {
                    self.try_id_index = (self.try_id_index+1)%self.server_ids.len();
                    self.leader = self.server_ids[self.try_id_index];
                    continue;
                }
            };
//            println!("1");
            let reply = match client.put(&req) {
                Ok(r) => r,
                _ => { // maybe leader error
                    self.try_id_index = (self.try_id_index+1)%self.server_ids.len();
                    self.leader = self.server_ids[self.try_id_index];
                    println!("try {}",self.leader);
                    continue;
                }
            };
//            println!("2");

            // try add new node
            if reply.get_address_map()!=""{
                println!("get addres");
                let address_map:HashMap<u64,String> = deserialize(&reply.get_address_map().as_bytes()).unwrap();
                for (id,address) in &address_map {
                    if let Some(v) = self.clients.get(id) {
                        continue;
                    }
                    println!("insert {} client",id);
                    let client = create_client(trans_addr(address));
                    self.clients.insert(id.clone(),Arc::new(client));
                    self.server_ids.push(id.clone());
                }
            }
//            println!("3");
            match reply.get_state() {
                State::OK => {
                    println!("put {} success!", req.get_key());
                    return;
                }
                State::WRONG_LEADER => {
//                    println!("wrong leader");
                    self.leader = reply.get_leader_id();
//                    println!("set leader id to {}",self.leader);
                }
                _ => {
                    println!("put error! retry");
                    continue ;
                }
            };
        }
    }
}