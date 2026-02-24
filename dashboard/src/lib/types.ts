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
  totalBytesSent: number;
  totalBytesReceived: number;
  trafficQuotaGb: number | null;
  remainingQuotaGb: number | null;
  trafficResetCycle: string;
  lastResetAt: string | null;
  isTrafficExceeded: boolean;
  maxPortCount: number | null;
  allowedPortRange: string | null;
  currentPortCount?: number;
}

export interface UserWithNodeCount extends User {
  node_count: number;
}

// 用户配额信息
export interface UserQuotaInfo {
  user_id: number;
  username: string;
  total_quota_gb: number | null;
  used_gb: number;
  allocated_to_clients_gb: number;
  available_gb: number;
  quota_usage_percent: number | null;
}

// 客户端类型
export interface Client {
  id: number;
  name: string;
  token: string;
  is_online: boolean;
  userId: number | null;
  totalBytesSent: number;
  totalBytesReceived: number;
  trafficQuotaGb: number | null;
  trafficResetCycle: string;
  lastResetAt: string | null;
  isTrafficExceeded: boolean;
  created_at: string;
  updated_at: string;
}

// 客户端流量详情
export interface ClientTrafficInfo {
  client_id: number;
  client_name: string;
  total_bytes_sent: number;
  total_bytes_received: number;
  total_bytes: number;
  quota_gb: number | null;
  remaining_quota_gb: number | null;
  quota_usage_percent: number | null;
  is_traffic_exceeded: boolean;
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
  nodeId: number | null;
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
  publicIp: string | null;
  description: string | null;
  tunnelAddr: string;
  tunnelPort: number;
  tunnelProtocol: string;
  kcpConfig: string | null;
  nodeType: string;
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

// 订阅套餐类型
export interface Subscription {
  id: number;
  name: string;
  durationType: string; // daily, weekly, monthly, yearly
  durationValue: number;
  trafficQuotaGb: number;
  price: number | null;
  description: string | null;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

// 用户订阅类型
export interface UserSubscription {
  id: number;
  userId: number;
  subscriptionId: number;
  subscriptionName: string;
  startDate: string;
  endDate: string;
  trafficQuotaGb: number;
  trafficUsedGb: number;
  trafficRemainingGb: number;
  isActive: boolean;
  isExpired: boolean;
  createdAt: string;
  updatedAt: string;
}
