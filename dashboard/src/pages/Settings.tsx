import { useState, useEffect } from 'react';
import { useAuth } from '../contexts/AuthContext';
import { systemService } from '../lib/services';
import { useToast } from '../contexts/ToastContext';
import ConfirmDialog from '../components/ConfirmDialog';
import SkeletonBlock from '../components/Skeleton';

interface ConfigItem {
  id: number;
  key: string;
  value: number | string | boolean;
  description: string;
  valueType: 'number' | 'string' | 'boolean';
}

export default function Settings() {
  const [configs, setConfigs] = useState<ConfigItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [restarting, setRestarting] = useState(false);
  const [editedValues, setEditedValues] = useState<Record<string, any>>({});
  const { showToast } = useToast();
  const { isAdmin } = useAuth();
  const [confirmDialog, setConfirmDialog] = useState<{ open: boolean; title: string; message: string; variant: 'danger' | 'warning' | 'info'; confirmText: string; onConfirm: () => void }>({ open: false, title: '', message: '', variant: 'warning', confirmText: '确定', onConfirm: () => {} });

  useEffect(() => {
    loadConfigs();
  }, []);

  const restartSystem = () => {
    setConfirmDialog({
      open: true,
      title: '重启系统',
      message: '确定要重启系统吗？重启期间服务将暂时不可用。',
      variant: 'warning',
      confirmText: '重启',
      onConfirm: async () => {
        setRestarting(true);
        try {
          const response = await systemService.restart();
          if (response.success) {
            showToast('系统正在重启，请稍候...', 'success');
            setTimeout(() => {
              window.location.reload();
            }, 5000);
          } else {
            showToast(response.message || '重启失败', 'error');
            setRestarting(false);
          }
        } catch (error) {
          showToast('网络错误，请稍后重试', 'error');
          setRestarting(false);
        }
      },
    });
  };

  const loadConfigs = async () => {
    try {
      const response = await systemService.getConfigs();
      if (response.success && response.data) {
        setConfigs(response.data.configs);
        const initialValues: Record<string, any> = {};
        response.data.configs.forEach(config => {
          initialValues[config.key] = config.value;
        });
        setEditedValues(initialValues);
      } else {
        showToast(response.message || '无法加载系统配置', 'error');
      }
    } catch (error) {
      showToast('网络错误，请稍后重试', 'error');
    } finally {
      setLoading(false);
    }
  };

  const handleValueChange = (key: string, value: any, valueType: string) => {
    let parsedValue = value;

    if (valueType === 'number') {
      parsedValue = value === '' ? 0 : Number(value);
    } else if (valueType === 'boolean') {
      parsedValue = value === 'true' || value === true;
    }

    setEditedValues(prev => ({
      ...prev,
      [key]: parsedValue,
    }));
  };

  // 比较两个值是否相等（处理类型转换）
  const valuesEqual = (a: any, b: any, valueType: string): boolean => {
    if (valueType === 'boolean') {
      const aBool = a === true || a === 'true';
      const bBool = b === true || b === 'true';
      return aBool === bBool;
    }
    if (valueType === 'number') {
      return Number(a) === Number(b);
    }
    return a === b;
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      const updates = configs
        .filter(config => !valuesEqual(editedValues[config.key], config.value, config.valueType))
        .map(config => ({
          key: config.key,
          value: editedValues[config.key],
        }));

      if (updates.length === 0) {
        showToast('没有需要保存的更改', 'success');
        setSaving(false);
        return;
      }

      const protocolChanged = updates.some(u => u.key === 'use_kcp');

      const response = await systemService.batchUpdateConfigs(updates);

      if (response.success) {
        showToast(`已成功更新 ${updates.length} 个配置项`, 'success');
        loadConfigs();

        if (protocolChanged && isAdmin) {
          setTimeout(() => {
            setConfirmDialog({
              open: true,
              title: '重启系统',
              message: '协议设置已更改，需要重启系统才能生效。是否立即重启？',
              variant: 'warning',
              confirmText: '立即重启',
              onConfirm: async () => {
                setRestarting(true);
                try {
                  const resp = await systemService.restart();
                  if (resp.success) {
                    showToast('系统正在重启，请稍候...', 'success');
                    setTimeout(() => window.location.reload(), 5000);
                  } else {
                    showToast(resp.message || '重启失败', 'error');
                    setRestarting(false);
                  }
                } catch {
                  showToast('网络错误，请稍后重试', 'error');
                  setRestarting(false);
                }
              },
            });
          }, 500);
        }
      } else {
        showToast(response.message || '无法保存配置', 'error');
      }
    } catch (error) {
      showToast('网络错误，请稍后重试', 'error');
    } finally {
      setSaving(false);
    }
  };

  const handleReset = () => {
    const initialValues: Record<string, any> = {};
    configs.forEach(config => {
      initialValues[config.key] = config.value;
    });
    setEditedValues(initialValues);
    showToast('所有更改已撤销', 'success');
  };

  const hasChanges = configs.some(config => !valuesEqual(editedValues[config.key], config.value, config.valueType));

  const renderConfigInput = (config: ConfigItem) => {
    const value = editedValues[config.key];
    const inputClassName = "w-full max-w-xs px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500";

    switch (config.valueType) {
      case 'number':
        return (
          <input
            type="number"
            value={value ?? 0}
            onChange={(e) => handleValueChange(config.key, e.target.value, config.valueType)}
            className={inputClassName}
          />
        );

      case 'boolean':
        return (
          <select
            value={value === true || value === 'true' ? 'true' : 'false'}
            onChange={(e) => handleValueChange(config.key, e.target.value, config.valueType)}
            className={inputClassName}
          >
            <option value="true">启用</option>
            <option value="false">禁用</option>
          </select>
        );

      case 'string':
      default:
        return (
          <input
            type="text"
            value={value || ''}
            onChange={(e) => handleValueChange(config.key, e.target.value, config.valueType)}
            className={inputClassName}
          />
        );
    }
  };

  const getConfigCategory = (key: string): string => {
    if (key === 'use_kcp') {
      return '传输协议';
    }
    if (key.startsWith('kcp_')) {
      return 'KCP 协议参数';
    }
    if (key.includes('timeout') || key.includes('interval') || key.includes('streams')) {
      return 'QUIC 连接配置';
    }
    if (key.includes('registration') || key.includes('name')) {
      return '系统配置';
    }
    return '其他配置';
  };

  const categoryOrder = ['传输协议', 'KCP 协议参数', 'QUIC 连接配置', '系统配置', '其他配置'];

  const groupedConfigs = configs.reduce((acc, config) => {
    const category = getConfigCategory(config.key);
    if (!acc[category]) {
      acc[category] = [];
    }
    acc[category].push(config);
    return acc;
  }, {} as Record<string, ConfigItem[]>);

  const sortedCategories = Object.keys(groupedConfigs).sort((a, b) => {
    return categoryOrder.indexOf(a) - categoryOrder.indexOf(b);
  });

  if (loading) {
    return (
      <div className="space-y-6">
        <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-6">
          <div className="flex items-center justify-between">
            <div className="space-y-2">
              <SkeletonBlock className="h-8 w-32" />
              <SkeletonBlock className="h-4 w-48" />
            </div>
            <div className="flex gap-3">
              <SkeletonBlock className="h-10 w-28 rounded-xl" />
              <SkeletonBlock className="h-10 w-24 rounded-xl" />
              <SkeletonBlock className="h-10 w-28 rounded-xl" />
            </div>
          </div>
        </div>
        <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-6">
          <SkeletonBlock className="h-6 w-24 mb-4" />
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <SkeletonBlock className="h-24 rounded-lg" />
            <SkeletonBlock className="h-24 rounded-lg" />
          </div>
        </div>
        {Array.from({ length: 2 }).map((_, i) => (
          <div key={i} className="bg-white rounded-2xl shadow-sm border border-gray-100 p-6 space-y-6">
            <SkeletonBlock className="h-6 w-32 mb-2" />
            {Array.from({ length: 3 }).map((_, j) => (
              <div key={j} className="space-y-2">
                <SkeletonBlock className="h-4 w-40" />
                <SkeletonBlock className="h-10 w-64 rounded-md" />
              </div>
            ))}
          </div>
        ))}
      </div>
    );
  }

  const useKcp = editedValues['use_kcp'] === true || editedValues['use_kcp'] === 'true';

  // 根据协议选择过滤要显示的分类
  const visibleCategories = sortedCategories.filter(cat => {
    if (cat === '传输协议') return false;
    if (cat === 'KCP 协议参数') return useKcp;
    if (cat === 'QUIC 连接配置') return !useKcp;
    return true;
  });

  return (
    <div className="space-y-6">
      {/* 页面标题和操作按钮 */}
      <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-gray-900">系统配置</h1>
            <p className="text-gray-600 mt-1">管理 RFRP 系统的全局配置项</p>
          </div>
          <div className="flex gap-3">
            {isAdmin && (
              <button
                onClick={restartSystem}
                disabled={restarting || saving}
                className="px-4 py-2 text-white bg-red-600 rounded-md hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {restarting ? '🔄 重启中...' : '🔄 重启系统'}
              </button>
            )}
            <button
              onClick={handleReset}
              disabled={!hasChanges || saving}
              className="px-4 py-2 text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              🔄 重置
            </button>
            <button
              onClick={handleSave}
              disabled={!hasChanges || saving}
              className="px-4 py-2 text-white bg-blue-600 rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {saving ? '💾 保存中...' : '💾 保存更改'}
            </button>
          </div>
        </div>
      </div>

      {/* 协议选择卡片 */}
      <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-6">
        <h2 className="text-xl font-semibold text-gray-900 mb-4">传输协议</h2>
        <p className="text-sm text-gray-600 mb-4">
          选择服务端使用的传输协议（修改后需重启服务端生效）
        </p>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {/* QUIC 选项卡片 */}
          <div
            onClick={() => handleValueChange('use_kcp', false, 'boolean')}
            className={`border-2 rounded-lg p-4 cursor-pointer transition-all ${
              !useKcp
                ? 'border-blue-500 bg-blue-50'
                : 'border-gray-200 bg-white hover:border-gray-300'
            }`}
          >
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-3">
                <div className={`w-4 h-4 rounded-full border-2 flex items-center justify-center ${
                  !useKcp ? 'border-blue-500' : 'border-gray-300'
                }`}>
                  {!useKcp && <div className="w-2 h-2 rounded-full bg-blue-500"></div>}
                </div>
                <span className="font-medium text-gray-900">QUIC 协议</span>
              </div>
              {!useKcp && (
                <span className="px-2 py-1 text-xs font-medium rounded-full bg-blue-100 text-blue-800">
                  当前使用
                </span>
              )}
            </div>
            <p className="text-sm text-gray-600 ml-7">
              基于 UDP 的安全传输协议，内置 TLS 加密，适合大多数场景
            </p>
          </div>

          {/* KCP 选项卡片 */}
          <div
            onClick={() => handleValueChange('use_kcp', true, 'boolean')}
            className={`border-2 rounded-lg p-4 cursor-pointer transition-all ${
              useKcp
                ? 'border-green-500 bg-green-50'
                : 'border-gray-200 bg-white hover:border-gray-300'
            }`}
          >
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-3">
                <div className={`w-4 h-4 rounded-full border-2 flex items-center justify-center ${
                  useKcp ? 'border-green-500' : 'border-gray-300'
                }`}>
                  {useKcp && <div className="w-2 h-2 rounded-full bg-green-500"></div>}
                </div>
                <span className="font-medium text-gray-900">KCP 协议</span>
              </div>
              {useKcp && (
                <span className="px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-800">
                  当前使用
                </span>
              )}
            </div>
            <p className="text-sm text-gray-600 ml-7">
              快速可靠的 UDP 传输协议，适合高延迟或不稳定的网络环境
            </p>
          </div>
        </div>
      </div>

      {/* 配置项分组 - 只显示当前协议相关的配置 */}
      {visibleCategories.map((category) => {
        const categoryConfigs = groupedConfigs[category];

        return (
          <div key={category} className="bg-white rounded-2xl shadow-sm border border-gray-100 p-6">
            <div className="mb-6">
              <h2 className="text-xl font-semibold text-gray-900">{category}</h2>
              <p className="text-sm text-gray-600 mt-1">
                {category === 'KCP 协议参数' && 'KCP 协议的详细参数配置（修改后需重启服务端生效）'}
                {category === 'QUIC 连接配置' && 'QUIC 协议相关的连接参数（修改后需客户端重新连接生效）'}
                {category === '系统配置' && '系统级别的基本配置'}
              </p>
            </div>

            <div className="space-y-6">
              {categoryConfigs.map((config) => (
                <div key={config.key} className="border-b border-gray-200 pb-6 last:border-b-0 last:pb-0">
                  <label className="block text-base font-medium text-gray-700 mb-2">
                    {config.description}
                  </label>
                  <div className="flex items-center gap-4">
                    {renderConfigInput(config)}
                    <span className="text-sm text-gray-500">
                      {config.valueType === 'number' && (
                        config.key === 'kcp_interval'
                          ? '毫秒'
                          : (config.key.includes('interval') || config.key.includes('timeout'))
                            ? '秒'
                            : ''
                      )}
                    </span>
                  </div>
                  {config.key === 'health_check_interval' && (
                    <p className="text-sm text-gray-500 mt-2">
                      💡 服务端检查客户端连接状态的间隔时间
                    </p>
                  )}
                  {config.key === 'idle_timeout' && (
                    <p className="text-sm text-gray-500 mt-2">
                      💡 无数据传输时连接的超时时间
                    </p>
                  )}
                  {config.key === 'keep_alive_interval' && (
                    <p className="text-sm text-gray-500 mt-2">
                      💡 心跳包发送间隔，用于保持连接活跃
                    </p>
                  )}
                  {config.key === 'max_concurrent_streams' && (
                    <p className="text-sm text-gray-500 mt-2">
                      💡 单个客户端连接允许的最大并发流数量
                    </p>
                  )}
                  {config.key === 'kcp_nodelay' && (
                    <p className="text-sm text-gray-500 mt-2">
                      💡 启用后禁用 Nagle 算法，降低延迟
                    </p>
                  )}
                  {config.key === 'kcp_interval' && (
                    <p className="text-sm text-gray-500 mt-2">
                      💡 内部更新时钟间隔，值越小延迟越低，建议 10-40
                    </p>
                  )}
                  {config.key === 'kcp_resend' && (
                    <p className="text-sm text-gray-500 mt-2">
                      💡 快速重传触发次数，0 表示禁用，建议值 2
                    </p>
                  )}
                  {config.key === 'kcp_nc' && (
                    <p className="text-sm text-gray-500 mt-2">
                      💡 关闭拥塞控制，发送速度更快
                    </p>
                  )}
                </div>
              ))}
            </div>
          </div>
        );
      })}

      {/* 未保存提示 */}
      {hasChanges && (
        <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
          <div className="flex items-center gap-2 text-yellow-800">
            <div className="h-2 w-2 rounded-full bg-yellow-500 animate-pulse" />
            <span className="text-sm font-medium">你有未保存的更改（修改后需要重启服务端生效）</span>
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
          setConfirmDialog(prev => ({ ...prev, open: false }));
        }}
        onCancel={() => setConfirmDialog(prev => ({ ...prev, open: false }))}
      />
    </div>
  );
}
