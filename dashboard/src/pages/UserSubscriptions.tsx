import { useEffect, useState } from 'react';
import { userSubscriptionService, subscriptionService, userService } from '../lib/services';
import type { UserSubscription, Subscription, UserWithNodeCount } from '../lib/types';
import { formatDate } from '../lib/utils';
import { useToast } from '../contexts/ToastContext';
import ConfirmDialog from '../components/ConfirmDialog';
import { TableSkeleton } from '../components/Skeleton';

export default function UserSubscriptions() {
  const { showToast } = useToast();
  const [userSubscriptions, setUserSubscriptions] = useState<UserSubscription[]>([]);
  const [subscriptions, setSubscriptions] = useState<Subscription[]>([]);
  const [users, setUsers] = useState<UserWithNodeCount[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [confirmDialog, setConfirmDialog] = useState<{
    open: boolean;
    title: string;
    message: string;
    variant: 'danger' | 'warning' | 'info';
    confirmText: string;
    onConfirm: () => void;
  }>({
    open: false,
    title: '',
    message: '',
    variant: 'danger',
    confirmText: '确定',
    onConfirm: () => {},
  });
  const [formData, setFormData] = useState({
    userId: '',
    subscriptionId: '',
  });

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      const [subsResponse, usersResponse, userSubsResponse] = await Promise.all([
        subscriptionService.getActiveSubscriptions(),
        userService.getUsers(),
        userSubscriptionService.getAllUserSubscriptions(),
      ]);

      if (subsResponse.success && subsResponse.data) {
        setSubscriptions(subsResponse.data);
      }
      if (usersResponse.success && usersResponse.data) {
        setUsers(usersResponse.data);
      }
      if (userSubsResponse.success && userSubsResponse.data) {
        setUserSubscriptions(userSubsResponse.data);
      }
    } catch (error) {
      console.error('加载数据失败:', error);
      showToast('加载失败', 'error');
    } finally {
      setLoading(false);
    }
  };

  const resetForm = () => {
    setFormData({
      userId: '',
      subscriptionId: '',
    });
  };

  const handleCreateUserSubscription = async () => {
    if (!formData.userId || !formData.subscriptionId) {
      showToast('请选择用户和订阅套餐', 'error');
      return;
    }

    try {
      const response = await userSubscriptionService.createUserSubscription({
        user_id: parseInt(formData.userId),
        subscription_id: parseInt(formData.subscriptionId),
      });
      if (response.success) {
        showToast('用户订阅创建成功', 'success');
        resetForm();
        setShowCreateModal(false);
        loadData();
      } else {
        showToast(response.message || '创建失败', 'error');
      }
    } catch (error) {
      console.error('创建用户订阅失败:', error);
      showToast('创建失败', 'error');
    }
  };

  const handleDeleteUserSubscription = (userSubscription: UserSubscription) => {
    setConfirmDialog({
      open: true,
      title: '删除用户订阅',
      message: `确定要删除用户订阅吗？`,
      variant: 'danger',
      confirmText: '删除',
      onConfirm: async () => {
        try {
          const response = await userSubscriptionService.deleteUserSubscription(userSubscription.id);
          if (response.success) {
            showToast('用户订阅删除成功', 'success');
            loadData();
          } else {
            showToast(response.message || '删除失败', 'error');
          }
        } catch (error) {
          console.error('删除用户订阅失败:', error);
          showToast('删除失败', 'error');
        }
      },
    });
  };

  const handleToggleActive = async (userSubscription: UserSubscription) => {
    try {
      const response = await userSubscriptionService.updateUserSubscription(userSubscription.id, {
        is_active: !userSubscription.isActive,
      });
      if (response.success) {
        showToast(`用户订阅已${userSubscription.isActive ? '停用' : '激活'}`, 'success');
        loadData();
      } else {
        showToast(response.message || '更新失败', 'error');
      }
    } catch (error) {
      console.error('更新用户订阅失败:', error);
      showToast('更新失败', 'error');
    }
  };

  const getUserName = (userId: number) => {
    const user = users.find((u) => u.id === userId);
    return user ? user.username : `用户 ${userId}`;
  };

  const getStatusBadge = (userSubscription: UserSubscription) => {
    if (userSubscription.isExpired) {
      return <span className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-red-100 text-red-800">已过期</span>;
    }
    if (!userSubscription.isActive) {
      return <span className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-gray-100 text-gray-800">已停用</span>;
    }
    return <span className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800">激活中</span>;
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
      <div className="w-full bg-gray-200 rounded-full h-2">
        <div className={`${colorClass} h-2 rounded-full`} style={{ width: `${Math.min(percentage, 100)}%` }}></div>
      </div>
    );
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <h1 className="text-2xl font-bold text-gray-900">用户订阅管理</h1>
        <button
          onClick={() => {
            resetForm();
            setShowCreateModal(true);
          }}
          className="inline-flex items-center gap-2 px-5 py-2.5 bg-gradient-to-r from-blue-600 to-indigo-600 text-white text-sm font-medium rounded-xl shadow-sm hover:from-blue-700 hover:to-indigo-700 transition-all"
        >
          分配订阅
        </button>
      </div>

      {loading ? (
        <TableSkeleton />
      ) : (
        <div className="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden">
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200 text-sm">
            <thead>
              <tr className="bg-gradient-to-r from-gray-50 to-gray-100/50">
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  用户
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  订阅套餐
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  有效期
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  流量使用
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  状态
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  操作
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {userSubscriptions.map((userSub) => (
                <tr key={userSub.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm font-medium text-gray-900">{getUserName(userSub.userId)}</div>
                    <div className="text-sm text-gray-500">ID: {userSub.userId}</div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {userSub.subscriptionName}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm text-gray-900">{formatDate(userSub.startDate)}</div>
                    <div className="text-sm text-gray-500">至 {formatDate(userSub.endDate)}</div>
                  </td>
                  <td className="px-6 py-4">
                    <div className="text-sm text-gray-900 mb-1">
                      {userSub.trafficUsedGb.toFixed(2)} / {userSub.trafficQuotaGb.toFixed(2)} GB
                    </div>
                    {getTrafficProgress(userSub.trafficUsedGb, userSub.trafficQuotaGb)}
                    <div className="text-xs text-gray-500 mt-1">
                      剩余: {userSub.trafficRemainingGb.toFixed(2)} GB
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    {getStatusBadge(userSub)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                    <div className="flex flex-wrap items-center justify-end gap-1.5">
                    {!userSub.isExpired && (
                      <button
                        onClick={() => handleToggleActive(userSub)}
                        className="text-yellow-600 hover:text-yellow-900"
                      >
                        {userSub.isActive ? '停用' : '激活'}
                      </button>
                    )}
                    <button
                      onClick={() => handleDeleteUserSubscription(userSub)}
                      className="text-red-600 hover:text-red-900"
                    >
                      删除
                    </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
            </table>
          </div>
        </div>
      )}

      {/* 创建用户订阅模态框 */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
          <div className="relative top-20 mx-auto p-5 border w-96 shadow-lg rounded-md bg-white">
            <h3 className="text-lg font-medium leading-6 text-gray-900 mb-4">分配订阅</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700">选择用户 *</label>
                <select
                  value={formData.userId}
                  onChange={(e) => setFormData({ ...formData, userId: e.target.value })}
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                >
                  <option value="">请选择用户</option>
                  {users.map((user) => (
                    <option key={user.id} value={user.id}>
                      {user.username} (ID: {user.id})
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">选择订阅套餐 *</label>
                <select
                  value={formData.subscriptionId}
                  onChange={(e) => setFormData({ ...formData, subscriptionId: e.target.value })}
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                >
                  <option value="">请选择订阅套餐</option>
                  {subscriptions.map((sub) => (
                    <option key={sub.id} value={sub.id}>
                      {sub.name} - {sub.trafficQuotaGb} GB
                      {sub.price && ` (¥${sub.price})`}
                    </option>
                  ))}
                </select>
              </div>
              <div className="text-sm text-gray-500">
                订阅将从当前时间开始生效
              </div>
            </div>
            <div className="mt-6 flex justify-end space-x-3">
              <button
                onClick={() => {
                  setShowCreateModal(false);
                  resetForm();
                }}
                className="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300"
              >
                取消
              </button>
              <button
                onClick={handleCreateUserSubscription}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
              >
                分配
              </button>
            </div>
          </div>
        </div>
      )}

      <ConfirmDialog
        open={confirmDialog.open}
        title={confirmDialog.title}
        message={confirmDialog.message}
        variant={confirmDialog.variant}
        confirmText={confirmDialog.confirmText}
        onConfirm={() => {
          confirmDialog.onConfirm();
          setConfirmDialog({ ...confirmDialog, open: false });
        }}
        onCancel={() => setConfirmDialog({ ...confirmDialog, open: false })}
      />
    </div>
  );
}
