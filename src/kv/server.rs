// kv rpc server

extern crate rocksdb;
//extern crate raft;

use rocksdb::{DB, Writable};
use raft::storage::MemStorage;
use super::super::protos::service::{PutReply,PutReq,State,GetReply,GetReq,Null};
use super::super::protos::service_grpc::{KvService,RaftService};
use raft::eraftpb::Message;
use raft::prelude::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use futures::Future;
use grpcio::{RpcContext, UnarySink};
use super::super::raft_config::config;
use super::super::raft_config::server::RaftServer;
use std::sync::mpsc::{Sender,Receiver,self};

#[derive(Clone)]
pub struct KVServer {
    db:Arc<DB>,
    sender:Sender<config::Msg>,    // for propose raft
    seq:u64,                       // operation sequence number
//    cbs:HashMap<u64, Box<Fn()>>,    // hold callbacks
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Op {
    Put{key:String, val:String, seq:u64},
    Get{key:String, seq:u64},
}

impl KVServer {
    // new kv server and related raft server
    pub fn new (
        db_path:String,
        raft_storage:MemStorage,
        server_id:u64,
        num_servers:u64,
        addresses:Vec<String>,
    ) -> (KVServer,RaftServer) {
        let db = DB::open_default(&db_path).unwrap();

        // run raft node
        let (rs, rr) = mpsc::channel(); // for append raft
        let (apply_s,apply_r) = mpsc::channel(); // for get apply msg from raft
        thread::spawn(move || {
            config::init_and_run(raft_storage,rr,apply_s,server_id,num_servers,addresses);
        });

        let kv_server = KVServer{
            db:Arc::new(db),
            sender:rs.clone(),
            seq:0
        };
        let raft_server = RaftServer{
            sender:rs,
        };

        let db = kv_server.db.clone();
        thread::spawn(move || {
            apply_daemon(apply_r, db);
        });

        return (kv_server,raft_server)
    }
}

impl KvService for KVServer {
    fn get(&mut self, ctx:RpcContext, req:GetReq, sink:UnarySink<GetReply>) {
        let(s1,r1) = mpsc::channel();
        let db = Arc::clone(&self.db);
        let sender = self.sender.clone();
        let op = Op::Get {key:String::from(req.get_key()),seq:self.seq};
        self.seq+=1;
        // propose get request to raft
        sender.send(
            config::Msg::Propose {
                op,
                cb: Box::new(move |is_leader:bool| {
                    // Get
                    let mut reply = GetReply::new();
                    if !is_leader{
                        reply.set_state(State::WRONG_LEADER);
                    } else {
                        let (state, value) = match db.get(req.get_key().as_bytes()) {
                            Ok(Some(v)) => (State::OK, String::from(v.to_utf8().unwrap())),
                            Ok(None) => (State::NOT_FOUND, String::from("")),
                            Err(e) => (State::IO_ERROR, String::from(e))
                        };
                        reply.set_state(state);
                        reply.set_value(value);
                    }
                    // done job, wake
                    s1.send(reply).expect("cb channel closed");
                }),
            }).unwrap();
        // wait job done
        let reply = match r1.recv_timeout(Duration::from_secs(3)){
            Ok(r) => r,
            Err(_e) => {
                let mut r = GetReply::new();
                r.set_state(State::IO_ERROR);
                r
            }
        };
        let f = sink
            .success(reply.clone())
            .map_err(move |err| eprintln!("Failed to reply get: {:?}", err));
        ctx.spawn(f);
    }

    fn put(&mut self, ctx:RpcContext, req:PutReq, sink:UnarySink<PutReply>) {
        println!("get put request");
        let(s1,r1) = mpsc::channel();
        let db = Arc::clone(&self.db);
        let sender = self.sender.clone();
        let op = Op::Put {key:String::from(req.get_key()), val:String::from(req.get_value()),seq:self.seq};
        self.seq += 1;
        // propose put request to raft
        println!("send requst");
        sender.send(
            config::Msg::Propose {
                op,
                cb: Box::new(move |is_leader:bool| {
                    let mut reply = PutReply::new();
                    reply.set_state(if is_leader {State::OK} else {State::WRONG_LEADER});
                    // done job, wake
                    s1.send(reply).expect("cb channel closed");
                }),
            }).unwrap();
        // wait job done
        println!("send done");
        let reply = match r1.recv_timeout(Duration::from_secs(3)){
            Ok(r) => r,
            Err(_e) => {
                let mut r = PutReply::new();
                r.set_state(State::IO_ERROR);
                r
            }
        };
        println!("get reply");
        let f = sink
            .success(reply.clone())
            .map_err(move |err| eprintln!("Failed to reply put: {:?}", err));
        ctx.spawn(f);
    }
}

// wait for applied entries
fn apply_daemon(receiver:Receiver<Op>, db:Arc<DB>) {
    loop {
        let op = match receiver.recv() {
            Ok(o) => o,
            _ => {
                println!("apply dammon return");
                return;
            }
        };
        match op {
            Op::Get {key:_k, seq:_s} => {}  // get done by leader
            Op::Put {key, val, seq:_seq} => {
                println!("put");
                db.put(key.as_bytes(),val.as_bytes()).unwrap();
                println!("put done");
            }
        }
    }
}

