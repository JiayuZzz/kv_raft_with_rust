// Copyright 2018 PingCAP, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate raft;

use std::collections::HashMap;
use std::sync::mpsc::{RecvTimeoutError, Receiver, Sender};
use std::time::{Duration, Instant};
use super::super::kv::server::Op;
use super::super::protos::service_grpc::RaftServiceClient;
use grpcio::{ChannelBuilder, EnvBuilder};
use std::sync::Arc;
use protobuf::Message as PMessage;

use raft::prelude::*;
use raft::storage::MemStorage;
use bincode::{serialize,deserialize};
use std::thread;

type ProposeCallback = Box<Fn(bool) + Send>;  // if false, not leader

pub enum Msg {
    Propose {
        op: Op,
        cb: ProposeCallback,
    },
    ConfigChange {
        change:ConfChange,
        address:String,
        cb: ProposeCallback
    },
    // Here we don't use Raft Message, so use dead_code to
    // avoid the compiler warning.
    #[allow(dead_code)]
    Raft(Message),
}

pub fn init_and_run(storage:MemStorage, receiver:Receiver<Msg>, apply_sender:Sender<Op>,id:u64, num:u64, addresses:Vec<String>) {
    let mut peers = vec![];
    for i in 1..num+1 {
        peers.push(i);
    }
    // Create the configuration for the Raft node.
    let cfg = Config {
        // The unique ID for the Raft node.
        id,
        // The Raft node list.
        // Mostly, the peers need to be saved in the storage
        peers,
        // Election tick is for how long the follower may campaign again after
        // it doesn't receive any message from the leader.
        election_tick: 10,
        // Heartbeat tick is for how long the leader needs to send
        // a heartbeat to keep alive.
        heartbeat_tick: 3,
        // The max size limits the max size of each appended message. Mostly, 1 MB is enough.
        max_size_per_msg: 1024 * 1024 * 1024,
        // Max inflight msgs that the leader sends messages to follower without
        // receiving ACKs.
        max_inflight_msgs: 256,
        // The Raft applied index.
        // You need to save your applied index when you apply the committed Raft logs.
        applied: 0,
        // Just for log
        tag: format!("[{}]", 1),
        ..Default::default()
    };

    // Create the Raft node.
    let mut r = RawNode::new(&cfg, storage, vec![]).unwrap();

    // Loop forever to drive the Raft.
    let mut t = Instant::now();
    let mut timeout = Duration::from_millis(100);

    // Use a HashMap to hold the `propose` callbacks.
    let mut cbs = HashMap::new();
    let mut rpc_clients = init_clients(addresses);         // for send msg to other raft nodes

    loop {
        match receiver.recv_timeout(timeout) {
            Ok(Msg::Propose { op, cb }) => {
                if r.raft.leader_id != r.raft.id{ // not leader, callback to notify client
                    cb(false);
                    continue;
                }
                let se_op = serialize(&op).unwrap();
                let seq = match op {
                    Op::Get{key:_key,seq} => seq,
                    Op::Put{key:_key, val:_val,seq}=>seq
                };
                cbs.insert(seq, cb);
                r.propose(vec![], se_op).unwrap();
            }
            Ok(Msg::ConfigChange {change,address,cb}) => {
                if r.raft.leader_id != r.raft.id {
                    cb(false);
                    continue;
                } else {
                    r.propose_conf_change(vec![],change).unwrap();
                }
            },
            Ok(Msg::Raft(m)) => {
//                println!("{} got raft msg from {}",r.raft.id,m.from);
                if m.get_msg_type() == MessageType::MsgAppend {
                    println!("node get append");
                }
                r.step(m).unwrap()
            },
            Err(RecvTimeoutError::Timeout) => (),
            Err(RecvTimeoutError::Disconnected) => return (),
        }

        let d = t.elapsed();
        if d >= timeout {
            t = Instant::now();
            timeout = Duration::from_millis(100);
            // We drive Raft every 100ms.
            r.tick();
        } else {
            timeout -= d;
        }

        on_ready(&mut r, &mut cbs,&mut rpc_clients ,apply_sender.clone(), );
    }
}

fn on_ready(r: &mut RawNode<MemStorage>, cbs: &mut HashMap<u64, ProposeCallback>,clients: &mut HashMap<u64,Arc<RaftServiceClient>>, apply_sender:Sender<Op>) {
    if !r.has_ready() {
        return;
    }

    // The Raft is ready, we can do something now.
    let mut ready = r.ready();

    let is_leader = r.raft.leader_id == r.raft.id;
    if is_leader {
        // If the peer is leader, the leader can send messages to other followers ASAP.
        let msgs = ready.messages.drain(..);
        for msg in msgs {
            let client = match clients.get(&msg.get_to()) {
                Some(c) => c.clone(),
                None => {continue;},
            };
            if msg.get_msg_type() == MessageType::MsgAppend {
                println!("append to {}",msg.get_to());
            }
            let me = r.raft.id;
            thread::spawn(move || {
                let msg = msg;
                if msg.get_to() == me {
                    println!("sed to me");
                }
                client.send_msg(&msg);
                if msg.get_to() == me {
                    println!("sed to me done");
                }
            });
        }
    }

    if !raft::is_empty_snap(ready.snapshot()) {
        // This is a snapshot, we need to apply the snapshot at first.
        r.mut_store()
            .wl()
            .apply_snapshot(ready.snapshot().clone())
            .unwrap();
    }

    if !ready.entries().is_empty() {
        // Append entries to the Raft log
        r.mut_store().wl().append(ready.entries()).unwrap();
    }

    if let Some(hs) = ready.hs() {
        // Raft HardState changed, and we need to persist it.
        r.mut_store().wl().set_hardstate(hs.clone());
    }

    if !is_leader {
        // If not leader, the follower needs to reply the messages to
        // the leader after appending Raft entries.
        let msgs = ready.messages.drain(..);
        for msg in msgs {
            // Send messages to other peers.
            let client = match clients.get(&msg.get_to()) {
                Some(c) => c.clone(),
                None => {continue;},
            };
//            println!("send no leader msg to {}",msg.to);
            thread::spawn(move || {
                let msg = msg;
                client.send_msg(&msg);
            });
        }
    }

    if let Some(committed_entries) = ready.committed_entries.take() {
        let mut _last_apply_index = 0;
        for entry in committed_entries {
            println!("handle commited entries");
            // Mostly, you need to save the last apply index to resume applying
            // after restart. Here we just ignore this because we use a Memory storage.
            _last_apply_index = entry.get_index();

            if entry.get_data().is_empty() {
                println!("empty entry");
                // Emtpy entry, when the peer becomes Leader it will send an empty entry.
                continue;
            }

            if entry.get_entry_type() == EntryType::EntryNormal {
                let op:Op = deserialize(entry.get_data()).unwrap();
                let seq = match &op {
                    Op::Get{key:_key,seq} => seq,
                    Op::Put{key:_key, val:_val,seq}=>seq,
                };
                if let Some(cb) = cbs.remove(&seq) {
                    cb(true);
                }
                match apply_sender.send(op) {
                    _ => {}
                }
            }

            // handle EntryConfChange
            if entry.get_entry_type() == EntryType::EntryConfChange {
                let mut change = ConfChange::new();
                change.merge_from_bytes(entry.get_data());
                r.apply_conf_change(&change);
            }
        }
    }

    // Advance the Raft
    r.advance(ready);
}

// init communication clients
fn init_clients(addresses:Vec<String>) -> HashMap<u64,Arc<RaftServiceClient>> {
    let mut clients = HashMap::new();
    for i in 1..addresses.len()+1 {
        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(addresses[i-1].as_str());
        let client = RaftServiceClient::new(ch);
        clients.insert(i as u64,Arc::new(client));
    }
    clients
}

