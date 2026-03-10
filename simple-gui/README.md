# GlobalProtect Simple GUI

这是一个基于现有GlobalProtect-openconnect项目API接口的简化版React GUI界面。

## 功能特点

- 连接/断开VPN
- 实时显示VPN连接状态
- 显示连接详情（门户、网关信息）
- WebSocket实时通信
- 响应式设计

## 技术栈

- React 18
- TypeScript
- Material UI (MUI)
- Vite
- WebSocket API

## API接口说明

### WebSocket消息格式

**客户端发送的消息:**
```typescript
// 连接请求
{
  type: 'Connect',
  data: {
    info: {
      portal: string,
      gateway: { name: string, address: string },
      gateways: Array<{ name: string, address: string }>
    },
    cookie: string
  }
}

// 断开请求  
{
  type: 'Disconnect',
  data: {}
}
```

**服务端发送的消息:**
```typescript
// VPN状态更新
{
  type: 'VpnState',
  data: {
    type: 'Disconnected' | 'Connecting' | 'Connected' | 'Disconnecting',
    info?: { /* 连接信息 */ }
  }
}

// VPN环境配置
{
  type: 'VpnEnv',
  data: {
    vpnState: VpnState,
    vpncScript?: string,
    csdWrapper?: string,
    authExecutable: string
  }
}
```

## 使用方法

1. **确保GPS服务正在运行**
   ```bash
   # 在项目根目录启动GPS服务
   cd d:\GlobalProtect-openconnect
   cargo run --bin gpservice
   ```

2. **安装依赖并启动GUI**
   ```bash
   cd simple-gui
   npm install
   npm run dev
   ```

3. **访问应用**
   - 应用将在 http://localhost:3000 自动打开
   - WebSocket连接将自动建立到 http://localhost:8080/ws

## 注意事项

- 当前实现使用了简化的认证流程（mock数据）
- 实际部署时需要实现完整的加密/解密逻辑
- 需要处理证书管理、FIDO2认证等高级功能
- 生产环境应该使用HTTPS和安全的WebSocket连接

## 扩展建议

1. **添加认证流程**: 集成浏览器认证或Webview认证
2. **多门户支持**: 允许用户配置和切换多个VPN门户
3. **系统托盘集成**: 添加系统托盘图标进行快速控制
4. **自动连接**: 支持开机自动连接和网络恢复后重连
5. **日志查看**: 添加日志查看和导出功能
6. **设置页面**: 允许配置高级选项（MTU、IPv6、DTLS等）

## 文件结构

```
simple-gui/
├── src/
│   ├── App.tsx          # 主组件
│   └── main.tsx         # 入口文件
├── index.html           # HTML模板
├── package.json         # 依赖配置
├── tsconfig.json        # TypeScript配置
├── vite.config.ts       # Vite配置
└── README.md            # 说明文档
```