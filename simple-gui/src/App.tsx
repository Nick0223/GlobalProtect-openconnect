import React, { useState, useEffect, useCallback } from 'react';
import { 
  Box, 
  Button, 
  Typography, 
  LinearProgress, 
  Alert, 
  Card, 
  CardContent,
  List,
  ListItem,
  ListItemText,
  Divider,
  CircularProgress
} from '@mui/material';

// API接口定义
interface VpnState {
  type: 'Disconnected' | 'Connecting' | 'Connected' | 'Disconnecting';
  info?: {
    portal: string;
    gateway: {
      name: string;
      address: string;
    };
    gateways: Array<{
      name: string;
      address: string;
    }>;
  };
}

interface VpnEnv {
  vpnState: VpnState;
  vpncScript?: string;
  csdWrapper?: string;
  authExecutable: string;
}

interface WsEvent {
  type: 'VpnEnv' | 'VpnState' | 'ActiveGui' | 'ResumeConnection';
  data: VpnEnv | VpnState | null;
}

interface ConnectRequest {
  info: {
    portal: string;
    gateway: {
      name: string;
      address: string;
    };
    gateways: Array<{
      name: string;
      address: string;
    }>;
  };
  cookie: string;
}

// WebSocket客户端
class WebSocketClient {
  private ws: WebSocket | null = null;
  private url: string;
  private onMessage: (event: WsEvent) => void;
  private onError: (error: string) => void;

  constructor(url: string, onMessage: (event: WsEvent) => void, onError: (error: string) => void) {
    this.url = url;
    this.onMessage = onMessage;
    this.onError = onError;
  }

  connect() {
    try {
      this.ws = new WebSocket(this.url);
      
      this.ws.onopen = () => {
        console.log('WebSocket connected');
      };

      this.ws.onmessage = (event) => {
        // 这里应该是解密后的消息，简化处理
        try {
          const data = JSON.parse(event.data as string);
          this.onMessage(data);
        } catch (error) {
          console.error('Failed to parse message:', error);
        }
      };

      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        this.onError('WebSocket connection error');
      };

      this.ws.onclose = () => {
        console.log('WebSocket disconnected');
        this.onError('WebSocket disconnected');
      };
    } catch (error) {
      this.onError('Failed to connect to WebSocket');
    }
  }

  sendConnectRequest(request: ConnectRequest) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      // 这里应该是加密后的消息，简化处理
      const message = {
        type: 'Connect',
        data: request
      };
      this.ws.send(JSON.stringify(message));
    }
  }

  sendDisconnectRequest() {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      const message = {
        type: 'Disconnect',
        data: {}
      };
      this.ws.send(JSON.stringify(message));
    }
  }

  close() {
    if (this.ws) {
      this.ws.close();
    }
  }
}

const SimpleGPGUI: React.FC = () => {
  const [vpnState, setVpnState] = useState<VpnState>({ type: 'Disconnected' });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [wsClient, setWsClient] = useState<WebSocketClient | null>(null);
  const [connected, setConnected] = useState(false);

  // 初始化WebSocket连接
  useEffect(() => {
    const wsUrl = 'ws://localhost:8080/ws'; // 假设服务运行在8080端口
    const client = new WebSocketClient(
      wsUrl,
      (event: WsEvent) => {
        if (event.type === 'VpnState') {
          setVpnState(event.data as VpnState);
          setConnected((event.data as VpnState).type === 'Connected');
        } else if (event.type === 'VpnEnv') {
          const env = event.data as VpnEnv;
          setVpnState(env.vpnState);
          setConnected(env.vpnState.type === 'Connected');
        }
      },
      (errorMessage: string) => {
        setError(errorMessage);
      }
    );

    client.connect();
    setWsClient(client);

    return () => {
      client.close();
    };
  }, []);

  const handleConnect = useCallback(async () => {
    if (!wsClient) return;
    
    setLoading(true);
    setError(null);
    
    try {
      // 这里应该从认证流程获取cookie，简化处理使用假数据
      const mockConnectRequest: ConnectRequest = {
        info: {
          portal: 'https://vpn.example.com',
          gateway: {
            name: 'Main Gateway',
            address: 'gateway.example.com'
          },
          gateways: [
            {
              name: 'Main Gateway',
              address: 'gateway.example.com'
            }
          ]
        },
        cookie: 'mock-cookie-data'
      };
      
      wsClient.sendConnectRequest(mockConnectRequest);
    } catch (err) {
      setError('Failed to connect VPN');
      setLoading(false);
    }
  }, [wsClient]);

  const handleDisconnect = useCallback(() => {
    if (!wsClient) return;
    
    setLoading(true);
    wsClient.sendDisconnectRequest();
    // 状态会通过WebSocket事件自动更新
  }, [wsClient]);

  const renderStatus = () => {
    switch (vpnState.type) {
      case 'Disconnected':
        return <Typography color="error">Disconnected</Typography>;
      case 'Connecting':
        return <Typography color="warning">Connecting...</Typography>;
      case 'Connected':
        return <Typography color="success">Connected</Typography>;
      case 'Disconnecting':
        return <Typography color="warning">Disconnecting...</Typography>;
      default:
        return <Typography>Unknown</Typography>;
    }
  };

  const renderGatewayInfo = () => {
    if (vpnState.type !== 'Connected' && vpnState.type !== 'Connecting') {
      return null;
    }

    const info = vpnState.info;
    if (!info) return null;

    return (
      <Box mt={2}>
        <Typography variant="subtitle2">Connection Details:</Typography>
        <List dense>
          <ListItem>
            <ListItemText primary="Portal" secondary={info.portal} />
          </ListItem>
          <ListItem>
            <ListItemText primary="Gateway" secondary={`${info.gateway.name} (${info.gateway.address})`} />
          </ListItem>
        </List>
      </Box>
    );
  };

  return (
    <Box sx={{ p: 3, maxWidth: 600, mx: 'auto' }}>
      <Card>
        <CardContent>
          <Typography variant="h4" component="h1" gutterBottom align="center">
            GlobalProtect VPN Client
          </Typography>
          
          <Divider sx={{ my: 2 }} />
          
          {/* Status Section */}
          <Box display="flex" alignItems="center" justifyContent="space-between" mb={3}>
            <Typography variant="h6">Status:</Typography>
            {renderStatus()}
          </Box>

          {/* Progress Indicator */}
          {(vpnState.type === 'Connecting' || vpnState.type === 'Disconnecting' || loading) && (
            <Box mb={2}>
              <LinearProgress />
            </Box>
          )}

          {/* Action Buttons */}
          <Box display="flex" gap={2} justifyContent="center" mb={3}>
            {!connected ? (
              <Button
                variant="contained"
                color="primary"
                onClick={handleConnect}
                disabled={loading || vpnState.type === 'Connecting'}
                startIcon={loading && !connected ? <CircularProgress size={20} /> : null}
              >
                {loading && !connected ? 'Connecting...' : 'Connect'}
              </Button>
            ) : (
              <Button
                variant="contained"
                color="secondary"
                onClick={handleDisconnect}
                disabled={loading || vpnState.type === 'Disconnecting'}
                startIcon={loading && connected ? <CircularProgress size={20} /> : null}
              >
                {loading && connected ? 'Disconnecting...' : 'Disconnect'}
              </Button>
            )}
          </Box>

          {/* Gateway Info */}
          {renderGatewayInfo()}

          {/* Error Alert */}
          {error && (
            <Alert severity="error" onClose={() => setError(null)} sx={{ mt: 2 }}>
              {error}
            </Alert>
          )}

          {/* Connection Tips */}
          <Box mt={3} p={2} bgcolor="#f5f5f5" borderRadius={1}>
            <Typography variant="body2" color="text.secondary">
              <strong>Tip:</strong> This is a simplified GUI interface for GlobalProtect VPN.
              In a real implementation, you would need to handle authentication flows,
              certificate management, and proper encryption/decryption of WebSocket messages.
            </Typography>
          </Box>
        </CardContent>
      </Card>
    </Box>
  );
};

export default SimpleGPGUI;