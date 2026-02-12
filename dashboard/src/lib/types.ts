// API 响应类型
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  message: string;
}

// 用户类型
export interface User {
  id: number;
  username: string;
  is_admin: boolean;
  created_at: string;
  updated_at: string;
  total_bytes_sent: number;
  total_bytes_received: number;
  uploadLimitGb: number | null;
  downloadLimitGb: number | null;
  trafficResetCycle: string;
  lastResetAt: string | null;
  isTrafficExceeded: boolean;
}

export interface UserWithClientCount extends User {
  client_count: number;
}

// 客户端类型
export interface Client {
  id: number;
  name: string;
  token: string;
  is_online: boolean;
  nodeId: number | null;
  node_name?: string;
  total_bytes_sent: number;
  total_bytes_received: number;
  uploadLimitGb: number | null;
  downloadLimitGb: number | null;
  trafficResetCycle: string;
  lastResetAt: string | null;
  isTrafficExceeded: boolean;
  created_at: string;
  updated_at: string;
}

// 代理类型
export interface Proxy {
  id: number;
  client_id: string;
  name: string;
  type: string;  // 后端返回的是 "type" 不是 "proxy_type"
  localIP: string;  // 后端返回驼峰命名
  localPort: number;  // 后端返回驼峰命名
  remotePort: number;  // 后端返回驼峰命名
  enabled: boolean;
  totalBytesSent: number;  // 后端返回驼峰命名
  totalBytesReceived: number;  // 后端返回驼峰命名
  created_at: string;
  updated_at: string;
}

// 流量类型
export interface TrafficOverview {
  total_traffic: TotalTraffic;
  by_user: UserTraffic[];
  by_client: ClientTraffic[];
  by_proxy: ProxyTraffic[];
  daily_traffic: DailyTraffic[];
}

export interface TotalTraffic {
  total_bytes_sent: number;
  total_bytes_received: number;
  total_bytes: number;
}

export interface UserTraffic {
  user_id: number;
  username: string;
  total_bytes_sent: number;
  total_bytes_received: number;
  total_bytes: number;
}

export interface ClientTraffic {
  client_id: number;
  client_name: string;
  total_bytes_sent: number;
  total_bytes_received: number;
  total_bytes: number;
}

export interface ProxyTraffic {
  proxy_id: number;
  proxy_name: string;
  client_id: number;
  client_name: string;
  total_bytes_sent: number;
  total_bytes_received: number;
  total_bytes: number;
}

export interface DailyTraffic {
  date: string;
  total_bytes_sent: number;
  total_bytes_received: number;
  total_bytes: number;
}

// Dashboard 统计
export interface DashboardStats {
  total_clients: number;
  total_proxies: number;
  online_clients: number;
  enabled_proxies: number;
  total_nodes: number;
  online_nodes: number;
  user_traffic: {
    total_bytes_sent: number;
    total_bytes_received: number;
    total_bytes: number;
  };
}

// 节点类型
export interface Node {
  id: number;
  name: string;
  url: string;
  secret: string;
  isOnline: boolean;
  region: string | null;
  description: string | null;
  tunnelAddr: string;
  tunnelPort: number;
  tunnelProtocol: string;
  kcpConfig: string | null;
  created_at: string;
  updated_at: string;
}

// 登录请求
export interface LoginRequest {
  username: string;
  password: string;
}

// 登录响应
export interface LoginResponse {
  token: string;
  user: {
    id: number;
    username: string;
    is_admin: boolean;
  };
}

// 日志条目
export interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
}
