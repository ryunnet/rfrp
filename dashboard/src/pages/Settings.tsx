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

const configHints: Record<string, string> = {
  health_check_interval: '服务端检查客户端连接状态的间隔时间',
  idle_timeout: '无数据传输时连接的超时时间',
  keep_alive_interval: '心跳包发送间隔，用于保持连接活跃',
  max_concurrent_streams: '单个客户端连接允许的最大并发流数量',
};

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

      const response = await systemService.batchUpdateConfigs(updates);

      if (response.success) {
        showToast(`已成功更新 ${updates.length} 个配置项`, 'success');
        loadConfigs();
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
    const inputClassName = "w-full max-w-xs px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white";

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

  if (loading) {
    return (
      <div className="space-y-6">
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
        <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-6">
          <SkeletonBlock className="h-6 w-24 mb-6" />
          <div className="space-y-6">
            {Array.from({ length: 4 }).map((_, i) => (
              <div key={i} className="space-y-2">
                <SkeletonBlock className="h-4 w-40" />
                <SkeletonBlock className="h-12 w-80 rounded-xl" />
                <SkeletonBlock className="h-3 w-56" />
              </div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* 页面标题和操作按钮 */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">系统配置</h2>
          <p className="mt-1 text-sm text-gray-500">管理 RFRP 系统的连接配置参数</p>
        </div>
        <div className="flex gap-3">
          {isAdmin && (
            <button
              onClick={restartSystem}
              disabled={restarting || saving}
              className="inline-flex items-center gap-2 px-4 py-2.5 bg-gradient-to-r from-red-500 to-rose-600 text-white text-sm font-medium rounded-xl hover:from-red-600 hover:to-rose-700 focus:outline-none focus:ring-2 focus:ring-red-500/40 shadow-lg shadow-red-500/25 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182" />
              </svg>
              {restarting ? '重启中...' : '重启系统'}
            </button>
          )}
          <button
            onClick={handleReset}
            disabled={!hasChanges || saving}
            className="inline-flex items-center gap-2 px-4 py-2.5 bg-gray-100 text-gray-700 text-sm font-medium rounded-xl hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-gray-500/20 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" d="M9 15 3 9m0 0 6-6M3 9h12a6 6 0 0 1 0 12h-3" />
            </svg>
            重置
          </button>
          <button
            onClick={handleSave}
            disabled={!hasChanges || saving}
            className="inline-flex items-center gap-2 px-5 py-2.5 bg-gradient-to-r from-blue-600 to-indigo-600 text-white text-sm font-medium rounded-xl hover:from-blue-700 hover:to-indigo-700 focus:outline-none focus:ring-2 focus:ring-blue-500/40 shadow-lg shadow-blue-500/25 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" d="M9 3.75H6.912a2.25 2.25 0 0 0-2.15 1.588L2.35 13.177a2.25 2.25 0 0 0-.1.661V18a2.25 2.25 0 0 0 2.25 2.25h15A2.25 2.25 0 0 0 21.75 18v-4.162c0-.224-.034-.447-.1-.661L19.24 5.338a2.25 2.25 0 0 0-2.15-1.588H15M2.25 13.5h3.86a2.25 2.25 0 0 1 2.012 1.244l.256.512a2.25 2.25 0 0 0 2.013 1.244h3.218a2.25 2.25 0 0 0 2.013-1.244l.256-.512a2.25 2.25 0 0 1 2.013-1.244h3.859M12 3v8.25m0 0-3-3m3 3 3-3" />
            </svg>
            {saving ? '保存中...' : '保存更改'}
          </button>
        </div>
      </div>

      {/* 连接配置 */}
      <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-6">
        <div className="flex items-center gap-3 mb-6">
          <div className="w-10 h-10 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-xl flex items-center justify-center">
            <svg className="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" d="M10.343 3.94c.09-.542.56-.94 1.11-.94h1.093c.55 0 1.02.398 1.11.94l.149.894c.07.424.384.764.78.93.398.164.855.142 1.205-.108l.737-.527a1.125 1.125 0 0 1 1.45.12l.773.774c.39.389.44 1.002.12 1.45l-.527.737c-.25.35-.272.806-.107 1.204.165.397.505.71.93.78l.893.15c.543.09.94.559.94 1.109v1.094c0 .55-.397 1.02-.94 1.11l-.894.149c-.424.07-.764.383-.929.78-.165.398-.143.854.107 1.204l.527.738c.32.447.269 1.06-.12 1.45l-.774.773a1.125 1.125 0 0 1-1.449.12l-.738-.527c-.35-.25-.806-.272-1.204-.107-.397.165-.71.505-.78.929l-.15.894c-.09.542-.56.94-1.11.94h-1.094c-.55 0-1.019-.398-1.11-.94l-.148-.894c-.071-.424-.384-.764-.781-.93-.398-.164-.854-.142-1.204.108l-.738.527c-.447.32-1.06.269-1.45-.12l-.773-.774a1.125 1.125 0 0 1-.12-1.45l.527-.737c.25-.35.272-.806.108-1.204-.165-.397-.506-.71-.93-.78l-.894-.15c-.542-.09-.94-.56-.94-1.109v-1.094c0-.55.398-1.02.94-1.11l.894-.149c.424-.07.765-.383.93-.78.165-.398.143-.854-.108-1.204l-.526-.738a1.125 1.125 0 0 1 .12-1.45l.773-.773a1.125 1.125 0 0 1 1.45-.12l.737.527c.35.25.807.272 1.204.107.397-.165.71-.505.78-.929l.15-.894Z" />
              <path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" />
            </svg>
          </div>
          <div>
            <h3 className="text-lg font-bold text-gray-900">连接配置</h3>
            <p className="text-sm text-gray-500">QUIC 协议相关的连接参数（修改后需客户端重新连接生效）</p>
          </div>
        </div>

        <div className="space-y-6">
          {configs.map((config) => (
            <div key={config.key} className="border-b border-gray-100 pb-6 last:border-b-0 last:pb-0">
              <label className="block text-sm font-medium text-gray-700 mb-1.5">
                {config.description}
              </label>
              <div className="flex items-center gap-4">
                {renderConfigInput(config)}
                <span className="text-sm text-gray-500">
                  {config.valueType === 'number' && (
                    (config.key.includes('interval') || config.key.includes('timeout'))
                      ? '秒'
                      : ''
                  )}
                </span>
              </div>
              {configHints[config.key] && (
                <div className="flex items-center gap-1.5 mt-2">
                  <svg className="w-3.5 h-3.5 text-gray-400 flex-shrink-0" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" d="m11.25 11.25.041-.02a.75.75 0 0 1 1.063.852l-.708 2.836a.75.75 0 0 0 1.063.853l.041-.021M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9-3.75h.008v.008H12V8.25Z" />
                  </svg>
                  <p className="text-sm text-gray-500">{configHints[config.key]}</p>
                </div>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* 未保存提示 */}
      {hasChanges && (
        <div className="bg-amber-50 border border-amber-200 rounded-xl p-4">
          <div className="flex items-center gap-2 text-amber-800">
            <svg className="w-4 h-4 flex-shrink-0" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z" />
            </svg>
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
