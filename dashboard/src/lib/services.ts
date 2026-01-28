import api from './api';
import type {
  ApiResponse,
  UserWithClientCount,
  Client,
  Proxy,
  TrafficOverview,
  DashboardStats,
  LoginRequest,
  LoginResponse,
  LogEntry,
} from './types';

// ============ 认证服务 ============
export const authService = {
  async login(data: LoginRequest): Promise<ApiResponse<LoginResponse>> {
    const response = await api.post<ApiResponse<LoginResponse>>('/auth/login', data);
    return response.data;
  },
};

// ============ 用户服务 ============
export const userService = {
  async getUsers(): Promise<ApiResponse<UserWithClientCount[]>> {
    const response = await api.get<ApiResponse<UserWithClientCount[]>>('/users');
    return response.data;
  },

  async createUser(data: {
    username: string;
    password?: string;
    is_admin?: boolean;
  }): Promise<ApiResponse<any>> {
    const response = await api.post<ApiResponse<any>>('/users', data);
    return response.data;
  },

  async updateUser(
    id: number,
    data: {
      username?: string;
      password?: string;
      is_admin?: boolean;
    }
  ): Promise<ApiResponse<any>> {
    const response = await api.put<ApiResponse<any>>(`/users/${id}`, data);
    return response.data;
  },

  async deleteUser(id: number): Promise<ApiResponse<string>> {
    const response = await api.delete<ApiResponse<string>>(`/users/${id}`);
    return response.data;
  },

  async getUserClients(id: number): Promise<ApiResponse<Client[]>> {
    const response = await api.get<ApiResponse<Client[]>>(`/users/${id}/clients`);
    return response.data;
  },

  async assignClient(userId: number, clientId: number): Promise<ApiResponse<string>> {
    const response = await api.post<ApiResponse<string>>(`/users/${userId}/clients/${clientId}`);
    return response.data;
  },

  async removeClient(userId: number, clientId: number): Promise<ApiResponse<string>> {
    const response = await api.delete<ApiResponse<string>>(`/users/${userId}/clients/${clientId}`);
    return response.data;
  },
};

// ============ 客户端服务 ============
export const clientService = {
  async getClients(): Promise<ApiResponse<Client[]>> {
    const response = await api.get<ApiResponse<Client[]>>('/clients');
    return response.data;
  },

  async getClient(id: number): Promise<ApiResponse<Client>> {
    const response = await api.get<ApiResponse<Client>>(`/clients/${id}`);
    return response.data;
  },

  async createClient(data: { name: string; token?: string }): Promise<ApiResponse<Client>> {
    const response = await api.post<ApiResponse<Client>>('/clients', data);
    return response.data;
  },

  async deleteClient(id: number): Promise<ApiResponse<string>> {
    const response = await api.delete<ApiResponse<string>>(`/clients/${id}`);
    return response.data;
  },

  async getClientLogs(id: number): Promise<ApiResponse<LogEntry[]>> {
    const response = await api.get<ApiResponse<LogEntry[]>>(`/clients/${id}/logs`);
    return response.data;
  },

  async updateClient(
    id: number,
    data: {
      name?: string;
      upload_limit_gb?: number | null;
      download_limit_gb?: number | null;
      traffic_reset_cycle?: string;
      is_traffic_exceeded?: boolean;
    }
  ): Promise<ApiResponse<Client>> {
    const response = await api.put<ApiResponse<Client>>(`/clients/${id}`, data);
    return response.data;
  },
};

// ============ 代理服务 ============
export const proxyService = {
  async getProxies(): Promise<ApiResponse<Proxy[]>> {
    const response = await api.get<ApiResponse<Proxy[]>>('/proxies');
    return response.data;
  },

  async getProxiesByClient(clientId: number): Promise<ApiResponse<Proxy[]>> {
    const response = await api.get<ApiResponse<Proxy[]>>(`/clients/${clientId}/proxies`);
    return response.data;
  },

  async createProxy(data: {
    client_id: string;
    name: string;
    type: string;
    localIP: string;
    localPort: number;
    remotePort: number;
  }): Promise<ApiResponse<Proxy>> {
    const response = await api.post<ApiResponse<Proxy>>('/proxies', data);
    return response.data;
  },

  async updateProxy(
    id: number,
    data: {
      name?: string;
      type?: string;
      localIP?: string;
      localPort?: number;
      remotePort?: number;
      enabled?: boolean;
    }
  ): Promise<ApiResponse<Proxy>> {
    const response = await api.put<ApiResponse<Proxy>>(`/proxies/${id}`, data);
    return response.data;
  },

  async deleteProxy(id: number): Promise<ApiResponse<string>> {
    const response = await api.delete<ApiResponse<string>>(`/proxies/${id}`);
    return response.data;
  },
};

// ============ 流量服务 ============
export const trafficService = {
  async getTrafficOverview(days?: number): Promise<ApiResponse<TrafficOverview>> {
    const response = await api.get<ApiResponse<TrafficOverview>>('/traffic/overview', {
      params: { days },
    });
    return response.data;
  },

  async getUserTraffic(userId: number, days?: number): Promise<ApiResponse<TrafficOverview>> {
    const response = await api.get<ApiResponse<TrafficOverview>>(`/traffic/user/${userId}`, {
      params: { days },
    });
    return response.data;
  },
};

// ============ Dashboard 服务 ============
export const dashboardService = {
  async getDashboardStats(userId: number): Promise<ApiResponse<DashboardStats>> {
    const response = await api.get<ApiResponse<DashboardStats>>(`/dashboard/${userId}`);
    return response.data;
  },
};
