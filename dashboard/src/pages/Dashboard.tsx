import { useEffect, useState } from 'react';
import { useAuth } from '../contexts/AuthContext';
import { dashboardService } from '../lib/services';
import type { DashboardStats } from '../lib/types';
import { formatBytes } from '../lib/utils';

export default function Dashboard() {
  const { user } = useAuth();
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (user) {
      loadStats();
    }
  }, [user]);

  const loadStats = async () => {
    try {
      setLoading(true);
      const response = await dashboardService.getDashboardStats(user!.id);
      if (response.success && response.data) {
        setStats(response.data);
      }
    } catch (error) {
      console.error('加载统计数据失败:', error);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900">仪表板</h2>
        <p className="mt-1 text-sm text-gray-600">欢迎回来，{user?.username}</p>
      </div>

      {/* 统计卡片 */}
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard
          title="总客户端"
          value={stats?.total_clients || 0}
          icon="💻"
          color="bg-blue-500"
        />
        <StatCard
          title="在线客户端"
          value={stats?.online_clients || 0}
          icon="🟢"
          color="bg-green-500"
        />
        <StatCard
          title="总代理"
          value={stats?.total_proxies || 0}
          icon="🔀"
          color="bg-purple-500"
        />
        <StatCard
          title="启用代理"
          value={stats?.enabled_proxies || 0}
          icon="✅"
          color="bg-green-500"
        />
      </div>

      {/* 流量统计 */}
      <div className="bg-white shadow rounded-lg">
        <div className="px-4 py-5 sm:p-6">
          <h3 className="text-lg font-medium leading-6 text-gray-900 mb-4">
            我的流量统计
          </h3>
          <div className="grid grid-cols-1 gap-5 sm:grid-cols-3">
            <TrafficStatCard
              title="上传流量"
              value={formatBytes(stats?.user_traffic.total_bytes_sent || 0)}
              icon="⬆️"
              color="text-blue-600"
            />
            <TrafficStatCard
              title="下载流量"
              value={formatBytes(stats?.user_traffic.total_bytes_received || 0)}
              icon="⬇️"
              color="text-green-600"
            />
            <TrafficStatCard
              title="总流量"
              value={formatBytes(stats?.user_traffic.total_bytes || 0)}
              icon="📊"
              color="text-purple-600"
            />
          </div>
        </div>
      </div>
    </div>
  );
}

interface StatCardProps {
  title: string;
  value: number;
  icon: string;
  color: string;
}

function StatCard({ title, value, icon, color }: StatCardProps) {
  return (
    <div className="bg-white overflow-hidden shadow rounded-lg">
      <div className="p-5">
        <div className="flex items-center">
          <div className="flex-shrink-0">
            <div className={`w-10 h-10 rounded-md ${color} flex items-center justify-center text-white text-lg`}>
              {icon}
            </div>
          </div>
          <div className="ml-5 w-0 flex-1">
            <dl>
              <dt className="text-sm font-medium text-gray-500 truncate">{title}</dt>
              <dd className="text-lg font-semibold text-gray-900">{value}</dd>
            </dl>
          </div>
        </div>
      </div>
    </div>
  );
}

interface TrafficStatCardProps {
  title: string;
  value: string;
  icon: string;
  color: string;
}

function TrafficStatCard({ title, value, icon, color }: TrafficStatCardProps) {
  return (
    <div className="text-center p-4 bg-gray-50 rounded-lg">
      <div className={`text-3xl mb-2 ${color}`}>{icon}</div>
      <div className="text-sm font-medium text-gray-500">{title}</div>
      <div className="text-xl font-bold text-gray-900 mt-1">{value}</div>
    </div>
  );
}
