import { useEffect, useState } from 'react';
import { useAuth } from '../contexts/AuthContext';
import { dashboardService } from '../lib/services';
import type { DashboardStats } from '../lib/types';
import { formatBytes } from '../lib/utils';
import { DashboardSkeleton } from '../components/Skeleton';

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
    return <DashboardSkeleton />;
  }

  return (
    <div className="space-y-6">
      {/* 欢迎横幅 */}
      <div className="bg-gradient-to-r from-blue-600 via-blue-700 to-indigo-700 rounded-2xl p-6 text-white shadow-lg shadow-blue-500/20">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold">欢迎回来，{user?.username}</h2>
            <p className="mt-1 text-blue-100">这是您的 RFRP 服务概览</p>
          </div>
          <div className="hidden sm:block">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1} stroke="currentColor" className="w-24 h-24 text-white/20">
              <path strokeLinecap="round" strokeLinejoin="round" d="M5.25 14.25h13.5m-13.5 0a3 3 0 01-3-3m3 3a3 3 0 100 6h13.5a3 3 0 100-6m-16.5-3a3 3 0 013-3h13.5a3 3 0 013 3m-19.5 0a4.5 4.5 0 01.9-2.7L5.737 5.1a3.375 3.375 0 012.7-1.35h7.126c1.062 0 2.062.5 2.7 1.35l2.587 3.45a4.5 4.5 0 01.9 2.7m0 0a3 3 0 01-3 3m0 3h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008zm-3 6h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008z" />
            </svg>
          </div>
        </div>
      </div>

      {/* 统计卡片 */}
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard
          title="总节点"
          value={stats?.total_clients || 0}
          icon={
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-6 h-6">
              <path strokeLinecap="round" strokeLinejoin="round" d="M21 7.5l-9-5.25L3 7.5m18 0l-9 5.25m9-5.25v9l-9 5.25M3 7.5l9 5.25M3 7.5v9l9 5.25m0-9v9" />
            </svg>
          }
          color="blue"
        />
        <StatCard
          title="在线节点"
          value={stats?.online_clients || 0}
          icon={
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-6 h-6">
              <path strokeLinecap="round" strokeLinejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          }
          color="green"
        />
        <StatCard
          title="总代理"
          value={stats?.total_proxies || 0}
          icon={
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-6 h-6">
              <path strokeLinecap="round" strokeLinejoin="round" d="M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" />
            </svg>
          }
          color="purple"
        />
        <StatCard
          title="启用代理"
          value={stats?.enabled_proxies || 0}
          icon={
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-6 h-6">
              <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 13.5l10.5-11.25L12 10.5h8.25L9.75 21.75 12 13.5H3.75z" />
            </svg>
          }
          color="amber"
        />
      </div>

      {/* 流量统计 */}
      <div className="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden">
        <div className="px-6 py-4 border-b border-gray-100">
          <h3 className="text-lg font-semibold text-gray-900">我的流量统计</h3>
        </div>
        <div className="p-6">
          <div className="grid grid-cols-1 gap-6 sm:grid-cols-3">
            <TrafficStatCard
              title="上传流量"
              value={formatBytes(stats?.user_traffic.total_bytes_sent || 0)}
              icon={
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-8 h-8">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M4.5 10.5L12 3m0 0l7.5 7.5M12 3v18" />
                </svg>
              }
              color="blue"
            />
            <TrafficStatCard
              title="下载流量"
              value={formatBytes(stats?.user_traffic.total_bytes_received || 0)}
              icon={
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-8 h-8">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 13.5L12 21m0 0l-7.5-7.5M12 21V3" />
                </svg>
              }
              color="green"
            />
            <TrafficStatCard
              title="总流量"
              value={formatBytes(stats?.user_traffic.total_bytes || 0)}
              icon={
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-8 h-8">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M3 7.5L7.5 3m0 0L12 7.5M7.5 3v13.5m13.5-13.5L16.5 7.5m0 0L12 3m4.5 4.5v13.5" />
                </svg>
              }
              color="purple"
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
  icon: React.ReactNode;
  color: 'blue' | 'green' | 'purple' | 'amber';
}

function StatCard({ title, value, icon, color }: StatCardProps) {
  const colorConfig = {
    blue: {
      bg: 'bg-blue-50',
      icon: 'text-blue-600',
      border: 'border-blue-100',
    },
    green: {
      bg: 'bg-green-50',
      icon: 'text-green-600',
      border: 'border-green-100',
    },
    purple: {
      bg: 'bg-purple-50',
      icon: 'text-purple-600',
      border: 'border-purple-100',
    },
    amber: {
      bg: 'bg-amber-50',
      icon: 'text-amber-600',
      border: 'border-amber-100',
    },
  };

  const config = colorConfig[color];

  return (
    <div className={`bg-white rounded-2xl p-5 shadow-sm border ${config.border} hover:shadow-md transition-shadow`}>
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium text-gray-500">{title}</p>
          <p className="mt-2 text-3xl font-bold text-gray-900">{value}</p>
        </div>
        <div className={`p-3 rounded-xl ${config.bg}`}>
          <span className={config.icon}>{icon}</span>
        </div>
      </div>
    </div>
  );
}

interface TrafficStatCardProps {
  title: string;
  value: string;
  icon: React.ReactNode;
  color: 'blue' | 'green' | 'purple';
}

function TrafficStatCard({ title, value, icon, color }: TrafficStatCardProps) {
  const colorConfig = {
    blue: {
      bg: 'bg-gradient-to-br from-blue-50 to-blue-100',
      icon: 'text-blue-500',
      text: 'text-blue-600',
    },
    green: {
      bg: 'bg-gradient-to-br from-green-50 to-green-100',
      icon: 'text-green-500',
      text: 'text-green-600',
    },
    purple: {
      bg: 'bg-gradient-to-br from-purple-50 to-purple-100',
      icon: 'text-purple-500',
      text: 'text-purple-600',
    },
  };

  const config = colorConfig[color];

  return (
    <div className={`${config.bg} rounded-xl p-6 text-center`}>
      <div className={`inline-flex items-center justify-center mb-3 ${config.icon}`}>
        {icon}
      </div>
      <div className="text-sm font-medium text-gray-600">{title}</div>
      <div className={`text-2xl font-bold mt-1 ${config.text}`}>{value}</div>
    </div>
  );
}
