import axios from 'axios';

const API_BASE = '/api';

// Token management
const TOKEN_KEY = 'auth_token';

export const getToken = (): string | null => {
  return localStorage.getItem(TOKEN_KEY);
};

export const setToken = (token: string): void => {
  localStorage.setItem(TOKEN_KEY, token);
};

export const removeToken = (): void => {
  localStorage.removeItem(TOKEN_KEY);
};

// Create axios instance
const api = axios.create({
  baseURL: API_BASE,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Request interceptor - add token to headers
api.interceptors.request.use(
  (config) => {
    const token = getToken();
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// Response interceptor - handle 401 errors
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      removeToken();
      // 不跳转，让 AuthContext 自然处理未登录状态
    }
    return Promise.reject(error);
  }
);

// Types
export interface Client {
  id: string;
  name: string;
  token: string;
  is_online: boolean;
  created_at: string;
  updated_at: string;
}

export interface Proxy {
  id: string;
  client_id: string;
  name: string;
  type: string;
  localIP: string;
  localPort: number;
  remotePort: number;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface User {
  id: number;
  username: string;
  is_admin: boolean;
  created_at: string;
  updated_at: string;
  client_count?: number;
}

export interface UserWithPassword extends User {
  generated_password?: string;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  message: string;
}

// Auth APIs
export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  token: string;
  user: User;
}

export const authApi = {
  login: async (data: LoginRequest): Promise<LoginResponse> => {
    const response = await api.post<ApiResponse<LoginResponse>>('/auth/login', data);
    if (response.data.success && response.data.data) {
      setToken(response.data.data.token);
      return response.data.data;
    }
    throw new Error(response.data.message);
  },

  logout: (): void => {
    removeToken();
  },

  getCurrentUser: async (): Promise<User> => {
    const response = await api.get<ApiResponse<User>>('/auth/me');
    if (response.data.success && response.data.data) {
      return response.data.data;
    }
    throw new Error(response.data.message);
  },
};

// Client API
export const clientApi = {
  list: async (): Promise<Client[]> => {
    const response = await api.get<ApiResponse<Client[]>>('/clients');
    if (response.data.success && response.data.data) {
      return response.data.data;
    }
    throw new Error(response.data.message);
  },

  create: async (data: { name: string; token?: string }): Promise<Client> => {
    const response = await api.post<ApiResponse<Client>>('/clients', data);
    if (response.data.success && response.data.data) {
      return response.data.data;
    }
    throw new Error(response.data.message);
  },

  delete: async (id: string): Promise<void> => {
    const response = await api.delete<ApiResponse<string>>(`/clients/${id}`);
    if (!response.data.success) {
      throw new Error(response.data.message);
    }
  },
};

// Proxy API
export const proxyApi = {
  list: async (): Promise<Proxy[]> => {
    const response = await api.get<ApiResponse<Proxy[]>>('/proxies');
    if (response.data.success && response.data.data) {
      return response.data.data;
    }
    throw new Error(response.data.message);
  },

  listByClient: async (clientId: string): Promise<Proxy[]> => {
    const response = await api.get<ApiResponse<Proxy[]>>(`/clients/${clientId}/proxies`);
    if (response.data.success && response.data.data) {
      return response.data.data;
    }
    throw new Error(response.data.message);
  },

  create: async (data: {
    client_id: string;
    name: string;
    type: string;
    localIP: string;
    localPort: number;
    remotePort: number;
  }): Promise<Proxy> => {
    const response = await api.post<ApiResponse<Proxy>>('/proxies', data);
    if (response.data.success && response.data.data) {
      return response.data.data;
    }
    throw new Error(response.data.message);
  },

  update: async (id: string, data: Partial<Proxy>): Promise<Proxy> => {
    const response = await api.put<ApiResponse<Proxy>>(`/proxies/${id}`, data);
    if (response.data.success && response.data.data) {
      return response.data.data;
    }
    throw new Error(response.data.message);
  },

  delete: async (id: string): Promise<void> => {
    const response = await api.delete<ApiResponse<string>>(`/proxies/${id}`);
    if (!response.data.success) {
      throw new Error(response.data.message);
    }
  },
};

// User API (Admin only)
export const userApi = {
  list: async (): Promise<User[]> => {
    const response = await api.get<ApiResponse<User[]>>('/users');
    if (response.data.success && response.data.data) {
      return response.data.data;
    }
    throw new Error(response.data.message);
  },

  create: async (data: {
    username: string;
    password?: string;
    is_admin?: boolean;
  }): Promise<UserWithPassword> => {
    const response = await api.post<ApiResponse<UserWithPassword>>('/users', data);
    if (response.data.success && response.data.data) {
      return response.data.data;
    }
    throw new Error(response.data.message);
  },

  update: async (
    id: number,
    data: {
      username?: string;
      password?: string;
      is_admin?: boolean;
    }
  ): Promise<User> => {
    const response = await api.put<ApiResponse<User>>(`/users/${id}`, data);
    if (response.data.success && response.data.data) {
      return response.data.data;
    }
    throw new Error(response.data.message);
  },

  delete: async (id: number): Promise<void> => {
    const response = await api.delete<ApiResponse<string>>(`/users/${id}`);
    if (!response.data.success) {
      throw new Error(response.data.message);
    }
  },

  getClients: async (userId: number): Promise<Client[]> => {
    const response = await api.get<ApiResponse<Client[]>>(`/users/${userId}/clients`);
    if (response.data.success && response.data.data) {
      return response.data.data;
    }
    throw new Error(response.data.message);
  },

  assignClient: async (userId: number, clientId: string): Promise<void> => {
    const response = await api.post<ApiResponse<string>>(`/users/${userId}/clients/${clientId}`);
    if (!response.data.success) {
      throw new Error(response.data.message);
    }
  },

  removeClient: async (userId: number, clientId: string): Promise<void> => {
    const response = await api.delete<ApiResponse<string>>(`/users/${userId}/clients/${clientId}`);
    if (!response.data.success) {
      throw new Error(response.data.message);
    }
  },
};

// Traffic Statistics Types
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

export interface TrafficOverview {
  total_traffic: TotalTraffic;
  by_user: UserTraffic[];
  by_client: ClientTraffic[];
  by_proxy: ProxyTraffic[];
  daily_traffic: DailyTraffic[];
}

// Traffic API
export const trafficApi = {
  getOverview: async (days: number = 30): Promise<TrafficOverview> => {
    const response = await api.get<ApiResponse<TrafficOverview>>('/traffic/overview', {
      params: { days }
    });
    if (response.data.success && response.data.data) {
      return response.data.data;
    }
    throw new Error(response.data.message);
  },

  getUserTraffic: async (userId: number, days: number = 30): Promise<TrafficOverview> => {
    const response = await api.get<ApiResponse<TrafficOverview>>(`/traffic/users/${userId}`, {
      params: { days }
    });
    if (response.data.success && response.data.data) {
      return response.data.data;
    }
    throw new Error(response.data.message);
  },
};

// Format bytes to human readable
export const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
};

// Dashboard Statistics Types
export interface UserTrafficStats {
  total_bytes_sent: number;
  total_bytes_received: number;
  total_bytes: number;
}

export interface DashboardStats {
  total_clients: number;
  total_proxies: number;
  online_clients: number;
  enabled_proxies: number;
  user_traffic: UserTrafficStats;
}

// Dashboard API
export const dashboardApi = {
  getStats: async (userId: number): Promise<DashboardStats> => {
    const response = await api.get<ApiResponse<DashboardStats>>(`/dashboard/stats/${userId}`);
    if (response.data.success && response.data.data) {
      return response.data.data;
    }
    throw new Error(response.data.message);
  },
};
