use crate::networking::connection::{
    TransformMsg, deserialize_transform, serialize_transform, start_p2p,
};
use crate::render::renderer::RendererRenderResources;
use anyhow::Result;
use futures::StreamExt;
use glam::{Quat, Vec3};
use log::debug;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::task::JoinHandle;
use tokio::time::Duration;

pub struct TransformSync {
    renderer: Arc<RwLock<RendererRenderResources>>,
    outbound_tx: futures::channel::mpsc::UnboundedSender<Vec<u8>>,
    inbound_rx: futures::channel::mpsc::UnboundedReceiver<Vec<u8>>,
    _task: tokio::task::JoinHandle<()>,
    last_remote_ts: Option<u128>,
    instance_id: String,
    peers_rx: futures::channel::mpsc::UnboundedReceiver<usize>,
    peers_count: std::sync::Arc<std::sync::atomic::AtomicU8>,
    last_sync_ms: std::sync::Arc<std::sync::atomic::AtomicU64>,
    last_sent_pose: Option<([f32; 3], [f32; 4], [f32; 3])>,
    seq_out: u64,
    last_seq_in_by_peer: HashMap<String, u64>,
}

impl TransformSync {
    pub fn new(
        renderer: Arc<RwLock<RendererRenderResources>>,
        peers_count: std::sync::Arc<std::sync::atomic::AtomicU8>,
        last_sync_ms: std::sync::Arc<std::sync::atomic::AtomicU64>,
    ) -> Result<Self> {
        let (task, outbound_tx, inbound_rx, peer_id, peers_rx) = start_p2p("cube-transform")?;
        Ok(Self {
            renderer,
            outbound_tx,
            inbound_rx,
            _task: task,
            last_remote_ts: None,
            instance_id: peer_id.to_string(),
            peers_rx,
            peers_count,
            last_sync_ms,
            last_sent_pose: None,
            seq_out: 0,
            last_seq_in_by_peer: HashMap::new(),
        })
    }

    pub fn start(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(1000 / 20));
            let mut last_rec_pos = None;
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Avoid borrowing self immutably and mutably at once; clone needed bits
                        let renderer = self.renderer.clone();
                        if last_rec_pos == Some(renderer.read().unwrap().get_models().first().unwrap().get_transform()) {
                            continue;
                        }
                        let outbound = self.outbound_tx.clone();
                        if let Err(err) = self.broadcast_current_transform(&renderer, &outbound).await { eprintln!("broadcast err: {err:?}"); }
                    }
                    Some(bytes) = self.inbound_rx.next() => {
                        let renderer = self.renderer.clone();
                        let self_id = self.instance_id.clone();
                        if let Err(err) = self.apply_remote_transform(&renderer, bytes, &self_id) { eprintln!("apply err: {err:?}"); } else {
                            self.last_sync_ms.store(
                                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64,
                                std::sync::atomic::Ordering::Relaxed,
                            );
                        }
                        last_rec_pos = Some(renderer.read().unwrap().get_models().first().unwrap().get_transform())
                    }
                    Some(peers) = self.peers_rx.next() => {
                        self.peers_count.store(peers as u8, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }
        })
    }

    async fn broadcast_current_transform(
        &mut self,
        renderer: &Arc<RwLock<RendererRenderResources>>,
        outbound_tx: &futures::channel::mpsc::UnboundedSender<Vec<u8>>,
    ) -> Result<()> {
        let transform = {
            let renderer = renderer
                .read()
                .map_err(|_| anyhow::anyhow!("renderer read lock poisoned"))?;
            let Some(first) = renderer.get_models().first() else {
                return Ok(());
            };
            first.get_transform()
        };
        let rotation = transform.get_rotation().to_array(); // [x, y, z, w]
        self.seq_out = self.seq_out.wrapping_add(1);
        let msg = TransformMsg {
            position: transform.get_position().to_array(),
            rotation: [rotation[0], rotation[1], rotation[2], rotation[3]],
            scale: transform.get_scale().to_array(),
            ts_millis: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_millis(0))
                .as_millis(),
            instance_id: self.instance_id.clone(),
            seq: self.seq_out,
        };
        let pose = (msg.position, msg.rotation, msg.scale);
        if let Some(prev) = &self.last_sent_pose
            && !Self::pose_changed(prev, &pose)
        {
            return Ok(());
        }
        self.last_sent_pose = Some(pose);
        let bytes = serialize_transform(&msg)?;
        debug!(
            "broadcasting transform pos=({:.2},{:.2},{:.2}) scale=({:.2},{:.2},{:.2})",
            msg.position[0],
            msg.position[1],
            msg.position[2],
            msg.scale[0],
            msg.scale[1],
            msg.scale[2]
        );
        let _ = outbound_tx.unbounded_send(bytes);
        Ok(())
    }

    fn pose_changed(
        a: &([f32; 3], [f32; 4], [f32; 3]),
        b: &([f32; 3], [f32; 4], [f32; 3]),
    ) -> bool {
        let eps = 1e-3;
        let diff = |x: f32, y: f32| (x - y).abs() > eps;
        for i in 0..3 {
            if diff(a.0[i], b.0[i]) {
                return true;
            }
        }
        for i in 0..4 {
            if diff(a.1[i], b.1[i]) {
                return true;
            }
        }
        for i in 0..3 {
            if diff(a.2[i], b.2[i]) {
                return true;
            }
        }
        false
    }

    fn apply_remote_transform(
        &mut self,
        renderer: &Arc<RwLock<RendererRenderResources>>,
        bytes: Vec<u8>,
        self_id: &str,
    ) -> Result<()> {
        let msg = deserialize_transform(&bytes)?;
        // Ignore own messages first to avoid advancing local sequence watermark
        if msg.instance_id == self_id {
            return Ok(());
        }
        if let Some(prev) = self.last_seq_in_by_peer.get(&msg.instance_id)
            && msg.seq <= *prev
        {
            debug!(
                "ignoring duplicate/out-of-order seq from {} prev={} <= msg={}",
                msg.instance_id, prev, msg.seq
            );
            return Ok(());
        }
        self.last_seq_in_by_peer
            .insert(msg.instance_id.clone(), msg.seq);
        debug!(
            "received transform from {} ts={} pos=({:.2},{:.2},{:.2})",
            msg.instance_id, msg.ts_millis, msg.position[0], msg.position[1], msg.position[2]
        );
        // Apply all remote updates (last-writer-wins) to ensure visible sync; store ts for UI only
        self.last_remote_ts = Some(msg.ts_millis);

        let position = Vec3::from_array(msg.position);
        // msg.rotation is [x,y,z,w]
        let rotation = Quat::from_xyzw(
            msg.rotation[0],
            msg.rotation[1],
            msg.rotation[2],
            msg.rotation[3],
        );
        let scale = Vec3::from_array(msg.scale);

        let mut renderer = renderer
            .write()
            .map_err(|_| anyhow::anyhow!("renderer write lock poisoned"))?;
        if let Some(first) = renderer.get_models_mut().first_mut() {
            let t = first.get_transform_mut();
            t.set_position(position);
            t.set_rotation(rotation);
            t.set_scale(scale);
        }
        Ok(())
    }
}
