import { useEffect, useState } from 'react';
import { userSubscriptionService, subscriptionService, userService } from '../lib/services';
import type { UserSubscription, Subscription, UserWithNodeCount } from '../lib/types';
import { formatDate } from '../lib/utils';
import { useToast } from '../contexts/ToastContext';
import ConfirmDialog from '../components/ConfirmDialog';
import { TableSkeleton } from '../components/Skeleton';
import {
  TableContainer,
  Table,
  TableHeader,
  TableBody,
  TableHead,
  TableRow,
  TableCell,
} from '../components/ui/table';

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
        <TableContainer>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>用户</TableHead>
                <TableHead>订阅套餐</TableHead>
                <TableHead>有效期</TableHead>
                <TableHead>流量使用</TableHead>
                <TableHead>状态</TableHead>
                <TableHead className="text-right">操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {userSubscriptions.map((userSub) => (
                <TableRow key={userSub.id}>
                  <TableCell className="whitespace-nowrap">
                    <div className="text-sm font-medium text-gray-900">{getUserName(userSub.userId)}</div>
                    <div className="text-sm text-gray-500">ID: {userSub.userId}</div>
                  </TableCell>
                  <TableCell className="whitespace-nowrap text-sm text-gray-900">
                    {userSub.subscriptionName}
                  </TableCell>
                  <TableCell className="whitespace-nowrap">
                    <div className="text-sm text-gray-900">{formatDate(userSub.startDate)}</div>
                    <div className="text-sm text-gray-500">至 {formatDate(userSub.endDate)}</div>
                  </TableCell>
                  <TableCell>
                    <div className="text-sm text-gray-900 mb-1">
                      {userSub.trafficUsedGb.toFixed(2)} / {userSub.trafficQuotaGb.toFixed(2)} GB
                    </div>
                    {getTrafficProgress(userSub.trafficUsedGb, userSub.trafficQuotaGb)}
                    <div className="text-xs text-gray-500 mt-1">
                      剩余: {userSub.trafficRemainingGb.toFixed(2)} GB
                    </div>
                  </TableCell>
                  <TableCell className="whitespace-nowrap">
                    {getStatusBadge(userSub)}
                  </TableCell>
                  <TableCell className="whitespace-nowrap text-right text-sm font-medium">
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
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}

      {/* 分配订阅模态框 */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm overflow-y-auto h-full w-full flex items-center justify-center z-50">
          <div className="relative bg-white rounded-2xl shadow-2xl w-full max-w-md mx-4 transform transition-all">
            <div className="p-6">
              {/* 头部 */}
              <div className="flex items-center gap-3 mb-6">
                <div className="w-10 h-10 bg-gradient-to-br from-emerald-500 to-teal-600 rounded-xl flex items-center justify-center">
                  <svg className="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M18 7.5v3m0 0v3m0-3h3m-3 0h-3m-2.25-4.125a3.375 3.375 0 1 1-6.75 0 3.375 3.375 0 0 1 6.75 0ZM3 19.235v-.11a6.375 6.375 0 0 1 12.75 0v.109A12.318 12.318 0 0 1 9.374 21c-2.331 0-4.512-.645-6.374-1.766Z" />
                  </svg>
                </div>
                <div>
                  <h3 className="text-lg font-bold text-gray-900">分配订阅</h3>
                  <p className="text-sm text-gray-500">为用户分配订阅套餐</p>
                </div>
              </div>

              {/* 表单 */}
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">选择用户 *</label>
                  <select
                    value={formData.userId}
                    onChange={(e) => setFormData({ ...formData, userId: e.target.value })}
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white"
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
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">选择订阅套餐 *</label>
                  <select
                    value={formData.subscriptionId}
                    onChange={(e) => setFormData({ ...formData, subscriptionId: e.target.value })}
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white"
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
                {formData.subscriptionId && (() => {
                  const selected = subscriptions.find(s => s.id === parseInt(formData.subscriptionId));
                  if (!selected) return null;
                  const typeMap: Record<string, string> = { daily: '天', weekly: '周', monthly: '月', yearly: '年' };
                  return (
                    <div className="px-4 py-3 bg-blue-50/80 rounded-xl border border-blue-100">
                      <div className="flex items-center gap-2 mb-1">
                        <svg className="w-4 h-4 text-blue-500" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" d="m11.25 11.25.041-.02a.75.75 0 0 1 1.063.852l-.708 2.836a.75.75 0 0 0 1.063.853l.041-.021M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9-3.75h.008v.008H12V8.25Z" />
                        </svg>
                        <span className="text-sm font-medium text-blue-700">套餐详情</span>
                      </div>
                      <div className="text-sm text-blue-600 space-y-0.5">
                        <p>周期：{selected.durationValue} {typeMap[selected.durationType] || selected.durationType}</p>
                        <p>流量：{selected.trafficQuotaGb} GB</p>
                        {selected.price && <p>价格：¥{selected.price}</p>}
                        {selected.description && <p>描述：{selected.description}</p>}
                      </div>
                    </div>
                  );
                })()}
                <div className="flex items-center gap-2 px-4 py-3 bg-gray-50/80 rounded-xl border border-gray-200">
                  <svg className="w-4 h-4 text-gray-400" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
                  </svg>
                  <span className="text-sm text-gray-500">订阅将从当前时间开始生效</span>
                </div>
              </div>

              {/* 按钮 */}
              <div className="mt-6 flex gap-3">
                <button
                  onClick={() => {
                    setShowCreateModal(false);
                    resetForm();
                  }}
                  className="flex-1 px-4 py-2.5 bg-gray-100 text-gray-700 font-medium rounded-xl hover:bg-gray-200 transition-colors"
                >
                  取消
                </button>
                <button
                  onClick={handleCreateUserSubscription}
                  className="flex-1 px-4 py-2.5 bg-gradient-to-r from-emerald-600 to-teal-600 text-white font-medium rounded-xl hover:from-emerald-700 hover:to-teal-700 shadow-lg shadow-emerald-500/25 transition-all"
                >
                  分配
                </button>
              </div>
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
