extern crate rocksdb;
//extern crate raft;

use rocksdb::{DB, Writable};
use raft::storage::MemStorage;
use super::super::protos::kvservice::{PutReply,PutReq,State,GetReply,GetReq,Null};
use super::super::protos::kvservice_grpc::{KvService};
use raft::eraftpb::Message;
use std::sync::Arc;
use std::thread;
use futures::Future;
use grpcio::{RpcContext, UnarySink};
use super::super::raft_config::config;
use std::sync::mpsc::{Sender,Receiver,self};

#[derive(Clone)]
pub struct KVServer {
    db:Arc<DB>,
    sender:Sender<config::Msg>,    // for propose raft
//    cbs:HashMap<u64, Box<Fn()>>,    // hold callbacks
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Op {
    Put{key:String, val:String},
    Get{key:String},
}

impl KVServer {
    pub fn new (
        db_path:String,
        raft_storage:MemStorage,
        server_id:u64,
        num_servers:u64,
        addresses:Vec<String>,
    ) -> KVServer {
        let db = DB::open_default(&db_path).unwrap();

        // run raft node
        let (rs, rr) = mpsc::channel(); // for append raft
        let (apply_s,apply_r) = mpsc::channel(); // for get apply msg from raft
        thread::spawn(move || {
            config::init_and_run(raft_storage,rr,apply_s,server_id,num_servers,addresses);
        });

        let server = KVServer{
            db:Arc::new(db),
            sender:rs,
        };

        let db = server.db.clone();
        thread::spawn(move || {
            apply_daemon(apply_r, db);
        });

        return server;
    }
}

impl KvService for KVServer {
    fn get(&mut self, ctx:RpcContext, req:GetReq, sink:UnarySink<GetReply>) {
        let(s1,r1) = mpsc::channel();
        let db = Arc::clone(&self.db);
        let sender = self.sender.clone();
        let op = Op::Get {key:String::from(req.get_key())};
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
        let reply = r1.recv().expect("cb channel closed");
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
        let op = Op::Put {key:String::from(req.get_key()), val:String::from(req.get_value())};
        // propose put request to raft
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
        let reply = r1.recv().expect("cb channel closed");
        let f = sink
            .success(reply.clone())
            .map_err(move |err| eprintln!("Failed to reply put: {:?}", err));
        ctx.spawn(f);
    }

    // send raft message
    fn send_msg(&mut self, ctx:RpcContext, req:Message, sink: ::grpcio::UnarySink<Null>) {
        let sender = self.sender.clone();
        println!("get raft msg from {}",req.from);
        sender.send(config::Msg::Raft(req));
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
            Op::Get {key:_key} => {}  // get done by leader
            Op::Put {key, val} => {
                db.put(key.as_bytes(),val.as_bytes()).unwrap();
            }
        }
    }
}

