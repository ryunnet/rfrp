import { useEffect, useState } from 'react';
import { clientService, nodeService } from '../lib/services';
import type { Client, LogEntry, Node } from '../lib/types';
import { formatBytes, formatDate, copyToClipboard } from '../lib/utils';
import { useToast } from '../contexts/ToastContext';
import ConfirmDialog from '../components/ConfirmDialog';
import { TableSkeleton } from '../components/Skeleton';

export default function Clients() {
  const { showToast } = useToast();
  const [clients, setClients] = useState<Client[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newClientName, setNewClientName] = useState('');
  const [newClientNodeId, setNewClientNodeId] = useState<number | undefined>(undefined);
  const [nodes, setNodes] = useState<Node[]>([]);
  const [confirmDialog, setConfirmDialog] = useState<{ open: boolean; title: string; message: string; onConfirm: () => void }>({ open: false, title: '', message: '', onConfirm: () => {} });

  // 日志相关状态
  const [showLogsModal, setShowLogsModal] = useState(false);
  const [selectedClient, setSelectedClient] = useState<Client | null>(null);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [logsLoading, setLogsLoading] = useState(false);

  // 流量限制编辑相关状态
  const [showTrafficModal, setShowTrafficModal] = useState(false);
  const [trafficFormData, setTrafficFormData] = useState({
    uploadLimitGb: '' as string,
    downloadLimitGb: '' as string,
    trafficResetCycle: 'none',
  });
  const [trafficSaving, setTrafficSaving] = useState(false);

  useEffect(() => {
    loadClients();
    loadNodes();
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

  const loadNodes = async () => {
    try {
      const response = await nodeService.getNodes();
      if (response.success && response.data) {
        setNodes(response.data);
      }
    } catch (error) {
      // 非管理员可能无权限，静默处理
    }
  };

  const handleCreateClient = async () => {
    if (!newClientName.trim()) {
      showToast('请输入客户端名称', 'error');
      return;
    }

    try {
      const response = await clientService.createClient({ name: newClientName, node_id: newClientNodeId });
      if (response.success) {
        showToast('客户端创建成功', 'success');
        setNewClientName('');
        setNewClientNodeId(undefined);
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

  const handleDeleteClient = (id: number) => {
    setConfirmDialog({
      open: true,
      title: '删除客户端',
      message: '确定要删除这个客户端吗？',
      onConfirm: async () => {
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
      },
    });
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

  const handleEditTraffic = (client: Client) => {
    setSelectedClient(client);
    setTrafficFormData({
      uploadLimitGb: client.uploadLimitGb !== null ? String(client.uploadLimitGb) : '',
      downloadLimitGb: client.downloadLimitGb !== null ? String(client.downloadLimitGb) : '',
      trafficResetCycle: client.trafficResetCycle || 'none',
    });
    setShowTrafficModal(true);
  };

  const handleSaveTraffic = async () => {
    if (!selectedClient) return;

    setTrafficSaving(true);
    try {
      const response = await clientService.updateClient(selectedClient.id, {
        upload_limit_gb: trafficFormData.uploadLimitGb ? parseFloat(trafficFormData.uploadLimitGb) : null,
        download_limit_gb: trafficFormData.downloadLimitGb ? parseFloat(trafficFormData.downloadLimitGb) : null,
        traffic_reset_cycle: trafficFormData.trafficResetCycle,
      });

      if (response.success) {
        showToast('流量限制设置成功', 'success');
        setShowTrafficModal(false);
        setSelectedClient(null);
        loadClients();
      } else {
        showToast(response.message || '设置失败', 'error');
      }
    } catch (error) {
      console.error('设置流量限制失败:', error);
      showToast('设置失败', 'error');
    } finally {
      setTrafficSaving(false);
    }
  };

  const handleResetTrafficExceeded = (client: Client) => {
    setConfirmDialog({
      open: true,
      title: '重置流量超限',
      message: '确定要重置该客户端的流量超限状态吗？',
      onConfirm: async () => {
        try {
          const response = await clientService.updateClient(client.id, {
            is_traffic_exceeded: false,
          });

          if (response.success) {
            showToast('已重置流量超限状态', 'success');
            loadClients();
          } else {
            showToast(response.message || '重置失败', 'error');
          }
        } catch (error) {
          console.error('重置流量超限状态失败:', error);
          showToast('重置失败', 'error');
        }
      },
    });
  };

  const getLevelColor = (level: string) => {
    switch (level.toUpperCase()) {
      case 'ERROR':
        return 'bg-red-100 text-red-700';
      case 'WARN':
        return 'bg-amber-100 text-amber-700';
      case 'INFO':
        return 'bg-blue-100 text-blue-700';
      case 'DEBUG':
        return 'bg-gray-100 text-gray-700';
      default:
        return 'bg-gray-100 text-gray-700';
    }
  };

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">客户端管理</h2>
          <p className="mt-1 text-sm text-gray-500">管理所有客户端连接</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="inline-flex items-center gap-2 px-5 py-2.5 bg-gradient-to-r from-blue-600 to-indigo-600 text-white text-sm font-medium rounded-xl hover:from-blue-700 hover:to-indigo-700 focus:outline-none focus:ring-2 focus:ring-blue-500/40 shadow-lg shadow-blue-500/25 transition-all duration-200"
        >
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-4 h-4">
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
          </svg>
          新建客户端
        </button>
      </div>

      {loading ? (
        <TableSkeleton rows={5} cols={7} />
      ) : (
        <div className="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden">
          <table className="min-w-full">
            <thead>
              <tr className="bg-gradient-to-r from-gray-50 to-gray-100/50">
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  状态
                </th>
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  名称
                </th>
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  所属节点
                </th>
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  Token
                </th>
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  流量统计
                </th>
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  流量限制
                </th>
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  创建时间
                </th>
                <th className="px-6 py-4 text-right text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  操作
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              {clients.length === 0 ? (
                <tr>
                  <td colSpan={8} className="px-6 py-16 text-center">
                    <div className="flex flex-col items-center gap-3">
                      <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-8 h-8 text-gray-400">
                          <path strokeLinecap="round" strokeLinejoin="round" d="M21 7.5l-9-5.25L3 7.5m18 0l-9 5.25m9-5.25v9l-9 5.25M3 7.5l9 5.25M3 7.5v9l9 5.25m0-9v9" />
                        </svg>
                      </div>
                      <p className="text-gray-500">暂无客户端数据</p>
                      <button
                        onClick={() => setShowCreateModal(true)}
                        className="text-sm text-blue-600 hover:text-blue-700 font-medium"
                      >
                        创建第一个客户端
                      </button>
                    </div>
                  </td>
                </tr>
              ) : (
                clients.map((client) => (
                  <tr key={client.id} className="hover:bg-gray-50/50 transition-colors">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center gap-2">
                        <span className={`relative flex h-3 w-3`}>
                          {client.is_online && (
                            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
                          )}
                          <span className={`relative inline-flex rounded-full h-3 w-3 ${client.is_online ? 'bg-green-500' : 'bg-gray-300'}`}></span>
                        </span>
                        <span className={`text-sm font-medium ${client.is_online ? 'text-green-600' : 'text-gray-500'}`}>
                          {client.is_online ? '在线' : '离线'}
                        </span>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center gap-3">
                        <div className="w-9 h-9 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-lg flex items-center justify-center text-white text-sm font-semibold shadow-sm">
                          {client.name.charAt(0).toUpperCase()}
                        </div>
                        <span className="text-sm font-semibold text-gray-900">{client.name}</span>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      {client.nodeId ? (
                        <span className="inline-flex items-center px-2.5 py-1 rounded-lg text-xs font-medium bg-teal-50 text-teal-700">
                          {nodes.find(n => n.id === client.nodeId)?.name || `节点 #${client.nodeId}`}
                        </span>
                      ) : (
                        <span className="text-xs text-gray-400">未分配</span>
                      )}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center gap-2">
                        <code className="text-xs bg-gray-100 text-gray-600 px-2.5 py-1.5 rounded-lg font-mono">
                          {client.token.slice(0, 16)}...
                        </code>
                        <button
                          onClick={() => handleCopyToken(client.token)}
                          className="p-1.5 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                          title="复制 Token"
                        >
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-4 h-4">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M15.666 3.888A2.25 2.25 0 0013.5 2.25h-3c-1.03 0-1.9.693-2.166 1.638m7.332 0c.055.194.084.4.084.612v0a.75.75 0 01-.75.75H9a.75.75 0 01-.75-.75v0c0-.212.03-.418.084-.612m7.332 0c.646.049 1.288.11 1.927.184 1.1.128 1.907 1.077 1.907 2.185V19.5a2.25 2.25 0 01-2.25 2.25H6.75A2.25 2.25 0 014.5 19.5V6.257c0-1.108.806-2.057 1.907-2.185a48.208 48.208 0 011.927-.184" />
                          </svg>
                        </button>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex flex-col gap-1">
                        <div className="flex items-center gap-1.5 text-xs">
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-3.5 h-3.5 text-blue-500">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M4.5 10.5L12 3m0 0l7.5 7.5M12 3v18" />
                          </svg>
                          <span className="text-gray-600">{formatBytes(client.total_bytes_sent)}</span>
                        </div>
                        <div className="flex items-center gap-1.5 text-xs">
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-3.5 h-3.5 text-green-500">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 13.5L12 21m0 0l-7.5-7.5M12 21V3" />
                          </svg>
                          <span className="text-gray-600">{formatBytes(client.total_bytes_received)}</span>
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex flex-col gap-1">
                        {client.isTrafficExceeded && (
                          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-700 mb-1">
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-3 h-3">
                              <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
                            </svg>
                            已超限
                          </span>
                        )}
                        <div className="flex items-center gap-1.5 text-xs">
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-3.5 h-3.5 text-blue-500">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M4.5 10.5L12 3m0 0l7.5 7.5M12 3v18" />
                          </svg>
                          <span className="text-gray-600">
                            {client.uploadLimitGb !== null ? `${client.uploadLimitGb} GB` : '无限制'}
                          </span>
                        </div>
                        <div className="flex items-center gap-1.5 text-xs">
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-3.5 h-3.5 text-green-500">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 13.5L12 21m0 0l-7.5-7.5M12 21V3" />
                          </svg>
                          <span className="text-gray-600">
                            {client.downloadLimitGb !== null ? `${client.downloadLimitGb} GB` : '无限制'}
                          </span>
                        </div>
                        <div className="text-xs text-gray-400">
                          {client.trafficResetCycle === 'none' ? '不重置' :
                           client.trafficResetCycle === 'daily' ? '每日重置' :
                           client.trafficResetCycle === 'weekly' ? '每周重置' :
                           client.trafficResetCycle === 'monthly' ? '每月重置' : client.trafficResetCycle}
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {formatDate(client.created_at)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-right">
                      <div className="flex items-center justify-end gap-1">
                        <button
                          onClick={() => handleEditTraffic(client)}
                          className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-indigo-600 hover:bg-indigo-50 rounded-lg transition-colors"
                          title="设置流量限制"
                        >
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-4 h-4">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M10.5 6h9.75M10.5 6a1.5 1.5 0 11-3 0m3 0a1.5 1.5 0 10-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-9.75 0h9.75" />
                          </svg>
                          流量
                        </button>
                        {client.isTrafficExceeded && (
                          <button
                            onClick={() => handleResetTrafficExceeded(client)}
                            className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-amber-600 hover:bg-amber-50 rounded-lg transition-colors"
                            title="重置流量超限状态"
                          >
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-4 h-4">
                              <path strokeLinecap="round" strokeLinejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99" />
                            </svg>
                            重置
                          </button>
                        )}
                        <button
                          onClick={() => handleViewLogs(client)}
                          disabled={!client.is_online}
                          className={`inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg transition-colors ${
                            client.is_online
                              ? 'text-blue-600 hover:bg-blue-50'
                              : 'text-gray-400 cursor-not-allowed'
                          }`}
                          title={client.is_online ? '查看日志' : '客户端离线'}
                        >
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-4 h-4">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
                          </svg>
                          日志
                        </button>
                        <button
                          onClick={() => handleDeleteClient(client.id)}
                          className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                        >
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-4 h-4">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0" />
                          </svg>
                          删除
                        </button>
                      </div>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      )}

      {/* 创建节点模态框 */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm overflow-y-auto h-full w-full flex items-center justify-center z-50">
          <div className="relative bg-white rounded-2xl shadow-2xl w-full max-w-md mx-4 transform transition-all">
            <div className="p-6">
              <div className="flex items-center gap-3 mb-6">
                <div className="w-10 h-10 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-xl flex items-center justify-center">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-5 h-5 text-white">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
                  </svg>
                </div>
                <div>
                  <h3 className="text-lg font-bold text-gray-900">创建新客户端</h3>
                  <p className="text-sm text-gray-500">添加一个新的客户端</p>
                </div>
              </div>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">客户端名称</label>
                  <input
                    type="text"
                    value={newClientName}
                    onChange={(e) => setNewClientName(e.target.value)}
                    placeholder="请输入客户端名称"
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white"
                    autoFocus
                    onKeyDown={(e) => e.key === 'Enter' && handleCreateClient()}
                  />
                </div>
                {nodes.length > 0 && (
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1.5">所属节点</label>
                    <select
                      value={newClientNodeId ?? ''}
                      onChange={(e) => setNewClientNodeId(e.target.value ? Number(e.target.value) : undefined)}
                      className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white"
                    >
                      <option value="">不指定节点</option>
                      {nodes.map((node) => (
                        <option key={node.id} value={node.id}>
                          {node.name}{node.region ? ` (${node.region})` : ''}{node.isOnline ? '' : ' [离线]'}
                        </option>
                      ))}
                    </select>
                  </div>
                )}
              </div>
              <div className="mt-6 flex gap-3">
                <button
                  onClick={() => {
                    setShowCreateModal(false);
                    setNewClientName('');
                    setNewClientNodeId(undefined);
                  }}
                  className="flex-1 px-4 py-2.5 bg-gray-100 text-gray-700 font-medium rounded-xl hover:bg-gray-200 transition-colors"
                >
                  取消
                </button>
                <button
                  onClick={handleCreateClient}
                  className="flex-1 px-4 py-2.5 bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-medium rounded-xl hover:from-blue-700 hover:to-indigo-700 shadow-lg shadow-blue-500/25 transition-all"
                >
                  创建
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 日志查看模态框 */}
      {showLogsModal && selectedClient && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm overflow-y-auto h-full w-full flex items-center justify-center z-50">
          <div className="relative bg-white rounded-2xl shadow-2xl w-full max-w-4xl mx-4 max-h-[85vh] flex flex-col transform transition-all">
            <div className="flex items-center justify-between p-6 border-b border-gray-100">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-xl flex items-center justify-center">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-5 h-5 text-white">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
                  </svg>
                </div>
                <div>
                  <h3 className="text-lg font-bold text-gray-900">客户端日志</h3>
                  <p className="text-sm text-gray-500">{selectedClient.name}</p>
                </div>
              </div>
              <button
                onClick={() => {
                  setShowLogsModal(false);
                  setSelectedClient(null);
                  setLogs([]);
                }}
                className="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
              >
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-5 h-5">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <div className="flex-1 overflow-y-auto p-6">
              {logsLoading ? (
                <div className="flex items-center justify-center h-64">
                  <div className="flex flex-col items-center gap-3">
                    <div className="w-10 h-10 border-4 border-blue-200 border-t-blue-600 rounded-full animate-spin"></div>
                    <span className="text-sm text-gray-500">加载日志中...</span>
                  </div>
                </div>
              ) : logs.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-64 text-gray-500">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-12 h-12 text-gray-300 mb-3">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
                  </svg>
                  暂无日志数据
                </div>
              ) : (
                <div className="space-y-2 font-mono text-sm">
                  {logs.map((log, index) => (
                    <div
                      key={index}
                      className={`p-3 rounded-lg border-l-4 ${
                        log.level === 'ERROR' ? 'bg-red-50 border-red-500' :
                        log.level === 'WARN' ? 'bg-amber-50 border-amber-500' :
                        log.level === 'INFO' ? 'bg-blue-50 border-blue-500' : 'bg-gray-50 border-gray-400'
                      }`}
                    >
                      <div className="flex items-start gap-3">
                        <span className="text-gray-400 text-xs whitespace-nowrap">
                          {new Date(log.timestamp).toLocaleString('zh-CN')}
                        </span>
                        <span className={`font-semibold text-xs whitespace-nowrap px-1.5 py-0.5 rounded ${getLevelColor(log.level)}`}>
                          {log.level}
                        </span>
                        <span className="text-gray-700 flex-1 break-all">
                          {log.message.replace(/^"|"$/g, '')}
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div className="p-4 border-t border-gray-100 flex justify-end">
              <button
                onClick={() => {
                  setShowLogsModal(false);
                  setSelectedClient(null);
                  setLogs([]);
                }}
                className="px-5 py-2.5 bg-gray-100 text-gray-700 font-medium rounded-xl hover:bg-gray-200 transition-colors"
              >
                关闭
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 流量限制设置模态框 */}
      {showTrafficModal && selectedClient && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm overflow-y-auto h-full w-full flex items-center justify-center z-50">
          <div className="relative bg-white rounded-2xl shadow-2xl w-full max-w-md mx-4 transform transition-all">
            <div className="p-6">
              <div className="flex items-center gap-3 mb-6">
                <div className="w-10 h-10 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-xl flex items-center justify-center">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-5 h-5 text-white">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M10.5 6h9.75M10.5 6a1.5 1.5 0 11-3 0m3 0a1.5 1.5 0 10-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-9.75 0h9.75" />
                  </svg>
                </div>
                <div>
                  <h3 className="text-lg font-bold text-gray-900">流量限制设置</h3>
                  <p className="text-sm text-gray-500">{selectedClient.name}</p>
                </div>
              </div>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">上传限制 (GB)</label>
                  <input
                    type="number"
                    step="0.1"
                    min="0"
                    value={trafficFormData.uploadLimitGb}
                    onChange={(e) => setTrafficFormData({ ...trafficFormData, uploadLimitGb: e.target.value })}
                    placeholder="留空表示无限制"
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all bg-gray-50/50 hover:bg-white"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">下载限制 (GB)</label>
                  <input
                    type="number"
                    step="0.1"
                    min="0"
                    value={trafficFormData.downloadLimitGb}
                    onChange={(e) => setTrafficFormData({ ...trafficFormData, downloadLimitGb: e.target.value })}
                    placeholder="留空表示无限制"
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all bg-gray-50/50 hover:bg-white"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">流量重置周期</label>
                  <select
                    value={trafficFormData.trafficResetCycle}
                    onChange={(e) => setTrafficFormData({ ...trafficFormData, trafficResetCycle: e.target.value })}
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all bg-gray-50/50 hover:bg-white"
                  >
                    <option value="none">不重置</option>
                    <option value="daily">每日重置</option>
                    <option value="weekly">每周重置</option>
                    <option value="monthly">每月重置</option>
                  </select>
                </div>
              </div>
              <div className="mt-6 flex gap-3">
                <button
                  onClick={() => {
                    setShowTrafficModal(false);
                    setSelectedClient(null);
                  }}
                  className="flex-1 px-4 py-2.5 bg-gray-100 text-gray-700 font-medium rounded-xl hover:bg-gray-200 transition-colors"
                  disabled={trafficSaving}
                >
                  取消
                </button>
                <button
                  onClick={handleSaveTraffic}
                  disabled={trafficSaving}
                  className="flex-1 px-4 py-2.5 bg-gradient-to-r from-indigo-600 to-purple-600 text-white font-medium rounded-xl hover:from-indigo-700 hover:to-purple-700 shadow-lg shadow-indigo-500/25 transition-all disabled:opacity-50"
                >
                  {trafficSaving ? '保存中...' : '保存'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 确认对话框 */}
      <ConfirmDialog
        open={confirmDialog.open}
        title={confirmDialog.title}
        message={confirmDialog.message}
        variant="danger"
        confirmText="确定"
        onConfirm={() => {
          confirmDialog.onConfirm();
          setConfirmDialog(prev => ({ ...prev, open: false }));
        }}
        onCancel={() => setConfirmDialog(prev => ({ ...prev, open: false }))}
      />
    </div>
  );
}
