import { useState, useEffect } from 'react';

interface ConfigItem {
  id: number;
  key: string;
  value: number | string | boolean;
  description: string;
  valueType: 'number' | 'string' | 'boolean';
}

interface ConfigListResponse {
  configs: ConfigItem[];
}

interface ApiResponse<T> {
  success: boolean;
  data?: T;
  message: string;
}

export default function Settings() {
  const [configs, setConfigs] = useState<ConfigItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [editedValues, setEditedValues] = useState<Record<string, any>>({});
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' } | null>(null);

  useEffect(() => {
    loadConfigs();
  }, []);

  useEffect(() => {
    if (toast) {
      const timer = setTimeout(() => setToast(null), 3000);
      return () => clearTimeout(timer);
    }
  }, [toast]);

  const showToast = (message: string, type: 'success' | 'error') => {
    setToast({ message, type });
  };

  const loadConfigs = async () => {
    try {
      const token = localStorage.getItem('token');
      const response = await fetch('/api/system/configs', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      const data: ApiResponse<ConfigListResponse> = await response.json();

      if (data.success && data.data) {
        setConfigs(data.data.configs);
        // 初始化编辑值
        const initialValues: Record<string, any> = {};
        data.data.configs.forEach(config => {
          initialValues[config.key] = config.value;
        });
        setEditedValues(initialValues);
      } else {
        showToast(data.message || '无法加载系统配置', 'error');
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

  const handleSave = async () => {
    setSaving(true);
    try {
      const token = localStorage.getItem('token');

      // 准备批量更新的数据
      const updates = configs
        .filter(config => editedValues[config.key] !== config.value)
        .map(config => ({
          key: config.key,
          value: editedValues[config.key],
        }));

      if (updates.length === 0) {
        showToast('没有需要保存的更改', 'success');
        setSaving(false);
        return;
      }

      const response = await fetch('/api/system/configs/batch', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ configs: updates }),
      });

      const data: ApiResponse<ConfigListResponse> = await response.json();

      if (data.success) {
        showToast(`已成功更新 ${updates.length} 个配置项`, 'success');
        loadConfigs();
      } else {
        showToast(data.message || '无法保存配置', 'error');
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

  const hasChanges = configs.some(config => editedValues[config.key] !== config.value);

  const renderConfigInput = (config: ConfigItem) => {
    const value = editedValues[config.key];

    const inputClassName = "w-full max-w-xs px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500";

    switch (config.valueType) {
      case 'number':
        return (
          <input
            type="number"
            value={value || 0}
            onChange={(e) => handleValueChange(config.key, e.target.value, config.valueType)}
            className={inputClassName}
          />
        );

      case 'boolean':
        return (
          <select
            value={value ? 'true' : 'false'}
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
    if (key.includes('timeout') || key.includes('interval') || key.includes('streams')) {
      return 'QUIC 连接配置';
    }
    if (key.includes('registration') || key.includes('name')) {
      return '系统配置';
    }
    return '其他配置';
  };

  const groupedConfigs = configs.reduce((acc, config) => {
    const category = getConfigCategory(config.key);
    if (!acc[category]) {
      acc[category] = [];
    }
    acc[category].push(config);
    return acc;
  }, {} as Record<string, ConfigItem[]>);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Toast 通知 */}
      {toast && (
        <div className={`fixed top-4 right-4 px-6 py-3 rounded-lg shadow-lg ${
          toast.type === 'success' ? 'bg-green-500' : 'bg-red-500'
        } text-white z-50 animate-fade-in`}>
          {toast.message}
        </div>
      )}

      {/* 页面标题和操作按钮 */}
      <div className="bg-white shadow rounded-lg p-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-gray-900">系统配置</h1>
            <p className="text-gray-600 mt-1">管理 RFRP 系统的全局配置项</p>
          </div>
          <div className="flex gap-3">
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

      {/* 配置项分组 */}
      {Object.entries(groupedConfigs).map(([category, categoryConfigs]) => (
        <div key={category} className="bg-white shadow rounded-lg p-6">
          <div className="mb-6">
            <h2 className="text-xl font-semibold text-gray-900">{category}</h2>
            <p className="text-sm text-gray-600 mt-1">
              {category === 'QUIC 连接配置' && '配置 QUIC 协议相关的连接参数（修改后需要客户端重新连接才能生效）'}
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
                      config.key.includes('interval') || config.key.includes('timeout')
                        ? '秒'
                        : ''
                    )}
                  </span>
                </div>
                {config.key === 'health_check_interval' && (
                  <p className="text-sm text-gray-500 mt-2">
                    💡 服务端检查客户端连接状态的间隔时间，值越小检测越及时但消耗越高
                  </p>
                )}
                {config.key === 'idle_timeout' && (
                  <p className="text-sm text-gray-500 mt-2">
                    💡 无数据传输时连接的超时时间，超时后自动断开连接
                  </p>
                )}
                {config.key === 'keep_alive_interval' && (
                  <p className="text-sm text-gray-500 mt-2">
                    💡 QUIC 协议层面的心跳包发送间隔，用于保持连接活跃
                  </p>
                )}
                {config.key === 'max_concurrent_streams' && (
                  <p className="text-sm text-gray-500 mt-2">
                    💡 单个客户端连接允许的最大并发流数量
                  </p>
                )}
              </div>
            ))}
          </div>
        </div>
      ))}

      {/* 未保存提示 */}
      {hasChanges && (
        <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
          <div className="flex items-center gap-2 text-yellow-800">
            <div className="h-2 w-2 rounded-full bg-yellow-500 animate-pulse" />
            <span className="text-sm font-medium">你有未保存的更改</span>
          </div>
        </div>
      )}
    </div>
  );
}
