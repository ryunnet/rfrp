import { useEffect, useState } from 'react';
import { subscriptionService } from '../lib/services';
import type { Subscription } from '../lib/types';
import { formatDate } from '../lib/utils';
import { useToast } from '../contexts/ToastContext';
import ConfirmDialog from '../components/ConfirmDialog';
import { TableSkeleton } from '../components/Skeleton';

export default function Subscriptions() {
  const { showToast } = useToast();
  const [subscriptions, setSubscriptions] = useState<Subscription[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [selectedSubscription, setSelectedSubscription] = useState<Subscription | null>(null);
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
    name: '',
    durationType: 'monthly',
    durationValue: '1',
    trafficQuotaGb: '',
    price: '',
    description: '',
    isActive: true,
  });

  useEffect(() => {
    loadSubscriptions();
  }, []);

  const loadSubscriptions = async () => {
    try {
      setLoading(true);
      const response = await subscriptionService.getSubscriptions();
      if (response.success && response.data) {
        setSubscriptions(response.data);
      }
    } catch (error) {
      console.error('加载订阅套餐失败:', error);
      showToast('加载失败', 'error');
    } finally {
      setLoading(false);
    }
  };

  const resetForm = () => {
    setFormData({
      name: '',
      durationType: 'monthly',
      durationValue: '1',
      trafficQuotaGb: '',
      price: '',
      description: '',
      isActive: true,
    });
  };

  const handleCreateSubscription = async () => {
    if (!formData.name || !formData.trafficQuotaGb) {
      showToast('请填写必填字段', 'error');
      return;
    }

    try {
      const response = await subscriptionService.createSubscription({
        name: formData.name,
        duration_type: formData.durationType,
        duration_value: parseInt(formData.durationValue) || 1,
        traffic_quota_gb: parseFloat(formData.trafficQuotaGb),
        price: formData.price ? parseFloat(formData.price) : undefined,
        description: formData.description || undefined,
        is_active: formData.isActive,
      });
      if (response.success) {
        showToast('订阅套餐创建成功', 'success');
        resetForm();
        setShowCreateModal(false);
        loadSubscriptions();
      } else {
        showToast(response.message || '创建失败', 'error');
      }
    } catch (error) {
      console.error('创建订阅套餐失败:', error);
      showToast('创建失败', 'error');
    }
  };

  const handleEditSubscription = async () => {
    if (!selectedSubscription || !formData.name || !formData.trafficQuotaGb) {
      showToast('请填写必填字段', 'error');
      return;
    }

    try {
      const response = await subscriptionService.updateSubscription(selectedSubscription.id, {
        name: formData.name,
        duration_type: formData.durationType,
        duration_value: parseInt(formData.durationValue) || 1,
        traffic_quota_gb: parseFloat(formData.trafficQuotaGb),
        price: formData.price ? parseFloat(formData.price) : undefined,
        description: formData.description || undefined,
        is_active: formData.isActive,
      });
      if (response.success) {
        showToast('订阅套餐更新成功', 'success');
        resetForm();
        setShowEditModal(false);
        setSelectedSubscription(null);
        loadSubscriptions();
      } else {
        showToast(response.message || '更新失败', 'error');
      }
    } catch (error) {
      console.error('更新订阅套餐失败:', error);
      showToast('更新失败', 'error');
    }
  };

  const handleDeleteSubscription = (subscription: Subscription) => {
    setConfirmDialog({
      open: true,
      title: '删除订阅套餐',
      message: `确定要删除订阅套餐"${subscription.name}"吗？`,
      variant: 'danger',
      confirmText: '删除',
      onConfirm: async () => {
        try {
          const response = await subscriptionService.deleteSubscription(subscription.id);
          if (response.success) {
            showToast('订阅套餐删除成功', 'success');
            loadSubscriptions();
          } else {
            showToast(response.message || '删除失败', 'error');
          }
        } catch (error) {
          console.error('删除订阅套餐失败:', error);
          showToast('删除失败', 'error');
        }
      },
    });
  };

  const handleToggleActive = async (subscription: Subscription) => {
    try {
      const response = await subscriptionService.updateSubscription(subscription.id, {
        is_active: !subscription.isActive,
      });
      if (response.success) {
        showToast(`订阅套餐已${subscription.isActive ? '停用' : '激活'}`, 'success');
        loadSubscriptions();
      } else {
        showToast(response.message || '更新失败', 'error');
      }
    } catch (error) {
      console.error('更新订阅套餐失败:', error);
      showToast('更新失败', 'error');
    }
  };

  const openEditModal = (subscription: Subscription) => {
    setSelectedSubscription(subscription);
    setFormData({
      name: subscription.name,
      durationType: subscription.durationType,
      durationValue: subscription.durationValue.toString(),
      trafficQuotaGb: subscription.trafficQuotaGb.toString(),
      price: subscription.price?.toString() || '',
      description: subscription.description || '',
      isActive: subscription.isActive,
    });
    setShowEditModal(true);
  };

  const getDurationText = (type: string, value: number) => {
    const typeMap: Record<string, string> = {
      daily: '天',
      weekly: '周',
      monthly: '月',
      yearly: '年',
    };
    return `${value} ${typeMap[type] || type}`;
  };

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-bold text-gray-900">订阅套餐管理</h1>
        <button
          onClick={() => {
            resetForm();
            setShowCreateModal(true);
          }}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          创建订阅套餐
        </button>
      </div>

      {loading ? (
        <TableSkeleton />
      ) : (
        <div className="bg-white rounded-lg shadow overflow-hidden">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  套餐名称
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  周期
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  流量配额
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  价格
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  状态
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  创建时间
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  操作
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {subscriptions.map((subscription) => (
                <tr key={subscription.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm font-medium text-gray-900">{subscription.name}</div>
                    {subscription.description && (
                      <div className="text-sm text-gray-500">{subscription.description}</div>
                    )}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {getDurationText(subscription.durationType, subscription.durationValue)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {subscription.trafficQuotaGb} GB
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {subscription.price ? `¥${subscription.price}` : '-'}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span
                      className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${
                        subscription.isActive
                          ? 'bg-green-100 text-green-800'
                          : 'bg-gray-100 text-gray-800'
                      }`}
                    >
                      {subscription.isActive ? '激活' : '停用'}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {formatDate(subscription.createdAt)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-2">
                    <button
                      onClick={() => openEditModal(subscription)}
                      className="text-blue-600 hover:text-blue-900"
                    >
                      编辑
                    </button>
                    <button
                      onClick={() => handleToggleActive(subscription)}
                      className="text-yellow-600 hover:text-yellow-900"
                    >
                      {subscription.isActive ? '停用' : '激活'}
                    </button>
                    <button
                      onClick={() => handleDeleteSubscription(subscription)}
                      className="text-red-600 hover:text-red-900"
                    >
                      删除
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* 创建订阅套餐模态框 */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
          <div className="relative top-20 mx-auto p-5 border w-96 shadow-lg rounded-md bg-white">
            <h3 className="text-lg font-medium leading-6 text-gray-900 mb-4">创建订阅套餐</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700">套餐名称 *</label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                  placeholder="例如：月度套餐"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">订阅周期 *</label>
                <div className="flex space-x-2">
                  <select
                    value={formData.durationType}
                    onChange={(e) => setFormData({ ...formData, durationType: e.target.value })}
                    className="mt-1 block w-1/2 rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                  >
                    <option value="daily">日</option>
                    <option value="weekly">周</option>
                    <option value="monthly">月</option>
                    <option value="yearly">年</option>
                  </select>
                  <input
                    type="number"
                    value={formData.durationValue}
                    onChange={(e) => setFormData({ ...formData, durationValue: e.target.value })}
                    className="mt-1 block w-1/2 rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                    placeholder="数量"
                    min="1"
                  />
                </div>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">流量配额 (GB) *</label>
                <input
                  type="number"
                  value={formData.trafficQuotaGb}
                  onChange={(e) => setFormData({ ...formData, trafficQuotaGb: e.target.value })}
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                  placeholder="例如：100"
                  min="0"
                  step="0.1"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">价格 (¥)</label>
                <input
                  type="number"
                  value={formData.price}
                  onChange={(e) => setFormData({ ...formData, price: e.target.value })}
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                  placeholder="例如：29.9"
                  min="0"
                  step="0.01"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">描述</label>
                <textarea
                  value={formData.description}
                  onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                  rows={3}
                  placeholder="套餐描述"
                />
              </div>
              <div className="flex items-center">
                <input
                  type="checkbox"
                  checked={formData.isActive}
                  onChange={(e) => setFormData({ ...formData, isActive: e.target.checked })}
                  className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                />
                <label className="ml-2 block text-sm text-gray-900">激活状态</label>
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
                onClick={handleCreateSubscription}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
              >
                创建
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 编辑订阅套餐模态框 */}
      {showEditModal && selectedSubscription && (
        <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
          <div className="relative top-20 mx-auto p-5 border w-96 shadow-lg rounded-md bg-white">
            <h3 className="text-lg font-medium leading-6 text-gray-900 mb-4">编辑订阅套餐</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700">套餐名称 *</label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">订阅周期 *</label>
                <div className="flex space-x-2">
                  <select
                    value={formData.durationType}
                    onChange={(e) => setFormData({ ...formData, durationType: e.target.value })}
                    className="mt-1 block w-1/2 rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                  >
                    <option value="daily">日</option>
                    <option value="weekly">周</option>
                    <option value="monthly">月</option>
                    <option value="yearly">年</option>
                  </select>
                  <input
                    type="number"
                    value={formData.durationValue}
                    onChange={(e) => setFormData({ ...formData, durationValue: e.target.value })}
                    className="mt-1 block w-1/2 rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                    min="1"
                  />
                </div>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">流量配额 (GB) *</label>
                <input
                  type="number"
                  value={formData.trafficQuotaGb}
                  onChange={(e) => setFormData({ ...formData, trafficQuotaGb: e.target.value })}
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                  min="0"
                  step="0.1"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">价格 (¥)</label>
                <input
                  type="number"
                  value={formData.price}
                  onChange={(e) => setFormData({ ...formData, price: e.target.value })}
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                  min="0"
                  step="0.01"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">描述</label>
                <textarea
                  value={formData.description}
                  onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                  className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                  rows={3}
                />
              </div>
              <div className="flex items-center">
                <input
                  type="checkbox"
                  checked={formData.isActive}
                  onChange={(e) => setFormData({ ...formData, isActive: e.target.checked })}
                  className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                />
                <label className="ml-2 block text-sm text-gray-900">激活状态</label>
              </div>
            </div>
            <div className="mt-6 flex justify-end space-x-3">
              <button
                onClick={() => {
                  setShowEditModal(false);
                  setSelectedSubscription(null);
                  resetForm();
                }}
                className="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300"
              >
                取消
              </button>
              <button
                onClick={handleEditSubscription}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
              >
                保存
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
