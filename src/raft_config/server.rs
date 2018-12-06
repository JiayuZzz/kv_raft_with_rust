// raft rpc server

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
use std::sync::mpsc::{Sender,Receiver,self};


#[derive(Clone)]
pub struct RaftServer {
    pub sender:Sender<config::Msg>
}

impl RaftService for RaftServer {
    // send raft message
    fn send_msg(&mut self, ctx:RpcContext, req:Message, sink: ::grpcio::UnarySink<Null>) {
        let sender = self.sender.clone();
        if req.get_msg_type() == MessageType::MsgAppend {
        }
//        println!("get raft msg from {}",req.from);
        sender.send(config::Msg::Raft(req));
    }
}