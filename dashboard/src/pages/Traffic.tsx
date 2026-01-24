import { useEffect, useState, useRef } from 'react';
import { useAuth } from '../contexts/AuthContext';
import { trafficService } from '../lib/services';
import type { TrafficOverview } from '../lib/types';
import { formatBytes, formatShortDate } from '../lib/utils';

export default function Traffic() {
  const { user } = useAuth();
  const [traffic, setTraffic] = useState<TrafficOverview | null>(null);
  const [days, setDays] = useState(30);
  const [loading, setLoading] = useState(true);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [refreshInterval, setRefreshInterval] = useState(5); // 默认5秒刷新
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    loadTraffic();
  }, [days]);

  useEffect(() => {
    // 清除之前的定时器
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
    }

    // 如果启用了自动刷新，设置新的定时器
    if (autoRefresh) {
      intervalRef.current = setInterval(() => {
        loadTrafficSilently();
      }, refreshInterval * 1000);
    }

    // 清理函数
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [autoRefresh, refreshInterval, days]);

  const loadTraffic = async () => {
    try {
      setLoading(true);
      const response = user?.is_admin
        ? await trafficService.getTrafficOverview(days)
        : await trafficService.getUserTraffic(user!.id, days);

      if (response.success && response.data) {
        setTraffic(response.data);
      }
    } catch (error) {
      console.error('加载流量统计失败:', error);
    } finally {
      setLoading(false);
    }
  };

  const loadTrafficSilently = async () => {
    try {
      const response = user?.is_admin
        ? await trafficService.getTrafficOverview(days)
        : await trafficService.getUserTraffic(user!.id, days);

      if (response.success && response.data) {
        setTraffic(response.data);
      }
    } catch (error) {
      console.error('刷新流量统计失败:', error);
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
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">流量统计</h2>
          <p className="mt-1 text-sm text-gray-600">查看流量使用情况</p>
        </div>
        <div className="flex items-center space-x-4">
          {/* 自动刷新控制 */}
          <div className="flex items-center space-x-2">
            <label className="flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={autoRefresh}
                onChange={(e) => setAutoRefresh(e.target.checked)}
                className="sr-only peer"
              />
              <div className="relative w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
              <span className="ms-3 text-sm font-medium text-gray-900">自动刷新</span>
            </label>

            {autoRefresh && (
              <select
                value={refreshInterval}
                onChange={(e) => setRefreshInterval(Number(e.target.value))}
                className="px-2 py-1 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value={3}>3秒</option>
                <option value={5}>5秒</option>
                <option value={10}>10秒</option>
                <option value={30}>30秒</option>
              </select>
            )}
          </div>

          <select
            value={days}
            onChange={(e) => setDays(Number(e.target.value))}
            className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value={7}>最近 7 天</option>
            <option value={30}>最近 30 天</option>
            <option value={90}>最近 90 天</option>
          </select>
        </div>
      </div>

      {/* 总流量统计 */}
      <div className="bg-white shadow rounded-lg">
        <div className="px-4 py-5 sm:p-6">
          <h3 className="text-lg font-medium leading-6 text-gray-900 mb-4">总流量统计</h3>
          <div className="grid grid-cols-1 gap-5 sm:grid-cols-3">
            <TrafficCard
              title="总上传"
              value={formatBytes(traffic?.total_traffic.total_bytes_sent || 0)}
              icon="⬆️"
              color="blue"
            />
            <TrafficCard
              title="总下载"
              value={formatBytes(traffic?.total_traffic.total_bytes_received || 0)}
              icon="⬇️"
              color="green"
            />
            <TrafficCard
              title="总流量"
              value={formatBytes(traffic?.total_traffic.total_bytes || 0)}
              icon="📊"
              color="purple"
            />
          </div>
        </div>
      </div>

      {/* 每日流量趋势 */}
      <div className="bg-white shadow rounded-lg">
        <div className="px-4 py-5 sm:p-6">
          <h3 className="text-lg font-medium leading-6 text-gray-900 mb-4">每日流量趋势</h3>
          {traffic && traffic.daily_traffic.length > 0 ? (
            <div className="space-y-2">
              {traffic.daily_traffic.slice(-10).map((day, index) => {
                const maxBytes = Math.max(...traffic.daily_traffic.map((d) => d.total_bytes));
                const percentage = maxBytes > 0 ? (day.total_bytes / maxBytes) * 100 : 0;
                return (
                  <div key={index} className="flex items-center space-x-3">
                    <div className="w-24 text-sm text-gray-500">{formatShortDate(day.date)}</div>
                    <div className="flex-1 bg-gray-100 rounded-full h-6 overflow-hidden">
                      <div
                        className="bg-gradient-to-r from-blue-500 to-purple-500 h-full rounded-full flex items-center justify-end pr-2 transition-all duration-500"
                        style={{ width: `${percentage}%` }}
                      >
                        <span className="text-xs text-white font-medium whitespace-nowrap">
                          {formatBytes(day.total_bytes)}
                        </span>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          ) : (
            <p className="text-gray-500 text-sm">暂无数据</p>
          )}
        </div>
      </div>

      {/* 用户流量排行 */}
      {traffic && traffic.by_user.length > 0 && (
        <div className="bg-white shadow rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <h3 className="text-lg font-medium leading-6 text-gray-900 mb-4">用户流量排行</h3>
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                      用户
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                      上传
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                      下载
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                      总流量
                    </th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {traffic.by_user.map((userTraffic) => (
                    <tr key={userTraffic.user_id} className="hover:bg-gray-50">
                      <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                        {userTraffic.username}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {formatBytes(userTraffic.total_bytes_sent)}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {formatBytes(userTraffic.total_bytes_received)}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {formatBytes(userTraffic.total_bytes)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      {/* 客户端流量排行 */}
      {traffic && traffic.by_client.length > 0 && (
        <div className="bg-white shadow rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <h3 className="text-lg font-medium leading-6 text-gray-900 mb-4">客户端流量排行</h3>
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                      客户端
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                      上传
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                      下载
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                      总流量
                    </th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {traffic.by_client.map((clientTraffic) => (
                    <tr key={clientTraffic.client_id} className="hover:bg-gray-50">
                      <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                        {clientTraffic.client_name}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {formatBytes(clientTraffic.total_bytes_sent)}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {formatBytes(clientTraffic.total_bytes_received)}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {formatBytes(clientTraffic.total_bytes)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      {/* 代理流量排行 */}
      {traffic && traffic.by_proxy.length > 0 && (
        <div className="bg-white shadow rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <h3 className="text-lg font-medium leading-6 text-gray-900 mb-4">代理流量排行</h3>
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                      代理名称
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                      所属客户端
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                      上传
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                      下载
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                      总流量
                    </th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {traffic.by_proxy.map((proxyTraffic) => (
                    <tr key={proxyTraffic.proxy_id} className="hover:bg-gray-50">
                      <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                        {proxyTraffic.proxy_name}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {proxyTraffic.client_name}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {formatBytes(proxyTraffic.total_bytes_sent)}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {formatBytes(proxyTraffic.total_bytes_received)}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {formatBytes(proxyTraffic.total_bytes)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

interface TrafficCardProps {
  title: string;
  value: string;
  icon: string;
  color: 'blue' | 'green' | 'purple';
}

function TrafficCard({ title, value, icon, color }: TrafficCardProps) {
  const colorClasses = {
    blue: 'bg-blue-100 text-blue-600',
    green: 'bg-green-100 text-green-600',
    purple: 'bg-purple-100 text-purple-600',
  };

  return (
    <div className="text-center p-4 bg-gray-50 rounded-lg">
      <div className={`text-3xl mb-2 ${colorClasses[color]}`}>{icon}</div>
      <div className="text-sm font-medium text-gray-500">{title}</div>
      <div className="text-xl font-bold text-gray-900 mt-1">{value}</div>
    </div>
  );
}
