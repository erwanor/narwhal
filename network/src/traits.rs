// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
use crate::CancelOnDropHandler;
use async_trait::async_trait;
use crypto::NetworkPublicKey;
use rand::prelude::{SliceRandom, SmallRng};
use tokio::task::JoinHandle;

#[async_trait]
pub trait UnreliableNetwork<Request: Clone + Send + Sync> {
    type Response: Clone + Send + Sync;

    async fn unreliable_send(
        &mut self,
        peer: NetworkPublicKey,
        message: &Request,
    ) -> JoinHandle<anyhow::Result<anemo::Response<Self::Response>>>;

    /// Broadcasts a message to all `peers` passed as an argument.
    /// The attempts to send individual messages are best effort and will not be retried.
    async fn unreliable_broadcast(
        &mut self,
        peers: Vec<NetworkPublicKey>,
        message: &Request,
    ) -> Vec<JoinHandle<anyhow::Result<anemo::Response<Self::Response>>>> {
        let mut handlers = Vec::new();
        for peer in peers {
            let handle = { self.unreliable_send(peer, message).await };
            handlers.push(handle);
        }
        handlers
    }
}

pub trait Lucky {
    fn rng(&mut self) -> &mut SmallRng;
}

#[async_trait]
pub trait LuckyNetwork<Request> {
    type Response: Clone + Send + Sync;
    /// Pick a few addresses at random (specified by `nodes`) and try (best-effort) to send the
    /// message only to them. This is useful to pick nodes with whom to sync.
    async fn lucky_broadcast(
        &mut self,
        mut peers: Vec<NetworkPublicKey>,
        message: &Request,
        num_nodes: usize,
    ) -> Vec<JoinHandle<anyhow::Result<anemo::Response<Self::Response>>>>;
}

#[async_trait]
impl<T, M> LuckyNetwork<M> for T
where
    M: Clone + Send + Sync,
    T: UnreliableNetwork<M> + Send,
    T: Lucky,
{
    type Response = T::Response;
    async fn lucky_broadcast(
        &mut self,
        mut peers: Vec<NetworkPublicKey>,
        message: &M,
        nodes: usize,
    ) -> Vec<JoinHandle<anyhow::Result<anemo::Response<Self::Response>>>> {
        peers.shuffle(self.rng());
        peers.truncate(nodes);
        self.unreliable_broadcast(peers, message).await
    }
}

#[async_trait]
pub trait ReliableNetwork<Request: Clone + Send + Sync> {
    type Response: Clone + Send + Sync;

    async fn send(
        &mut self,
        peer: NetworkPublicKey,
        message: &Request,
    ) -> CancelOnDropHandler<anyhow::Result<anemo::Response<Self::Response>>>;

    async fn broadcast(
        &mut self,
        peers: Vec<NetworkPublicKey>,
        message: &Request,
    ) -> Vec<CancelOnDropHandler<anyhow::Result<anemo::Response<Self::Response>>>> {
        let mut handlers = Vec::new();
        for peer in peers {
            let handle = self.send(peer, message).await;
            handlers.push(handle);
        }
        handlers
    }
}
