import { useEffect, useState } from 'react';
import { clientService } from '../lib/services';
import type { Client, LogEntry } from '../lib/types';
import { formatBytes, formatDate, copyToClipboard, getOnlineStatusColor } from '../lib/utils';

export default function Clients() {
  const [clients, setClients] = useState<Client[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newClientName, setNewClientName] = useState('');
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' } | null>(null);

  // 日志相关状态
  const [showLogsModal, setShowLogsModal] = useState(false);
  const [selectedClient, setSelectedClient] = useState<Client | null>(null);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [logsLoading, setLogsLoading] = useState(false);

  useEffect(() => {
    loadClients();
  }, []);

  const loadClients = async () => {
    try {
      setLoading(true);
      const response = await clientService.getClients();
      if (response.success && response.data) {
        setClients(response.data);
      }
    } catch (error) {
      console.error('加载客户端失败:', error);
      showToast('加载失败', 'error');
    } finally {
      setLoading(false);
    }
  };

  const handleCreateClient = async () => {
    if (!newClientName.trim()) {
      showToast('请输入客户端名称', 'error');
      return;
    }

    try {
      const response = await clientService.createClient({ name: newClientName });
      if (response.success) {
        showToast('客户端创建成功', 'success');
        setNewClientName('');
        setShowCreateModal(false);
        loadClients();
      } else {
        showToast(response.message || '创建失败', 'error');
      }
    } catch (error) {
      console.error('创建客户端失败:', error);
      showToast('创建失败', 'error');
    }
  };

  const handleDeleteClient = async (id: number) => {
    if (!confirm('确定要删除这个客户端吗？')) return;

    try {
      const response = await clientService.deleteClient(id);
      if (response.success) {
        showToast('客户端删除成功', 'success');
        loadClients();
      } else {
        showToast(response.message || '删除失败', 'error');
      }
    } catch (error) {
      console.error('删除客户端失败:', error);
      showToast('删除失败', 'error');
    }
  };

  const handleViewLogs = async (client: Client) => {
    if (!client.is_online) {
      showToast('客户端离线，无法获取日志', 'error');
      return;
    }

    setSelectedClient(client);
    setShowLogsModal(true);
    setLogsLoading(true);

    try {
      const response = await clientService.getClientLogs(client.id);
      if (response.success && response.data) {
        setLogs(response.data);
      } else {
        showToast(response.message || '获取日志失败', 'error');
        setLogs([]);
      }
    } catch (error) {
      console.error('获取日志失败:', error);
      showToast('获取日志失败', 'error');
      setLogs([]);
    } finally {
      setLogsLoading(false);
    }
  };

  const handleCopyToken = async (token: string) => {
    const success = await copyToClipboard(token);
    showToast(success ? 'Token 已复制' : '复制失败', success ? 'success' : 'error');
  };

  const showToast = (message: string, type: 'success' | 'error') => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  };

  const getLevelColor = (level: string) => {
    switch (level.toUpperCase()) {
      case 'ERROR':
        return 'text-red-600';
      case 'WARN':
        return 'text-yellow-600';
      case 'INFO':
        return 'text-blue-600';
      case 'DEBUG':
        return 'text-gray-600';
      default:
        return 'text-gray-800';
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">客户端管理</h2>
          <p className="mt-1 text-sm text-gray-600">管理所有客户端连接</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          新建客户端
        </button>
      </div>

      {loading ? (
        <div className="flex items-center justify-center h-64">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
        </div>
      ) : (
        <div className="bg-white shadow rounded-lg overflow-hidden">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  状态
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  名称
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Token
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  上传 / 下载
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
              {clients.length === 0 ? (
                <tr>
                  <td colSpan={6} className="px-6 py-12 text-center text-gray-500">
                    暂无客户端数据
                  </td>
                </tr>
              ) : (
                clients.map((client) => (
                  <tr key={client.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center">
                        <span className={`h-3 w-3 rounded-full ${getOnlineStatusColor(client.is_online)}`}></span>
                        <span className="ml-2 text-sm text-gray-900">
                          {client.is_online ? '在线' : '离线'}
                        </span>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                      {client.name}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center space-x-2">
                        <code className="text-xs bg-gray-100 px-2 py-1 rounded">
                          {client.token.slice(0, 16)}...
                        </code>
                        <button
                          onClick={() => handleCopyToken(client.token)}
                          className="text-blue-600 hover:text-blue-800 text-sm"
                        >
                          复制
                        </button>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      <div>
                        <div>⬆️ {formatBytes(client.total_bytes_sent)}</div>
                        <div>⬇️ {formatBytes(client.total_bytes_received)}</div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {formatDate(client.created_at)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-3">
                      <button
                        onClick={() => handleViewLogs(client)}
                        disabled={!client.is_online}
                        className={`${
                          client.is_online
                            ? 'text-blue-600 hover:text-blue-900'
                            : 'text-gray-400 cursor-not-allowed'
                        }`}
                        title={client.is_online ? '查看日志' : '客户端离线'}
                      >
                        📋 日志
                      </button>
                      <button
                        onClick={() => handleDeleteClient(client.id)}
                        className="text-red-600 hover:text-red-900"
                      >
                        删除
                      </button>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      )}

      {/* 创建客户端模态框 */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full flex items-center justify-center z-50">
          <div className="relative p-5 border w-96 shadow-lg rounded-md bg-white">
            <h3 className="text-lg font-bold text-gray-900 mb-4">创建新客户端</h3>
            <input
              type="text"
              value={newClientName}
              onChange={(e) => setNewClientName(e.target.value)}
              placeholder="客户端名称"
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              autoFocus
              onKeyDown={(e) => e.key === 'Enter' && handleCreateClient()}
            />
            <div className="mt-4 flex justify-end space-x-2">
              <button
                onClick={() => {
                  setShowCreateModal(false);
                  setNewClientName('');
                }}
                className="px-4 py-2 bg-gray-200 text-gray-800 rounded-md hover:bg-gray-300"
              >
                取消
              </button>
              <button
                onClick={handleCreateClient}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
              >
                创建
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 日志查看模态框 */}
      {showLogsModal && selectedClient && (
        <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full flex items-center justify-center z-50">
          <div className="relative p-5 border w-4/5 max-w-4xl shadow-lg rounded-md bg-white max-h-[80vh] flex flex-col">
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-lg font-bold text-gray-900">
                客户端日志 - {selectedClient.name}
              </h3>
              <button
                onClick={() => {
                  setShowLogsModal(false);
                  setSelectedClient(null);
                  setLogs([]);
                }}
                className="text-gray-500 hover:text-gray-700 text-2xl"
              >
                ×
              </button>
            </div>

            <div className="flex-1 overflow-y-auto">
              {logsLoading ? (
                <div className="flex items-center justify-center h-64">
                  <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                </div>
              ) : logs.length === 0 ? (
                <div className="text-center py-12 text-gray-500">
                  暂无日志数据
                </div>
              ) : (
                <div className="space-y-1 font-mono text-sm">
                  {logs.map((log, index) => (
                    <div
                      key={index}
                      className="p-2 hover:bg-gray-50 rounded border-l-4"
                      style={{
                        borderLeftColor:
                          log.level === 'ERROR' ? '#dc2626' :
                          log.level === 'WARN' ? '#f59e0b' :
                          log.level === 'INFO' ? '#3b82f6' : '#6b7280'
                      }}
                    >
                      <div className="flex items-start space-x-3">
                        <span className="text-gray-500 text-xs whitespace-nowrap">
                          {new Date(log.timestamp).toLocaleString('zh-CN')}
                        </span>
                        <span className={`font-semibold text-xs whitespace-nowrap ${getLevelColor(log.level)}`}>
                          [{log.level}]
                        </span>
                        <span className="text-gray-800 flex-1 break-all">
                          {log.message.replace(/^"|"$/g, '')}
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div className="mt-4 flex justify-end">
              <button
                onClick={() => {
                  setShowLogsModal(false);
                  setSelectedClient(null);
                  setLogs([]);
                }}
                className="px-4 py-2 bg-gray-200 text-gray-800 rounded-md hover:bg-gray-300"
              >
                关闭
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 提示消息 */}
      {toast && (
        <div
          className={`fixed bottom-4 right-4 px-4 py-2 rounded-md text-white ${
            toast.type === 'success' ? 'bg-green-600' : 'bg-red-600'
          }`}
        >
          {toast.message}
        </div>
      )}
    </div>
  );
}
