// raft rpc server

extern crate rocksdb;
//extern crate raft;

use super::super::protos::service::{Null,AddressState};
use super::super::protos::service_grpc::RaftService;
use raft::eraftpb::Message;
use grpcio::{RpcContext, UnarySink};
use super::super::raft_config::config;
use std::sync::mpsc::Sender;


#[derive(Clone)]
pub struct RaftServer {
    pub sender:Sender<config::Msg>
}

impl RaftService for RaftServer {
    // send raft message
    fn send_msg(&mut self, _ctx:RpcContext, req:Message, _sink: ::grpcio::UnarySink<Null>) {
        let sender = self.sender.clone();
        sender.send(config::Msg::Raft(req)).unwrap();
    }

    fn send_address(&mut self, _ctx:RpcContext, req:AddressState, _sink:UnarySink<Null>) {
        let sender = self.sender.clone();
        sender.send(config::Msg::Address(req)).unwrap();
    }
}