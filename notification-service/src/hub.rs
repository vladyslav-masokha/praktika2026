use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, RwLock};

pub type OrderId = i64;
pub type ConnId = String;
pub type ClientTx = mpsc::UnboundedSender<String>;

#[derive(Clone, Default)]
pub struct WsHub {
    inner: Arc<RwLock<HashMap<OrderId, HashMap<ConnId, ClientTx>>>>,
}

impl WsHub {
    pub async fn add_client(&self, order_id: OrderId, conn_id: ConnId, tx: ClientTx) {
        self.inner
            .write()
            .await
            .entry(order_id)
            .or_default()
            .insert(conn_id, tx);
    }

    pub async fn remove_client(&self, order_id: OrderId, conn_id: &str) {
        let mut map = self.inner.write().await;
        if let Some(conns) = map.get_mut(&order_id) {
            conns.remove(conn_id);
            if conns.is_empty() {
                map.remove(&order_id);
            }
        }
    }

    pub async fn notify_order(&self, order_id: OrderId, message: String) {
        let map = self.inner.read().await;
        
        if let Some(conns) = map.get(&order_id) {
            for tx in conns.values() {
                let _ = tx.send(message.clone());
            }
        }
        
        if order_id != 0 {
            if let Some(global_conns) = map.get(&0) {
                for tx in global_conns.values() {
                    let _ = tx.send(message.clone());
                }
            }
        }
    }
}