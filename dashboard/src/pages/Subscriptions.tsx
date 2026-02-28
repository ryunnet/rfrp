import { useEffect, useState } from 'react';
import { subscriptionService } from '../lib/services';
import type { Subscription } from '../lib/types';
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
    maxPortCount: '',
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
      maxPortCount: '',
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
        max_port_count: formData.maxPortCount ? parseInt(formData.maxPortCount) : undefined,
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
        max_port_count: formData.maxPortCount ? parseInt(formData.maxPortCount) : undefined,
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
      maxPortCount: subscription.maxPortCount?.toString() || '',
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
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <h1 className="text-2xl font-bold text-foreground">订阅套餐管理</h1>
        <button
          onClick={() => {
            resetForm();
            setShowCreateModal(true);
          }}
          className="inline-flex items-center gap-2 px-5 py-2.5 text-primary-foreground text-sm font-medium rounded-xl focus:outline-none focus:ring-2 focus:ring-primary/40 shadow-sm transition-all duration-200 hover:opacity-90"
          style={{ background: 'linear-gradient(135deg, hsl(217 91% 60%), hsl(263 70% 58%))' }}
        >
          创建订阅套餐
        </button>
      </div>

      {loading ? (
        <TableSkeleton />
      ) : (
        <TableContainer>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>套餐名称</TableHead>
                <TableHead>周期</TableHead>
                <TableHead>流量配额</TableHead>
                <TableHead>端口数量</TableHead>
                <TableHead>价格</TableHead>
                <TableHead>状态</TableHead>
                <TableHead>创建时间</TableHead>
                <TableHead className="text-right">操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {subscriptions.map((subscription) => (
                <TableRow key={subscription.id}>
                  <TableCell>
                    <div className="text-sm font-medium text-foreground">{subscription.name}</div>
                    {subscription.description && (
                      <div className="text-sm text-muted-foreground">{subscription.description}</div>
                    )}
                  </TableCell>
                  <TableCell className="whitespace-nowrap text-sm text-foreground">
                    {getDurationText(subscription.durationType, subscription.durationValue)}
                  </TableCell>
                  <TableCell className="whitespace-nowrap text-sm text-foreground">
                    {subscription.trafficQuotaGb} GB
                  </TableCell>
                  <TableCell className="whitespace-nowrap text-sm text-foreground">
                    {subscription.maxPortCount || '无限制'}
                  </TableCell>
                  <TableCell className="whitespace-nowrap text-sm text-foreground">
                    {subscription.price ? `¥${subscription.price}` : '-'}
                  </TableCell>
                  <TableCell className="whitespace-nowrap">
                    <span
                      className="px-2 inline-flex text-xs leading-5 font-semibold rounded-full"
                      style={subscription.isActive
                        ? { background: 'hsl(142 71% 45% / 0.15)', color: 'hsl(142 71% 45%)' }
                        : { background: 'hsl(0 0% 50% / 0.1)', color: 'hsl(0 0% 45%)' }
                      }
                    >
                      {subscription.isActive ? '激活' : '停用'}
                    </span>
                  </TableCell>
                  <TableCell className="whitespace-nowrap text-sm text-muted-foreground">
                    {formatDate(subscription.createdAt)}
                  </TableCell>
                  <TableCell className="whitespace-nowrap text-right text-sm font-medium space-x-2">
                    <button
                      onClick={() => openEditModal(subscription)}
                      className="text-primary hover:text-primary/80"
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
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}

      {/* 创建/编辑订阅套餐模态框 */}
      {(showCreateModal || (showEditModal && selectedSubscription)) && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm overflow-y-auto h-full w-full flex items-center justify-center z-50">
          <div className="relative bg-white rounded-2xl shadow-2xl w-full max-w-md mx-4 transform transition-all">
            <div className="p-6">
              {/* 头部 */}
              <div className="flex items-center gap-3 mb-6">
                <div className="w-10 h-10 rounded-xl flex items-center justify-center" style={{ background: 'linear-gradient(135deg, hsl(217 91% 60%), hsl(263 70% 58%))' }}>
                  <svg className="w-5 h-5 text-primary-foreground" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M9 12h3.75M9 15h3.75M9 18h3.75m3 .75H18a2.25 2.25 0 0 0 2.25-2.25V6.108c0-1.135-.845-2.098-1.976-2.192a48.424 48.424 0 0 0-1.123-.08m-5.801 0c-.065.21-.1.433-.1.664 0 .414.336.75.75.75h4.5a.75.75 0 0 0 .75-.75 2.25 2.25 0 0 0-.1-.664m-5.8 0A2.251 2.251 0 0 1 13.5 2.25H15a2.25 2.25 0 0 1 2.15 1.586m-5.8 0c-.376.023-.75.05-1.124.08C9.095 4.01 8.25 4.973 8.25 6.108V8.25m0 0H4.875c-.621 0-1.125.504-1.125 1.125v11.25c0 .621.504 1.125 1.125 1.125h9.75c.621 0 1.125-.504 1.125-1.125V9.375c0-.621-.504-1.125-1.125-1.125H8.25Z" />
                  </svg>
                </div>
                <div>
                  <h3 className="text-lg font-bold text-foreground">
                    {showCreateModal ? '创建订阅套餐' : '编辑订阅套餐'}
                  </h3>
                  <p className="text-sm text-muted-foreground">
                    {showCreateModal ? '配置新的订阅套餐方案' : '修改订阅套餐配置'}
                  </p>
                </div>
              </div>

              {/* 表单 */}
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-foreground mb-1.5">套餐名称 *</label>
                  <input
                    type="text"
                    value={formData.name}
                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                    className="w-full px-4 py-3 border border-border rounded-xl text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary transition-all bg-muted/50 hover:bg-card"
                    placeholder="例如：月度套餐"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-foreground mb-1.5">订阅周期 *</label>
                  <div className="flex gap-2">
                    <select
                      value={formData.durationType}
                      onChange={(e) => setFormData({ ...formData, durationType: e.target.value })}
                      className="w-1/2 px-4 py-3 border border-border rounded-xl text-foreground focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary transition-all bg-muted/50 hover:bg-card"
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
                      className="w-1/2 px-4 py-3 border border-border rounded-xl text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary transition-all bg-muted/50 hover:bg-card"
                      placeholder="数量"
                      min="1"
                    />
                  </div>
                </div>
                <div>
                  <label className="block text-sm font-medium text-foreground mb-1.5">流量配额 (GB) *</label>
                  <input
                    type="number"
                    value={formData.trafficQuotaGb}
                    onChange={(e) => setFormData({ ...formData, trafficQuotaGb: e.target.value })}
                    className="w-full px-4 py-3 border border-border rounded-xl text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary transition-all bg-muted/50 hover:bg-card"
                    placeholder="例如：100"
                    min="0"
                    step="0.1"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-foreground mb-1.5">端口数量限制</label>
                  <input
                    type="number"
                    value={formData.maxPortCount}
                    onChange={(e) => setFormData({ ...formData, maxPortCount: e.target.value })}
                    className="w-full px-4 py-3 border border-border rounded-xl text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary transition-all bg-muted/50 hover:bg-card"
                    placeholder="留空表示无限制"
                    min="1"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-foreground mb-1.5">价格 (¥)</label>
                  <input
                    type="number"
                    value={formData.price}
                    onChange={(e) => setFormData({ ...formData, price: e.target.value })}
                    className="w-full px-4 py-3 border border-border rounded-xl text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary transition-all bg-muted/50 hover:bg-card"
                    placeholder="例如：29.9"
                    min="0"
                    step="0.01"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-foreground mb-1.5">描述</label>
                  <textarea
                    value={formData.description}
                    onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                    className="w-full px-4 py-3 border border-border rounded-xl text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/20 focus:border-primary transition-all bg-muted/50 hover:bg-card resize-none"
                    rows={3}
                    placeholder="套餐描述（可选）"
                  />
                </div>
                <div className="flex items-center gap-3 px-4 py-3 bg-muted/50 rounded-xl border border-border">
                  <button
                    type="button"
                    onClick={() => setFormData({ ...formData, isActive: !formData.isActive })}
                    className={`relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-primary/20 ${
                      formData.isActive ? 'bg-primary' : 'bg-muted'
                    }`}
                  >
                    <span
                      className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-card shadow ring-0 transition duration-200 ease-in-out ${
                        formData.isActive ? 'translate-x-5' : 'translate-x-0'
                      }`}
                    />
                  </button>
                  <span className="text-sm font-medium text-foreground">
                    {formData.isActive ? '已激活' : '已停用'}
                  </span>
                </div>
              </div>

              {/* 按钮 */}
              <div className="mt-6 flex gap-3">
                <button
                  onClick={() => {
                    setShowCreateModal(false);
                    setShowEditModal(false);
                    setSelectedSubscription(null);
                    resetForm();
                  }}
                  className="flex-1 px-4 py-2.5 bg-muted text-foreground font-medium rounded-xl hover:bg-accent transition-colors"
                >
                  取消
                </button>
                <button
                  onClick={showCreateModal ? handleCreateSubscription : handleEditSubscription}
                  className="flex-1 px-4 py-2.5 bg-primary text-primary-foreground font-medium rounded-xl hover:bg-primary/90 shadow-sm transition-all"
                >
                  {showCreateModal ? '创建' : '保存'}
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
