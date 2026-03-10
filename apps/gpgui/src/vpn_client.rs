use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VpnStatus {
    Disconnected,
    Connecting,
    Connected {
        portal: String,
        gateway: String,
    },
    Disconnecting,
}

impl Default for VpnStatus {
    fn default() -> Self {
        VpnStatus::Disconnected
    }
}

pub struct VpnClient {
    status: VpnStatus,
    ws_url: String,
    sender: Option<mpsc::UnboundedSender<Message>>,
}

impl VpnClient {
    pub fn new() -> Self {
        Self {
            status: VpnStatus::default(),
            ws_url: "ws://localhost:8080/ws".to_string(),
            sender: None,
        }
    }
    
    pub async fn connect(
        &mut self,
        portal_url: &str,
        gateway_address: Option<String>,
        cookie: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.status = VpnStatus::Connecting;
        
        // Connect to WebSocket service
        let url = Url::parse(&self.ws_url)?;
        let (ws_stream, _) = connect_async(url).await?;
        
        let (write, read) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        
        self.sender = Some(tx);
        
        // Spawn reader task
        tokio::spawn(async move {
            // TODO: Handle incoming messages from service
            // This would update the status based on service events
        });
        
        // Send connect request
        let gateway_addr = gateway_address.unwrap_or_else(|| {
            // Extract hostname from portal URL
            if let Ok(url) = Url::parse(portal_url) {
                url.host_str().unwrap_or("localhost").to_string()
            } else {
                "localhost".to_string()
            }
        });
        
        let connect_request = serde_json::json!({
            "type": "Connect",
            "data": {
                "info": {
                    "portal": portal_url,
                    "gateway": {
                        "name": "Default Gateway",
                        "address": gateway_addr
                    },
                    "gateways": [{
                        "name": "Default Gateway",
                        "address": gateway_addr
                    }]
                },
                "cookie": cookie
            }
        });
        
        if let Some(sender) = &self.sender {
            sender.send(Message::Text(connect_request.to_string()))?;
        }
        
        // Update status (in real implementation, this would be updated by service events)
        self.status = VpnStatus::Connected {
            portal: portal_url.to_string(),
            gateway: gateway_addr,
        };
        
        Ok(())
    }
    
    pub async fn disconnect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.status = VpnStatus::Disconnecting;
        
        // Send disconnect request
        let disconnect_request = serde_json::json!({
            "type": "Disconnect",
            "data": {}
        });
        
        if let Some(sender) = &self.sender {
            sender.send(Message::Text(disconnect_request.to_string()))?;
        }
        
        self.status = VpnStatus::Disconnected;
        Ok(())
    }
    
    pub fn get_status(&self) -> &VpnStatus {
        &self.status
    }
}