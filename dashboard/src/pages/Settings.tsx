import { useState, useEffect } from 'react';
import { useAuth } from '../contexts/AuthContext';
import { systemService } from '../lib/services';
import { useToast } from '../contexts/ToastContext';
import ConfirmDialog from '../components/ConfirmDialog';
import SkeletonBlock from '../components/Skeleton';
import { Settings as SettingsIcon, Save, RotateCcw, Power, Info, AlertCircle } from 'lucide-react';

interface ConfigItem {
  id: number;
  key: string;
  value: number | string | boolean;
  description: string;
  valueType: 'number' | 'string' | 'boolean';
}

const configHints: Record<string, string> = {
  web_port: 'Web 管理界面的访问端口',
  internal_port: 'Node 和 Client 连接到 Controller 的 gRPC 端口',
  jwt_expiration_hours: 'JWT 令牌的有效期，过期后需要重新登录',
  db_path: '数据库文件的存储路径',
  grpc_tls_enabled: '启用后 gRPC 连接将使用 TLS 加密，可避免 GFW 干扰',
  grpc_tls_cert_path: 'TLS 证书文件的绝对路径（PEM 格式）',
  grpc_tls_key_path: 'TLS 私钥文件的绝对路径（PEM 格式）',
  grpc_tls_cert_content: 'TLS 证书内容（PEM 格式，可直接上传证书文件）',
  grpc_tls_key_content: 'TLS 私钥内容（PEM 格式，可直接上传私钥文件）',
  grpc_domain: 'gRPC 服务器域名（可选，用于 SNI）',
  web_tls_enabled: '启用后 Web 管理界面将使用 HTTPS 加密访问',
  web_tls_cert_path: 'Web TLS 证书文件的绝对路径（PEM 格式）',
  web_tls_key_path: 'Web TLS 私钥文件的绝对路径（PEM 格式）',
  web_tls_cert_content: 'Web TLS 证书内容（PEM 格式，可直接上传证书文件）',
  web_tls_key_content: 'Web TLS 私钥内容（PEM 格式，可直接上传私钥文件）',
};

// gRPC TLS 证书相关的 key
const GRPC_TLS_CERT_PATH_KEYS = ['grpc_tls_cert_path', 'grpc_tls_key_path'];
const GRPC_TLS_CERT_CONTENT_KEYS = ['grpc_tls_cert_content', 'grpc_tls_key_content'];
const GRPC_TLS_CERT_ALL_KEYS = [...GRPC_TLS_CERT_PATH_KEYS, ...GRPC_TLS_CERT_CONTENT_KEYS];

// Web TLS 证书相关的 key
const WEB_TLS_CERT_PATH_KEYS = ['web_tls_cert_path', 'web_tls_key_path'];
const WEB_TLS_CERT_CONTENT_KEYS = ['web_tls_cert_content', 'web_tls_key_content'];
const WEB_TLS_CERT_ALL_KEYS = [...WEB_TLS_CERT_PATH_KEYS, ...WEB_TLS_CERT_CONTENT_KEYS];

// 所有 TLS 证书相关的 key
const ALL_TLS_CERT_KEYS = [...GRPC_TLS_CERT_ALL_KEYS, ...WEB_TLS_CERT_ALL_KEYS];

export default function Settings() {
  const [configs, setConfigs] = useState<ConfigItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [restarting, setRestarting] = useState(false);
  const [editedValues, setEditedValues] = useState<Record<string, any>>({});
  const [grpcCertMode, setGrpcCertMode] = useState<'upload' | 'path'>('upload');
  const [webCertMode, setWebCertMode] = useState<'upload' | 'path'>('upload');
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
        // 根据已有数据自动判断 gRPC 证书配置模式
        const hasGrpcPath = initialValues['grpc_tls_cert_path'] && initialValues['grpc_tls_cert_path'] !== '';
        const hasGrpcContent = initialValues['grpc_tls_cert_content'] && initialValues['grpc_tls_cert_content'] !== '';
        setGrpcCertMode(hasGrpcPath && !hasGrpcContent ? 'path' : 'upload');
        // 根据已有数据自动判断 Web 证书配置模式
        const hasWebPath = initialValues['web_tls_cert_path'] && initialValues['web_tls_cert_path'] !== '';
        const hasWebContent = initialValues['web_tls_cert_content'] && initialValues['web_tls_cert_content'] !== '';
        setWebCertMode(hasWebPath && !hasWebContent ? 'path' : 'upload');
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

    // 文件上传组件（用于证书内容）
    if (config.key === 'grpc_tls_cert_content' || config.key === 'grpc_tls_key_content' ||
        config.key === 'web_tls_cert_content' || config.key === 'web_tls_key_content') {
      const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;

        try {
          const text = await file.text();
          // 将文件内容转换为 base64
          const base64 = btoa(text);
          handleValueChange(config.key, base64, config.valueType);
          const certType = config.key.includes('cert_content') ? '证书' : '私钥';
          showToast(`${certType}文件已加载`, 'success');
        } catch (error) {
          showToast('文件读取失败', 'error');
        }
      };

      const hasContent = value && value !== '';

      return (
        <div className="space-y-2">
          <div className="flex items-center gap-3">
            <label className="inline-flex items-center gap-2 px-4 py-2.5 bg-gradient-to-r from-blue-600 to-indigo-600 text-white text-sm font-medium rounded-xl hover:from-blue-700 hover:to-indigo-700 focus:outline-none focus:ring-2 focus:ring-blue-500/40 shadow-lg shadow-blue-500/25 transition-all duration-200 cursor-pointer">
              <input
                type="file"
                accept=".pem,.crt,.key"
                onChange={handleFileUpload}
                className="hidden"
              />
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" d="M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5m-13.5-9L12 3m0 0 4.5 4.5M12 3v13.5" />
              </svg>
              上传文件
            </label>
            {hasContent && (
              <span className="text-sm text-green-600 font-medium">✓ 已上传</span>
            )}
          </div>
          {hasContent && (
            <button
              onClick={() => handleValueChange(config.key, '', config.valueType)}
              className="text-sm text-red-600 hover:text-red-700 font-medium"
            >
              清除内容
            </button>
          )}
        </div>
      );
    }

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
          <p className="mt-1 text-sm text-gray-500">管理 RFRP Controller 的运行参数</p>
        </div>
        <div className="flex gap-3">
          {isAdmin && (
            <button
              onClick={restartSystem}
              disabled={restarting || saving}
              className="inline-flex items-center gap-2 px-4 py-2.5 bg-gradient-to-r from-red-500 to-rose-600 text-white text-sm font-medium rounded-xl hover:from-red-600 hover:to-rose-700 focus:outline-none focus:ring-2 focus:ring-red-500/40 shadow-lg shadow-red-500/25 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <Power className="w-4 h-4" />
              {restarting ? '重启中...' : '重启系统'}
            </button>
          )}
          <button
            onClick={handleReset}
            disabled={!hasChanges || saving}
            className="inline-flex items-center gap-2 px-4 py-2.5 bg-gray-100 text-gray-700 text-sm font-medium rounded-xl hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-gray-500/20 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <RotateCcw className="w-4 h-4" />
            重置
          </button>
          <button
            onClick={handleSave}
            disabled={!hasChanges || saving}
            className="inline-flex items-center gap-2 px-5 py-2.5 bg-gradient-to-r from-blue-600 to-indigo-600 text-white text-sm font-medium rounded-xl hover:from-blue-700 hover:to-indigo-700 focus:outline-none focus:ring-2 focus:ring-blue-500/40 shadow-lg shadow-blue-500/25 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Save className="w-4 h-4" />
            {saving ? '保存中...' : '保存更改'}
          </button>
        </div>
      </div>

      {/* 连接配置 */}
      <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-6">
        <div className="flex items-center gap-3 mb-6">
          <div className="w-10 h-10 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-xl flex items-center justify-center">
            <SettingsIcon className="w-5 h-5 text-white" />
          </div>
          <div>
            <h3 className="text-lg font-bold text-gray-900">系统配置</h3>
            <p className="text-sm text-gray-500">Controller 运行参数（修改后需重启系统生效）</p>
          </div>
        </div>

        <div className="space-y-6">
          {configs.filter(c => !ALL_TLS_CERT_KEYS.includes(c.key)).map((config) => (
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
                  <Info className="w-3.5 h-3.5 text-gray-400 flex-shrink-0" />
                  <p className="text-sm text-gray-500">{configHints[config.key]}</p>
                </div>
              )}
            </div>
          ))}

          {/* gRPC TLS 证书配置区域 */}
          {configs.some(c => GRPC_TLS_CERT_ALL_KEYS.includes(c.key)) && (
            <div className="border-t border-gray-200 pt-6">
              <div className="flex items-center justify-between mb-4">
                <span className="text-sm font-medium text-gray-700">gRPC TLS 证书配置方式</span>
                <div className="inline-flex rounded-lg border border-gray-200 p-0.5 bg-gray-50">
                  <button
                    type="button"
                    onClick={() => {
                      setGrpcCertMode('upload');
                      GRPC_TLS_CERT_PATH_KEYS.forEach(k => handleValueChange(k, '', 'string'));
                    }}
                    className={`px-3 py-1.5 text-sm font-medium rounded-md transition-all ${grpcCertMode === 'upload' ? 'bg-white text-blue-600 shadow-sm' : 'text-gray-500 hover:text-gray-700'}`}
                  >
                    上传文件
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      setGrpcCertMode('path');
                      GRPC_TLS_CERT_CONTENT_KEYS.forEach(k => handleValueChange(k, '', 'string'));
                    }}
                    className={`px-3 py-1.5 text-sm font-medium rounded-md transition-all ${grpcCertMode === 'path' ? 'bg-white text-blue-600 shadow-sm' : 'text-gray-500 hover:text-gray-700'}`}
                  >
                    指定路径
                  </button>
                </div>
              </div>

              {configs
                .filter(c => grpcCertMode === 'upload' ? GRPC_TLS_CERT_CONTENT_KEYS.includes(c.key) : GRPC_TLS_CERT_PATH_KEYS.includes(c.key))
                .map((config) => (
                  <div key={config.key} className="border-b border-gray-100 pb-6 mb-6 last:border-b-0 last:pb-0 last:mb-0">
                    <label className="block text-sm font-medium text-gray-700 mb-1.5">
                      {config.description}
                    </label>
                    <div className="flex items-center gap-4">
                      {renderConfigInput(config)}
                    </div>
                    {configHints[config.key] && (
                      <div className="flex items-center gap-1.5 mt-2">
                        <Info className="w-3.5 h-3.5 text-gray-400 flex-shrink-0" />
                        <p className="text-sm text-gray-500">{configHints[config.key]}</p>
                      </div>
                    )}
                  </div>
                ))
              }
            </div>
          )}

          {/* Web TLS 证书配置区域 */}
          {configs.some(c => WEB_TLS_CERT_ALL_KEYS.includes(c.key)) && (
            <div className="border-t border-gray-200 pt-6">
              <div className="flex items-center justify-between mb-4">
                <span className="text-sm font-medium text-gray-700">Web TLS 证书配置方式</span>
                <div className="inline-flex rounded-lg border border-gray-200 p-0.5 bg-gray-50">
                  <button
                    type="button"
                    onClick={() => {
                      setWebCertMode('upload');
                      WEB_TLS_CERT_PATH_KEYS.forEach(k => handleValueChange(k, '', 'string'));
                    }}
                    className={`px-3 py-1.5 text-sm font-medium rounded-md transition-all ${webCertMode === 'upload' ? 'bg-white text-blue-600 shadow-sm' : 'text-gray-500 hover:text-gray-700'}`}
                  >
                    上传文件
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      setWebCertMode('path');
                      WEB_TLS_CERT_CONTENT_KEYS.forEach(k => handleValueChange(k, '', 'string'));
                    }}
                    className={`px-3 py-1.5 text-sm font-medium rounded-md transition-all ${webCertMode === 'path' ? 'bg-white text-blue-600 shadow-sm' : 'text-gray-500 hover:text-gray-700'}`}
                  >
                    指定路径
                  </button>
                </div>
              </div>

              {configs
                .filter(c => webCertMode === 'upload' ? WEB_TLS_CERT_CONTENT_KEYS.includes(c.key) : WEB_TLS_CERT_PATH_KEYS.includes(c.key))
                .map((config) => (
                  <div key={config.key} className="border-b border-gray-100 pb-6 mb-6 last:border-b-0 last:pb-0 last:mb-0">
                    <label className="block text-sm font-medium text-gray-700 mb-1.5">
                      {config.description}
                    </label>
                    <div className="flex items-center gap-4">
                      {renderConfigInput(config)}
                    </div>
                    {configHints[config.key] && (
                      <div className="flex items-center gap-1.5 mt-2">
                        <Info className="w-3.5 h-3.5 text-gray-400 flex-shrink-0" />
                        <p className="text-sm text-gray-500">{configHints[config.key]}</p>
                      </div>
                    )}
                  </div>
                ))
              }
            </div>
          )}
        </div>
      </div>

      {/* 未保存提示 */}
      {hasChanges && (
        <div className="bg-amber-50 border border-amber-200 rounded-xl p-4">
          <div className="flex items-center gap-2 text-amber-800">
            <AlertCircle className="w-4 h-4 flex-shrink-0" />
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
