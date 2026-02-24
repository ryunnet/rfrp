import { useEffect, useState } from 'react';
import { userSubscriptionService } from '../lib/services';
import type { UserSubscription } from '../lib/types';
import { formatDate } from '../lib/utils';
import { useToast } from '../contexts/ToastContext';
import { useAuth } from '../contexts/AuthContext';
import { TableSkeleton } from '../components/Skeleton';

export default function MySubscription() {
  const { showToast } = useToast();
  const { user } = useAuth();
  const [subscriptions, setSubscriptions] = useState<UserSubscription[]>([]);
  const [activeSubscription, setActiveSubscription] = useState<UserSubscription | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (user?.id) {
      loadSubscriptions();
    }
  }, [user]);

  const loadSubscriptions = async () => {
    if (!user?.id) return;

    try {
      setLoading(true);
      const [subsResponse, activeResponse] = await Promise.all([
        userSubscriptionService.getUserSubscriptions(user.id),
        userSubscriptionService.getUserActiveSubscription(user.id),
      ]);

      if (subsResponse.success && subsResponse.data) {
        setSubscriptions(subsResponse.data);
      }
      if (activeResponse.success && activeResponse.data) {
        setActiveSubscription(activeResponse.data);
      }
    } catch (error) {
      console.error('加载订阅信息失败:', error);
      showToast('加载失败', 'error');
    } finally {
      setLoading(false);
    }
  };

  const getStatusBadge = (subscription: UserSubscription) => {
    if (subscription.isExpired) {
      return <span className="px-3 py-1 text-sm font-semibold rounded-full bg-red-100 text-red-800">已过期</span>;
    }
    if (!subscription.isActive) {
      return <span className="px-3 py-1 text-sm font-semibold rounded-full bg-gray-100 text-gray-800">已停用</span>;
    }
    return <span className="px-3 py-1 text-sm font-semibold rounded-full bg-green-100 text-green-800">激活中</span>;
  };

  const getTrafficProgress = (used: number, total: number) => {
    const percentage = (used / total) * 100;
    let colorClass = 'bg-green-500';
    if (percentage >= 90) {
      colorClass = 'bg-red-500';
    } else if (percentage >= 70) {
      colorClass = 'bg-yellow-500';
    }
    return (
      <div className="w-full bg-gray-200 rounded-full h-3">
        <div className={`${colorClass} h-3 rounded-full transition-all duration-300`} style={{ width: `${Math.min(percentage, 100)}%` }}></div>
      </div>
    );
  };

  const formatTraffic = (gb: number) => {
    if (gb >= 1024) {
      return `${(gb / 1024).toFixed(2)} TB`;
    }
    return `${gb.toFixed(2)} GB`;
  };

  const getDaysRemaining = (endDate: string) => {
    const end = new Date(endDate);
    const now = new Date();
    const diff = end.getTime() - now.getTime();
    const days = Math.ceil(diff / (1000 * 60 * 60 * 24));
    return days > 0 ? days : 0;
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <h1 className="text-2xl font-bold text-gray-900">我的订阅</h1>
      </div>

      {loading ? (
        <TableSkeleton />
      ) : (
        <>
          {/* 当前激活的订阅 */}
          {activeSubscription && (
            <div className="bg-gradient-to-br from-blue-50 to-indigo-50 rounded-2xl shadow-sm p-6 border border-blue-200">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-xl font-bold text-gray-900">当前订阅</h2>
                {getStatusBadge(activeSubscription)}
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-4">
                  <div>
                    <p className="text-sm text-gray-600 mb-1">套餐名称</p>
                    <p className="text-lg font-semibold text-gray-900">{activeSubscription.subscriptionName}</p>
                  </div>

                  <div>
                    <p className="text-sm text-gray-600 mb-1">有效期</p>
                    <p className="text-base text-gray-900">
                      {formatDate(activeSubscription.startDate)} 至 {formatDate(activeSubscription.endDate)}
                    </p>
                    <p className="text-sm text-blue-600 font-medium mt-1">
                      剩余 {getDaysRemaining(activeSubscription.endDate)} 天
                    </p>
                  </div>
                </div>

                <div className="space-y-4">
                  <div>
                    <p className="text-sm text-gray-600 mb-2">流量使用情况</p>
                    <div className="space-y-2">
                      <div className="flex justify-between text-sm">
                        <span className="text-gray-700">已使用</span>
                        <span className="font-semibold text-gray-900">
                          {formatTraffic(activeSubscription.trafficUsedGb)} / {formatTraffic(activeSubscription.trafficQuotaGb)}
                        </span>
                      </div>
                      {getTrafficProgress(activeSubscription.trafficUsedGb, activeSubscription.trafficQuotaGb)}
                      <div className="flex justify-between text-sm">
                        <span className="text-gray-700">剩余流量</span>
                        <span className="font-semibold text-green-600">
                          {formatTraffic(activeSubscription.trafficRemainingGb)}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* 订阅历史 */}
          <div className="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden">
            <div className="px-6 py-4 border-b border-gray-200">
              <h2 className="text-lg font-semibold text-gray-900">订阅历史</h2>
            </div>

            {subscriptions.length === 0 ? (
              <div className="p-8 text-center text-gray-500">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-16 h-16 mx-auto mb-4 text-gray-400">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M9 12h3.75M9 15h3.75M9 18h3.75m3 .75H18a2.25 2.25 0 002.25-2.25V6.108c0-1.135-.845-2.098-1.976-2.192a48.424 48.424 0 00-1.123-.08m-5.801 0c-.065.21-.1.433-.1.664 0 .414.336.75.75.75h4.5a.75.75 0 00.75-.75 2.25 2.25 0 00-.1-.664m-5.8 0A2.251 2.251 0 0113.5 2.25H15c1.012 0 1.867.668 2.15 1.586m-5.8 0c-.376.023-.75.05-1.124.08C9.095 4.01 8.25 4.973 8.25 6.108V8.25m0 0H4.875c-.621 0-1.125.504-1.125 1.125v11.25c0 .621.504 1.125 1.125 1.125h9.75c.621 0 1.125-.504 1.125-1.125V9.375c0-.621-.504-1.125-1.125-1.125H8.25zM6.75 12h.008v.008H6.75V12zm0 3h.008v.008H6.75V15zm0 3h.008v.008H6.75V18z" />
                </svg>
                <p className="text-lg font-medium">暂无订阅记录</p>
                <p className="text-sm mt-2">请联系管理员为您分配订阅套餐</p>
              </div>
            ) : (
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200 text-sm">
                <thead>
                  <tr className="bg-gradient-to-r from-gray-50 to-gray-100/50">
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      套餐名称
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      有效期
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      流量配额
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      已使用
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      状态
                    </th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {subscriptions.map((sub) => (
                    <tr key={sub.id} className="hover:bg-gray-50">
                      <td className="px-6 py-4 whitespace-nowrap">
                        <div className="text-sm font-medium text-gray-900">{sub.subscriptionName}</div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        <div className="text-sm text-gray-900">{formatDate(sub.startDate)}</div>
                        <div className="text-sm text-gray-500">至 {formatDate(sub.endDate)}</div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        {formatTraffic(sub.trafficQuotaGb)}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        <div className="text-sm text-gray-900">{formatTraffic(sub.trafficUsedGb)}</div>
                        <div className="text-xs text-gray-500">
                          {((sub.trafficUsedGb / sub.trafficQuotaGb) * 100).toFixed(1)}%
                        </div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        {getStatusBadge(sub)}
                      </td>
                    </tr>
                  ))}
                </tbody>
                </table>
            </div>
            )}
          </div>
        </>
      )}
    </div>
  );
}
