extern crate rocksdb;
//extern crate raft;

use rocksdb::{DB, Writable};
use raft::prelude::*;
use raft::storage::MemStorage;
use super::super::protos::kvservice::{PutReply,PutReq,State,GetReply,GetReq};
use super::super::protos::kvservice_grpc::KvService;
use std::sync::Arc;
use std::thread;
use futures::Future;
use grpcio::{RpcContext, UnarySink};
use super::super::raft_config::config;
use std::sync::mpsc::{Sender,Receiver,self};

#[derive(Clone)]
pub struct KVServer {
    db:Arc<DB>,
    sender:Sender<config::Msg>    // for propose raft
//    cbs:HashMap<u64, Box<Fn()>>,    // hold callbacks
}

impl KVServer {
    pub fn new (
        db_path:String,
        raft_storage:MemStorage,
    ) -> KVServer {
        let db = DB::open_default(&db_path).unwrap();

        // run raft node
        let (sender, receiver) = mpsc::channel(); // for communication with raft
        thread::spawn(move || {
            config::init_and_run(raft_storage,receiver);
        });

        let server = KVServer{
            db:Arc::new(db),
            sender,
        };

        return server;
    }
}

impl KvService for KVServer {
    fn get(&mut self, ctx:RpcContext, req:GetReq, sink:UnarySink<GetReply>) {
        let(s1,r1) = mpsc::channel();
        let client_id = req.get_client_id() as u8;
        let db = Arc::clone(&self.db);
        let sender = self.sender.clone();
        // propose get request to raft
        sender.send(
            config::Msg::Propose {
                id:client_id,
                cb: Box::new(move || {
                    // Get
                    let mut reply = GetReply::new();
                    let (state,value) = match db.get(req.get_key().as_bytes()) {
                        Ok(Some(v)) => (State::OK, String::from(v.to_utf8().unwrap())),
                        Ok(None) => (State::NOT_FOUND, String::from("")),
                        Err(e) => (State::IO_ERROR, String::from(e))
                    };
                    reply.set_state(state);
                    reply.set_value(value);
                    // done job, wake
                    s1.send(reply).unwrap();
                }),
            }).unwrap();
        // wait job done
        let reply = r1.recv().unwrap();
        let f = sink
            .success(reply.clone())
            .map_err(move |err| eprintln!("Failed to reply get: {:?}", err));
        ctx.spawn(f);
    }

    fn put(&mut self, ctx:RpcContext, req:PutReq, sink:UnarySink<PutReply>) {
        let(s1,r1) = mpsc::channel();
        let client_id = req.get_client_id() as u8;
        let db = Arc::clone(&self.db);
        let sender = self.sender.clone();
        // propose put request to raft
        sender.send(
            config::Msg::Propose {
                id:client_id,
                cb: Box::new(move || {
                    // Put
                    let mut reply = PutReply::new();
                    let state = match db.put(req.key.as_bytes(), req.value.as_bytes()) {
                        Ok(_) => State::OK,
                        _ => State::IO_ERROR
                    };
                    reply.set_state(state);
                    // done job, wake
                    s1.send(reply).unwrap();
                }),
            }).unwrap();
        // wait job done
        let reply = r1.recv().unwrap();
        let f = sink
            .success(reply.clone())
            .map_err(move |err| eprintln!("Failed to reply put: {:?}", err));
        ctx.spawn(f);
    }
}

