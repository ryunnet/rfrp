import { useEffect, useState } from 'react';
import { proxyService, clientService, nodeService } from '../lib/services';
import type { Proxy, Client, Node } from '../lib/types';
import { formatBytes } from '../lib/utils';
import { useToast } from '../contexts/ToastContext';
import ConfirmDialog from '../components/ConfirmDialog';
import { TableSkeleton } from '../components/Skeleton';

export default function Proxies() {
  const { showToast } = useToast();
  const [proxies, setProxies] = useState<Proxy[]>([]);
  const [clients, setClients] = useState<Client[]>([]);
  const [nodes, setNodes] = useState<Node[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [editingProxy, setEditingProxy] = useState<Proxy | null>(null);
  const [confirmDialog, setConfirmDialog] = useState<{ open: boolean; title: string; message: string; onConfirm: () => void }>({ open: false, title: '', message: '', onConfirm: () => {} });
  const [formData, setFormData] = useState({
    client_id: '',
    node_id: '',
    name: '',
    type: 'tcp',
    localIP: '127.0.0.1',
    localPort: '',
    remotePort: '',
    enabled: true,
  });

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      const [proxiesRes, clientsRes, nodesRes] = await Promise.all([
        proxyService.getProxies(),
        clientService.getClients(),
        nodeService.getNodes(),
      ]);
      if (proxiesRes.success && proxiesRes.data) setProxies(proxiesRes.data);
      if (clientsRes.success && clientsRes.data) setClients(clientsRes.data);
      if (nodesRes.success && nodesRes.data) setNodes(nodesRes.data);
    } catch (error) {
      console.error('加载数据失败:', error);
      showToast('加载失败', 'error');
    } finally {
      setLoading(false);
    }
  };

  const resetForm = () => {
    setFormData({
      client_id: '',
      node_id: '',
      name: '',
      type: 'tcp',
      localIP: '127.0.0.1',
      localPort: '',
      remotePort: '',
      enabled: true,
    });
    setEditingProxy(null);
  };

  const handleCreateProxy = async () => {
    if (!formData.name || !formData.client_id || !formData.node_id || !formData.localPort || !formData.remotePort) {
      showToast('请填写所有必填字段', 'error');
      return;
    }

    try {
      const response = await proxyService.createProxy({
        client_id: formData.client_id,
        name: formData.name,
        type: formData.type,
        localIP: formData.localIP,
        localPort: parseInt(formData.localPort),
        remotePort: parseInt(formData.remotePort),
        nodeId: parseInt(formData.node_id),
      });
      if (response.success) {
        showToast('代理创建成功', 'success');
        resetForm();
        setShowCreateModal(false);
        loadData();
      } else {
        showToast(response.message || '创建失败', 'error');
      }
    } catch (error) {
      console.error('创建代理失败:', error);
      showToast('创建失败', 'error');
    }
  };

  const handleUpdateProxy = async () => {
    if (!editingProxy) return;

    try {
      const response = await proxyService.updateProxy(editingProxy.id, {
        name: formData.name || undefined,
        type: formData.type || undefined,
        localIP: formData.localIP || undefined,
        localPort: formData.localPort ? parseInt(formData.localPort) : undefined,
        remotePort: formData.remotePort ? parseInt(formData.remotePort) : undefined,
        enabled: formData.enabled,
      });
      if (response.success) {
        showToast('代理更新成功', 'success');
        resetForm();
        setEditingProxy(null);
        setShowCreateModal(false);
        loadData();
      } else {
        showToast(response.message || '更新失败', 'error');
      }
    } catch (error) {
      console.error('更新代理失败:', error);
      showToast('更新失败', 'error');
    }
  };

  const handleEdit = (proxy: Proxy) => {
    setEditingProxy(proxy);
    setFormData({
      client_id: proxy.client_id,
      node_id: proxy.nodeId ? proxy.nodeId.toString() : '',
      name: proxy.name,
      type: proxy.type,
      localIP: proxy.localIP,
      localPort: proxy.localPort.toString(),
      remotePort: proxy.remotePort.toString(),
      enabled: proxy.enabled,
    });
    setShowCreateModal(true);
  };

  const handleDelete = (id: number) => {
    setConfirmDialog({
      open: true,
      title: '删除代理',
      message: '确定要删除这个代理吗？',
      onConfirm: async () => {
        try {
          const response = await proxyService.deleteProxy(id);
          if (response.success) {
            showToast('代理删除成功', 'success');
            loadData();
          } else {
            showToast(response.message || '删除失败', 'error');
          }
        } catch (error) {
          console.error('删除代理失败:', error);
          showToast('删除失败', 'error');
        }
      },
    });
  };

  const handleToggleEnabled = async (proxy: Proxy) => {
    try {
      const response = await proxyService.updateProxy(proxy.id, {
        enabled: !proxy.enabled,
      });
      if (response.success) {
        showToast(`代理已${proxy.enabled ? '禁用' : '启用'}`, 'success');
        loadData();
      }
    } catch (error) {
      console.error('切换状态失败:', error);
      showToast('操作失败', 'error');
    }
  };

  const getClientName = (clientId: string) => {
    const client = clients.find((c) => c.id.toString() === clientId);
    return client?.name || clientId;
  };

  const getNodeName = (nodeId: number | null) => {
    if (!nodeId) return '-';
    const node = nodes.find((n) => n.id === nodeId);
    return node?.name || String(nodeId);
  };

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">代理管理</h2>
          <p className="mt-1 text-sm text-gray-500">管理所有代理映射规则</p>
        </div>
        <button
          onClick={() => {
            resetForm();
            setShowCreateModal(true);
          }}
          className="inline-flex items-center gap-2 px-5 py-2.5 bg-gradient-to-r from-blue-600 to-indigo-600 text-white text-sm font-medium rounded-xl hover:from-blue-700 hover:to-indigo-700 focus:outline-none focus:ring-2 focus:ring-blue-500/40 shadow-lg shadow-blue-500/25 transition-all duration-200"
        >
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-4 h-4">
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
          </svg>
          新建代理
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
                  名称
                </th>
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  客户端
                </th>
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  节点
                </th>
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  类型
                </th>
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  端口映射
                </th>
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  状态
                </th>
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  流量
                </th>
                <th className="px-6 py-4 text-right text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  操作
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              {proxies.length === 0 ? (
                <tr>
                  <td colSpan={8} className="px-6 py-16 text-center">
                    <div className="flex flex-col items-center gap-3">
                      <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-8 h-8 text-gray-400">
                          <path strokeLinecap="round" strokeLinejoin="round" d="M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" />
                        </svg>
                      </div>
                      <p className="text-gray-500">暂无代理数据</p>
                      <button
                        onClick={() => {
                          resetForm();
                          setShowCreateModal(true);
                        }}
                        className="text-sm text-blue-600 hover:text-blue-700 font-medium"
                      >
                        创建第一个代理
                      </button>
                    </div>
                  </td>
                </tr>
              ) : (
                proxies.map((proxy) => (
                  <tr key={proxy.id} className="hover:bg-gray-50/50 transition-colors">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center gap-3">
                        <div className="w-9 h-9 bg-gradient-to-br from-purple-500 to-pink-600 rounded-lg flex items-center justify-center text-white text-sm font-semibold shadow-sm">
                          {proxy.name.charAt(0).toUpperCase()}
                        </div>
                        <span className="text-sm font-semibold text-gray-900">{proxy.name}</span>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className="text-sm text-gray-600">{getClientName(proxy.client_id)}</span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className="text-sm text-gray-600">{getNodeName(proxy.nodeId)}</span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className="inline-flex items-center px-2.5 py-1 text-xs font-semibold rounded-lg bg-blue-100 text-blue-700">
                        {(proxy.type || 'tcp').toUpperCase()}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center gap-2 text-sm">
                        <span className="px-2 py-1 bg-indigo-50 text-indigo-700 rounded-lg font-mono text-xs">
                          :{proxy.remotePort}
                        </span>
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-4 h-4 text-gray-400">
                          <path strokeLinecap="round" strokeLinejoin="round" d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3" />
                        </svg>
                        <span className="text-gray-500 font-mono text-xs">
                          {proxy.localIP}:{proxy.localPort}
                        </span>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 text-xs font-semibold rounded-lg ${
                        proxy.enabled
                          ? 'bg-green-100 text-green-700'
                          : 'bg-gray-100 text-gray-600'
                      }`}>
                        <span className={`w-1.5 h-1.5 rounded-full ${proxy.enabled ? 'bg-green-500' : 'bg-gray-400'}`}></span>
                        {proxy.enabled ? '启用' : '禁用'}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex flex-col gap-1">
                        <div className="flex items-center gap-1.5 text-xs">
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-3.5 h-3.5 text-blue-500">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M4.5 10.5L12 3m0 0l7.5 7.5M12 3v18" />
                          </svg>
                          <span className="text-gray-600">{formatBytes(proxy.totalBytesSent)}</span>
                        </div>
                        <div className="flex items-center gap-1.5 text-xs">
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-3.5 h-3.5 text-green-500">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 13.5L12 21m0 0l-7.5-7.5M12 21V3" />
                          </svg>
                          <span className="text-gray-600">{formatBytes(proxy.totalBytesReceived)}</span>
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-right">
                      <div className="flex items-center justify-end gap-1">
                        <button
                          onClick={() => handleToggleEnabled(proxy)}
                          className={`inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg transition-colors ${
                            proxy.enabled
                              ? 'text-amber-600 hover:bg-amber-50'
                              : 'text-green-600 hover:bg-green-50'
                          }`}
                        >
                          {proxy.enabled ? '禁用' : '启用'}
                        </button>
                        <button
                          onClick={() => handleEdit(proxy)}
                          className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                        >
                          编辑
                        </button>
                        <button
                          onClick={() => handleDelete(proxy.id)}
                          className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                        >
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

      {/* 创建/编辑代理模态框 */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm overflow-y-auto h-full w-full flex items-center justify-center z-50">
          <div className="relative bg-white rounded-2xl shadow-2xl w-full max-w-lg mx-4 transform transition-all">
            <div className="p-6">
              <div className="flex items-center gap-3 mb-6">
                <div className="w-10 h-10 bg-gradient-to-br from-purple-500 to-pink-600 rounded-xl flex items-center justify-center">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-5 h-5 text-white">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" />
                  </svg>
                </div>
                <div>
                  <h3 className="text-lg font-bold text-gray-900">
                    {editingProxy ? '编辑代理' : '创建新代理'}
                  </h3>
                  <p className="text-sm text-gray-500">
                    {editingProxy ? '修改代理配置信息' : '添加一个新的端口映射规则'}
                  </p>
                </div>
              </div>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">客户端 *</label>
                  <select
                    value={formData.client_id}
                    onChange={(e) => setFormData({ ...formData, client_id: e.target.value })}
                    disabled={!!editingProxy}
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white disabled:bg-gray-100 disabled:cursor-not-allowed"
                  >
                    <option value="">选择客户端</option>
                    {clients.map((client) => (
                      <option key={client.id} value={client.id.toString()}>
                        {client.name}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">节点 *</label>
                  <select
                    value={formData.node_id}
                    onChange={(e) => setFormData({ ...formData, node_id: e.target.value })}
                    disabled={!!editingProxy}
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white disabled:bg-gray-100 disabled:cursor-not-allowed"
                  >
                    <option value="">选择节点</option>
                    {nodes.map((node) => (
                      <option key={node.id} value={node.id.toString()}>
                        {node.name}{node.region ? ` (${node.region})` : ''}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">代理名称 *</label>
                  <input
                    type="text"
                    value={formData.name}
                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                    placeholder="请输入代理名称"
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white"
                  />
                </div>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1.5">代理类型 *</label>
                    <select
                      value={formData.type}
                      onChange={(e) => setFormData({ ...formData, type: e.target.value })}
                      className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white"
                    >
                      <option value="tcp">TCP</option>
                      <option value="udp">UDP</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1.5">客户端本地 IP *</label>
                    <input
                      type="text"
                      value={formData.localIP}
                      onChange={(e) => setFormData({ ...formData, localIP: e.target.value })}
                      className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white"
                    />
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1.5">客户端本地端口 *</label>
                    <input
                      type="number"
                      value={formData.localPort}
                      onChange={(e) => setFormData({ ...formData, localPort: e.target.value })}
                      placeholder="如: 80"
                      className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1.5">节点端口 *</label>
                    <input
                      type="number"
                      value={formData.remotePort}
                      onChange={(e) => setFormData({ ...formData, remotePort: e.target.value })}
                      placeholder="如: 8080"
                      className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white"
                    />
                  </div>
                </div>
                {editingProxy && (
                  <div className="flex items-center gap-3 p-3 bg-gray-50 rounded-xl">
                    <input
                      type="checkbox"
                      id="enabled"
                      checked={formData.enabled}
                      onChange={(e) => setFormData({ ...formData, enabled: e.target.checked })}
                      className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                    />
                    <label htmlFor="enabled" className="text-sm text-gray-700 font-medium">
                      启用代理
                    </label>
                  </div>
                )}
              </div>
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
                  onClick={editingProxy ? handleUpdateProxy : handleCreateProxy}
                  className="flex-1 px-4 py-2.5 bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-medium rounded-xl hover:from-blue-700 hover:to-indigo-700 shadow-lg shadow-blue-500/25 transition-all"
                >
                  {editingProxy ? '更新' : '创建'}
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
        variant="danger"
        confirmText="删除"
        onConfirm={() => {
          confirmDialog.onConfirm();
          setConfirmDialog(prev => ({ ...prev, open: false }));
        }}
        onCancel={() => setConfirmDialog(prev => ({ ...prev, open: false }))}
      />
    </div>
  );
}
