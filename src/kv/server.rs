extern crate rocksdb;
//extern crate raft;

use rocksdb::{DB, Writable};
//use raft::prelude::*;
//use raft::storage::MemStorage;
use super::super::protos::kvservice::{PutReply,PutReq,State,GetReply,GetReq};
use super::super::protos::kvservice_grpc::KvService;
use std::sync::Arc;
use futures::Future;
use grpcio::{RpcContext, UnarySink};

#[derive(Clone)]
pub struct KVServer {
    db:Arc<DB>,
//    cbs:HashMap<u64, Box<Fn()>>,    // hold callbacks
}

impl KVServer {
    pub fn new(
        db_path:String
//        storage:MemStorage,
//        raft_cfg:Config
    ) -> KVServer {
        let db = DB::open_default(&db_path).unwrap();
        let server = KVServer{
            db:Arc::new(db),
        };
        return server;
//        let mut r = RawNode::new(&raft_cfg,storage,vec![]).unwrap();
    }
}

impl KvService for KVServer {
    fn get(&mut self, ctx:RpcContext, req:GetReq, sink:UnarySink<GetReply>) {
        let mut reply = GetReply::new();
        let (state,value) = match self.db.get(req.get_key().as_bytes()) {
            Ok(Some(v)) => (State::OK, String::from(v.to_utf8().unwrap())),
            Ok(None) => (State::NOT_FOUND, String::from("")),
            Err(e) => (State::IO_ERROR, String::from(e))
        };
        reply.set_state(state);
        reply.set_value(value);
        let f = sink
            .success(reply.clone())
            .map_err(move |err| eprintln!("Failed to reply get: {:?}", err));
        ctx.spawn(f)
    }

    fn put(&mut self, ctx:RpcContext, req:PutReq, sink:UnarySink<PutReply>) {
        let mut reply = PutReply::new();
        let state = match self.db.put(req.key.as_bytes(), req.value.as_bytes()) {
            Ok(_) => State::OK,
            _ => State::IO_ERROR
        };
        reply.state = state;
        let f = sink
            .success(reply.clone())
            .map_err(move |err| eprintln!("Failed to reply put: {:?}", err));
        ctx.spawn(f)
    }
}