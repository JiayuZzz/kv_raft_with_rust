// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_KV_SERVICE_GET: ::grpcio::Method<super::service::GetReq, super::service::GetReply> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/KvService/Get",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_KV_SERVICE_PUT: ::grpcio::Method<super::service::PutReq, super::service::PutReply> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/KvService/Put",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct KvServiceClient {
    client: ::grpcio::Client,
}

impl KvServiceClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        KvServiceClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn get_opt(&self, req: &super::service::GetReq, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::service::GetReply> {
        self.client.unary_call(&METHOD_KV_SERVICE_GET, req, opt)
    }

    pub fn get(&self, req: &super::service::GetReq) -> ::grpcio::Result<super::service::GetReply> {
        self.get_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_async_opt(&self, req: &super::service::GetReq, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::service::GetReply>> {
        self.client.unary_call_async(&METHOD_KV_SERVICE_GET, req, opt)
    }

    pub fn get_async(&self, req: &super::service::GetReq) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::service::GetReply>> {
        self.get_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn put_opt(&self, req: &super::service::PutReq, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::service::PutReply> {
        self.client.unary_call(&METHOD_KV_SERVICE_PUT, req, opt)
    }

    pub fn put(&self, req: &super::service::PutReq) -> ::grpcio::Result<super::service::PutReply> {
        self.put_opt(req, ::grpcio::CallOption::default())
    }

    pub fn put_async_opt(&self, req: &super::service::PutReq, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::service::PutReply>> {
        self.client.unary_call_async(&METHOD_KV_SERVICE_PUT, req, opt)
    }

    pub fn put_async(&self, req: &super::service::PutReq) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::service::PutReply>> {
        self.put_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait KvService {
    fn get(&mut self, ctx: ::grpcio::RpcContext, req: super::service::GetReq, sink: ::grpcio::UnarySink<super::service::GetReply>);
    fn put(&mut self, ctx: ::grpcio::RpcContext, req: super::service::PutReq, sink: ::grpcio::UnarySink<super::service::PutReply>);
}

pub fn create_kv_service<S: KvService + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_KV_SERVICE_GET, move |ctx, req, resp| {
        instance.get(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_KV_SERVICE_PUT, move |ctx, req, resp| {
        instance.put(ctx, req, resp)
    });
    builder.build()
}

const METHOD_RAFT_SERVICE_SEND_MSG: ::grpcio::Method<super::eraftpb::Message, super::service::Null> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/RaftService/SendMsg",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct RaftServiceClient {
    client: ::grpcio::Client,
}

impl RaftServiceClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        RaftServiceClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn send_msg_opt(&self, req: &super::eraftpb::Message, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::service::Null> {
        self.client.unary_call(&METHOD_RAFT_SERVICE_SEND_MSG, req, opt)
    }

    pub fn send_msg(&self, req: &super::eraftpb::Message) -> ::grpcio::Result<super::service::Null> {
        self.send_msg_opt(req, ::grpcio::CallOption::default())
    }

    pub fn send_msg_async_opt(&self, req: &super::eraftpb::Message, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::service::Null>> {
        self.client.unary_call_async(&METHOD_RAFT_SERVICE_SEND_MSG, req, opt)
    }

    pub fn send_msg_async(&self, req: &super::eraftpb::Message) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::service::Null>> {
        self.send_msg_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait RaftService {
    fn send_msg(&mut self, ctx: ::grpcio::RpcContext, req: super::eraftpb::Message, sink: ::grpcio::UnarySink<super::service::Null>);
}

pub fn create_raft_service<S: RaftService + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_RAFT_SERVICE_SEND_MSG, move |ctx, req, resp| {
        instance.send_msg(ctx, req, resp)
    });
    builder.build()
}
