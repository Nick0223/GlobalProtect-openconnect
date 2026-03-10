import React, { useState, useEffect, useCallback, useRef } from 'react';
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
  CircularProgress,
  TextField,
  FormControl,
  InputLabel,
  Select,
  MenuItem
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
  private reconnectAttempts: number = 0;
  private maxReconnectAttempts: number = 5;
  private reconnectTimeout: NodeJS.Timeout | null = null;

  constructor(url: string, onMessage: (event: WsEvent) => void, onError: (error: string) => void) {
    this.url = url;
    this.onMessage = onMessage;
    this.onError = onError;
  }

  connect() {
    try {
      // 从环境变量获取服务URL，如果没有则使用默认值
      const serviceUrl = process.env.REACT_APP_SERVICE_URL || 'ws://localhost:8080/ws';
      this.ws = new WebSocket(serviceUrl);
      
      this.ws.onopen = () => {
        console.log('WebSocket connected');
        this.reconnectAttempts = 0;
      };

      this.ws.onmessage = (event) => {
        try {
          // 在实际实现中，这里应该是解密后的消息
          // 由于simple-gui主要用于演示，我们假设消息已经是JSON格式
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
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
          this.reconnectAttempts++;
          this.reconnectTimeout = setTimeout(() => {
            this.connect();
          }, 1000 * this.reconnectAttempts);
        } else {
          this.onError('WebSocket disconnected - max reconnect attempts reached');
        }
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
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
    }
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
  const [portalUrl, setPortalUrl] = useState('');
  const [gateways, setGateways] = useState<Array<{name: string, address: string}>>([]);
  const [selectedGateway, setSelectedGateway] = useState<string>('');

  const wsClientRef = useRef<WebSocketClient | null>(null);

  // 初始化WebSocket连接
  useEffect(() => {
    const client = new WebSocketClient(
      '',
      (event: WsEvent) => {
        if (event.type === 'VpnState') {
          const state = event.data as VpnState;
          setVpnState(state);
          setConnected(state.type === 'Connected');
          
          // 更新网关列表
          if (state.info?.gateways) {
            setGateways(state.info.gateways);
            if (state.info.gateways.length > 0) {
              setSelectedGateway(state.info.gateways[0].address);
            }
          }
        } else if (event.type === 'VpnEnv') {
          const env = event.data as VpnEnv;
          setVpnState(env.vpnState);
          setConnected(env.vpnState.type === 'Connected');
          
          // 更新网关列表
          if (env.vpnState.info?.gateways) {
            setGateways(env.vpnState.info.gateways);
            if (env.vpnState.info.gateways.length > 0) {
              setSelectedGateway(env.vpnState.info.gateways[0].address);
            }
          }
        }
      },
      (errorMessage: string) => {
        setError(errorMessage);
      }
    );

    client.connect();
    wsClientRef.current = client;
    setWsClient(client);

    return () => {
      client.close();
    };
  }, []);

  const handleConnect = useCallback(async () => {
    if (!wsClientRef.current || !portalUrl.trim()) return;
    
    setLoading(true);
    setError(null);
    
    try {
      // 验证门户URL格式
      let validPortalUrl = portalUrl.trim();
      if (!validPortalUrl.startsWith('http')) {
        validPortalUrl = 'https://' + validPortalUrl;
      }
      
      // 尝试解析URL
      new URL(validPortalUrl);
      
      // 获取认证cookie - 这里应该调用gpauth工具
      // 由于simple-gui是演示用途，我们模拟这个过程
      const mockCookie = await getAuthCookie(validPortalUrl);
      
      if (!mockCookie) {
        throw new Error('Authentication failed');
      }
      
      // 构建网关列表
      const gatewayList = gateways.length > 0 
        ? gateways 
        : [{
            name: 'Default Gateway',
            address: new URL(validPortalUrl).hostname
          }];
      
      const selectedGatewayInfo = gatewayList.find(gw => gw.address === selectedGateway) || gatewayList[0];
      
      const connectRequest: ConnectRequest = {
        info: {
          portal: validPortalUrl,
          gateway: selectedGatewayInfo,
          gateways: gatewayList
        },
        cookie: mockCookie
      };
      
      wsClientRef.current.sendConnectRequest(connectRequest);
    } catch (err: any) {
      setError(err.message || 'Failed to connect VPN');
      setLoading(false);
    }
  }, [portalUrl, selectedGateway, gateways]);

  const handleDisconnect = useCallback(() => {
    if (!wsClientRef.current) return;
    
    setLoading(true);
    wsClientRef.current.sendDisconnectRequest();
    // 状态会通过WebSocket事件自动更新
  }, []);

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
            GlobalProtect VPN Client (Simple GUI)
          </Typography>
          
          <Divider sx={{ my: 2 }} />
          
          {/* Portal URL Input */}
          {!isConnected && (
            <Box mb={3}>
              <TextField
                fullWidth
                label="Portal URL"
                placeholder="https://vpn.example.com"
                value={portalUrl}
                onChange={(e) => setPortalUrl(e.target.value)}
                disabled={isConnecting}
                error={!portalUrl.trim() && loading}
                helperText={!portalUrl.trim() && loading ? "Portal URL is required" : ""}
              />
            </Box>
          )}

          {/* Gateway Selection */}
          {!isConnected && gateways.length > 1 && (
            <Box mb={3}>
              <FormControl fullWidth>
                <InputLabel>Gateway</InputLabel>
                <Select
                  value={selectedGateway}
                  label="Gateway"
                  onChange={(e) => setSelectedGateway(e.target.value as string)}
                  disabled={isConnecting}
                >
                  {gateways.map((gateway) => (
                    <MenuItem key={gateway.address} value={gateway.address}>
                      {gateway.name} ({gateway.address})
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
            </Box>
          )}

          {/* Status Section */}
          <Box display="flex" alignItems="center" justifyContent="space-between" mb={3}>
            <Typography variant="h6">Status:</Typography>
            {renderStatus()}
          </Box>

          {/* Progress Indicator */}
          {(isConnecting || vpnState.type === 'Disconnecting') && (
            <Box mb={2}>
              <LinearProgress />
            </Box>
          )}

          {/* Action Buttons */}
          <Box display="flex" gap={2} justifyContent="center" mb={3}>
            {!isConnected ? (
              <Button
                variant="contained"
                color="primary"
                onClick={handleConnect}
                disabled={isConnecting || !portalUrl.trim()}
                startIcon={isConnecting ? <CircularProgress size={20} /> : null}
              >
                {isConnecting ? 'Connecting...' : 'Connect'}
              </Button>
            ) : (
              <Button
                variant="contained"
                color="secondary"
                onClick={handleDisconnect}
                disabled={vpnState.type === 'Disconnecting'}
                startIcon={vpnState.type === 'Disconnecting' ? <CircularProgress size={20} /> : null}
              >
                {vpnState.type === 'Disconnecting' ? 'Disconnecting...' : 'Disconnect'}
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
              <strong>Note:</strong> This is a simplified GUI for development and testing purposes.
              For production use, please use the official gpgui application or the CLI client (gpclient).
              The actual implementation requires proper authentication integration with gpauth and encrypted WebSocket communication.
            </Typography>
          </Box>
        </CardContent>
      </Card>
    </Box>
  );
};

export default SimpleGPGUI;