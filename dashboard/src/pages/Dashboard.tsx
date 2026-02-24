import { useEffect, useState } from 'react';
import { useAuth } from '../contexts/AuthContext';
import { dashboardService } from '../lib/services';
import type { DashboardStats } from '../lib/types';
import { formatBytes } from '../lib/utils';
import { DashboardSkeleton } from '../components/Skeleton';
import { useNavigate } from 'react-router-dom';

export default function Dashboard() {
  const { user, isAdmin } = useAuth();
  const navigate = useNavigate();
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
    <div className="space-y-8">
      {/* 欢迎横幅 - 现代化设计 */}
      <div className="relative overflow-hidden bg-gradient-to-br from-blue-600 via-indigo-600 to-purple-700 rounded-3xl p-8 text-white shadow-2xl shadow-blue-500/30 animate-gradient">
        <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjAiIGhlaWdodD0iNjAiIHZpZXdCb3g9IjAgMCA2MCA2MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZyBmaWxsPSJub25lIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiPjxnIGZpbGw9IiNmZmYiIGZpbGwtb3BhY2l0eT0iMC4xIj48cGF0aCBkPSJNMzYgMzRjMC0yLjIxLTEuNzktNC00LTRzLTQgMS43OS00IDQgMS43OSA0IDQgNCA0LTEuNzkgNC00em0wLTEwYzAtMi4yMS0xLjc5LTQtNC00cy00IDEuNzktNCA0IDEuNzkgNCA0IDQgNC0xLjc5IDQtNHptMC0xMGMwLTIuMjEtMS43OS00LTQtNHMtNCAxLjc5LTQgNCAxLjc5IDQgNCA0IDQtMS43OSA0LTR6Ii8+PC9nPjwvZz48L3N2Zz4=')] opacity-20"></div>
        <div className="relative z-10 flex items-center justify-between">
          <div className="flex-1">
            <div className="flex items-center gap-3 mb-2">
              <div className="w-12 h-12 bg-white/20 backdrop-blur-sm rounded-2xl flex items-center justify-center shadow-lg">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-7 h-7">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M15.59 14.37a6 6 0 01-5.84 7.38v-4.8m5.84-2.58a14.98 14.98 0 006.16-12.12A14.98 14.98 0 009.631 8.41m5.96 5.96a14.926 14.926 0 01-5.841 2.58m-.119-8.54a6 6 0 00-7.381 5.84h4.8m2.581-5.84a14.927 14.927 0 00-2.58 5.84m2.699 2.7c-.103.021-.207.041-.311.06a15.09 15.09 0 01-2.448-2.448 14.9 14.9 0 01.06-.312m-2.24 2.39a4.493 4.493 0 00-1.757 4.306 4.493 4.493 0 004.306-1.758M16.5 9a1.5 1.5 0 11-3 0 1.5 1.5 0 013 0z" />
                </svg>
              </div>
              <div>
                <h2 className="text-3xl font-bold">欢迎回来，{user?.username}</h2>
                <p className="mt-1 text-blue-100 font-medium">这是您的 RFRP 服务概览</p>
              </div>
            </div>
          </div>
          <div className="hidden lg:block">
            <div className="w-32 h-32 bg-white/10 backdrop-blur-sm rounded-3xl flex items-center justify-center shadow-2xl">
              <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1} stroke="currentColor" className="w-20 h-20 text-white/40">
                <path strokeLinecap="round" strokeLinejoin="round" d="M5.25 14.25h13.5m-13.5 0a3 3 0 01-3-3m3 3a3 3 0 100 6h13.5a3 3 0 100-6m-16.5-3a3 3 0 013-3h13.5a3 3 0 013 3m-19.5 0a4.5 4.5 0 01.9-2.7L5.737 5.1a3.375 3.375 0 012.7-1.35h7.126c1.062 0 2.062.5 2.7 1.35l2.587 3.45a4.5 4.5 0 01.9 2.7m0 0a3 3 0 01-3 3m0 3h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008zm-3 6h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008z" />
              </svg>
            </div>
          </div>
        </div>
      </div>

      {/* 统计卡片 - 现代化设计 */}
      <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-5">
        <StatCard
          title="总客户端"
          value={stats?.total_clients || 0}
          icon={
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-6 h-6">
              <path strokeLinecap="round" strokeLinejoin="round" d="M21 7.5l-9-5.25L3 7.5m18 0l-9 5.25m9-5.25v9l-9 5.25M3 7.5l9 5.25M3 7.5v9l9 5.25m0-9v9" />
            </svg>
          }
          color="blue"
        />
        <StatCard
          title="在线客户端"
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
        <StatCard
          title="用户总配额(GB)"
          value={stats?.user_total_quota_gb == null ? '无限制' : stats.user_total_quota_gb.toFixed(2)}
          icon={
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-6 h-6">
              <path strokeLinecap="round" strokeLinejoin="round" d="M2.25 18L9 11.25l4.306 4.306a11.95 11.95 0 005.814-5.517L21.75 6.75M16.5 6.75h5.25V12" />
            </svg>
          }
          color="teal"
        />
      </div>

      {/* 节点统计卡片（仅管理员可见） */}
      {isAdmin && (
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
          <div
            onClick={() => navigate('/nodes')}
            className="group relative overflow-hidden bg-white/80 backdrop-blur-sm rounded-3xl p-6 shadow-lg border border-gray-200/50 hover:shadow-2xl hover:border-teal-300/50 transition-all duration-300 cursor-pointer card-hover"
          >
            <div className="absolute inset-0 bg-gradient-to-br from-teal-50 to-cyan-50 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
            <div className="relative z-10 flex items-center justify-between">
              <div>
                <p className="text-sm font-bold text-gray-500 uppercase tracking-wider">总节点</p>
                <p className="mt-3 text-4xl font-black text-gray-900">{stats?.total_nodes || 0}</p>
                <p className="mt-2 text-xs text-gray-500 font-medium">点击查看详情</p>
              </div>
              <div className="p-4 rounded-2xl bg-gradient-to-br from-teal-500 to-cyan-600 shadow-lg shadow-teal-500/30 group-hover:scale-110 transition-transform duration-300">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-8 h-8 text-white">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M5.25 14.25h13.5m-13.5 0a3 3 0 01-3-3m3 3a3 3 0 100 6h13.5a3 3 0 100-6m-16.5-3a3 3 0 013-3h13.5a3 3 0 013 3m-19.5 0a4.5 4.5 0 01.9-2.7L5.737 5.1a3.375 3.375 0 012.7-1.35h7.126c1.062 0 2.062.5 2.7 1.35l2.587 3.45a4.5 4.5 0 01.9 2.7m0 0a3 3 0 01-3 3m0 3h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008zm-3 6h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008z" />
                </svg>
              </div>
            </div>
          </div>
          <div
            onClick={() => navigate('/nodes')}
            className="group relative overflow-hidden bg-white/80 backdrop-blur-sm rounded-3xl p-6 shadow-lg border border-gray-200/50 hover:shadow-2xl hover:border-emerald-300/50 transition-all duration-300 cursor-pointer card-hover"
          >
            <div className="absolute inset-0 bg-gradient-to-br from-emerald-50 to-green-50 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
            <div className="relative z-10 flex items-center justify-between">
              <div>
                <p className="text-sm font-bold text-gray-500 uppercase tracking-wider">在线节点</p>
                <p className="mt-3 text-4xl font-black text-gray-900">{stats?.online_nodes || 0}</p>
                <p className="mt-2 text-xs text-gray-500 font-medium">点击查看详情</p>
              </div>
              <div className="p-4 rounded-2xl bg-gradient-to-br from-emerald-500 to-green-600 shadow-lg shadow-emerald-500/30 group-hover:scale-110 transition-transform duration-300">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-8 h-8 text-white">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 流量统计 - 现代化设计 */}
      <div className="bg-white/80 backdrop-blur-sm rounded-3xl shadow-lg border border-gray-200/50 overflow-hidden">
        <div className="px-8 py-6 border-b border-gray-100 bg-gradient-to-r from-gray-50 to-blue-50">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-blue-500 to-indigo-600 flex items-center justify-center shadow-lg shadow-blue-500/30">
              <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-5 h-5 text-white">
                <path strokeLinecap="round" strokeLinejoin="round" d="M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z" />
              </svg>
            </div>
            <h3 className="text-xl font-bold text-gray-900">我的流量统计</h3>
          </div>
        </div>
        <div className="p-8">
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
  value: number | string;
  icon: React.ReactNode;
  color: 'blue' | 'green' | 'purple' | 'amber' | 'teal';
}

function StatCard({ title, value, icon, color }: StatCardProps) {
  const colorConfig = {
    blue: {
      bg: 'bg-gradient-to-br from-blue-50 to-blue-100',
      icon: 'text-blue-600',
      border: 'border-blue-200/50',
      shadow: 'shadow-blue-500/20',
      hover: 'hover:shadow-blue-500/30',
    },
    green: {
      bg: 'bg-gradient-to-br from-green-50 to-emerald-100',
      icon: 'text-green-600',
      border: 'border-green-200/50',
      shadow: 'shadow-green-500/20',
      hover: 'hover:shadow-green-500/30',
    },
    purple: {
      bg: 'bg-gradient-to-br from-purple-50 to-indigo-100',
      icon: 'text-purple-600',
      border: 'border-purple-200/50',
      shadow: 'shadow-purple-500/20',
      hover: 'hover:shadow-purple-500/30',
    },
    amber: {
      bg: 'bg-gradient-to-br from-amber-50 to-yellow-100',
      icon: 'text-amber-600',
      border: 'border-amber-200/50',
      shadow: 'shadow-amber-500/20',
      hover: 'hover:shadow-amber-500/30',
    },
    teal: {
      bg: 'bg-gradient-to-br from-teal-50 to-cyan-100',
      icon: 'text-teal-600',
      border: 'border-teal-200/50',
      shadow: 'shadow-teal-500/20',
      hover: 'hover:shadow-teal-500/30',
    },
  };

  const config = colorConfig[color];

  return (
    <div className={`group relative overflow-hidden bg-white/80 backdrop-blur-sm rounded-3xl p-6 shadow-lg border ${config.border} ${config.shadow} ${config.hover} transition-all duration-300 card-hover`}>
      <div className={`absolute inset-0 ${config.bg} opacity-0 group-hover:opacity-100 transition-opacity duration-300`}></div>
      <div className="relative z-10 flex items-center justify-between">
        <div>
          <p className="text-sm font-bold text-gray-500 uppercase tracking-wider">{title}</p>
          <p className="mt-3 text-4xl font-black text-gray-900">{value}</p>
        </div>
        <div className={`p-4 rounded-2xl ${config.bg} shadow-lg ${config.shadow} group-hover:scale-110 transition-transform duration-300`}>
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
      bg: 'bg-gradient-to-br from-blue-500 to-indigo-600',
      text: 'text-white',
      shadow: 'shadow-blue-500/30',
    },
    green: {
      bg: 'bg-gradient-to-br from-green-500 to-emerald-600',
      text: 'text-white',
      shadow: 'shadow-green-500/30',
    },
    purple: {
      bg: 'bg-gradient-to-br from-purple-500 to-indigo-600',
      text: 'text-white',
      shadow: 'shadow-purple-500/30',
    },
  };

  const config = colorConfig[color];

  return (
    <div className={`group relative overflow-hidden ${config.bg} rounded-2xl p-8 text-center shadow-xl ${config.shadow} hover:shadow-2xl transition-all duration-300 card-hover`}>
      <div className="absolute inset-0 bg-white/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
      <div className={`relative z-10 inline-flex items-center justify-center mb-4 ${config.text} group-hover:scale-110 transition-transform duration-300`}>
        {icon}
      </div>
      <div className={`text-sm font-bold ${config.text} opacity-90 uppercase tracking-wider`}>{title}</div>
      <div className={`text-3xl font-black mt-2 ${config.text}`}>{value}</div>
    </div>
  );
}
