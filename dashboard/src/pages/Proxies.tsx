import { useEffect, useState } from 'react';
import { proxyService, clientService } from '../lib/services';
import type { Proxy, Client } from '../lib/types';
import { formatBytes, getStatusBgColor } from '../lib/utils';

export default function Proxies() {
  const [proxies, setProxies] = useState<Proxy[]>([]);
  const [clients, setClients] = useState<Client[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [editingProxy, setEditingProxy] = useState<Proxy | null>(null);
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' } | null>(null);
  const [formData, setFormData] = useState({
    client_id: '',
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
      const [proxiesRes, clientsRes] = await Promise.all([
        proxyService.getProxies(),
        clientService.getClients(),
      ]);
      if (proxiesRes.success && proxiesRes.data) setProxies(proxiesRes.data);
      if (clientsRes.success && clientsRes.data) setClients(clientsRes.data);
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
    if (!formData.name || !formData.client_id || !formData.localPort || !formData.remotePort) {
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
      name: proxy.name,
      type: proxy.type,
      localIP: proxy.localIP,
      localPort: proxy.localPort.toString(),
      remotePort: proxy.remotePort.toString(),
      enabled: proxy.enabled,
    });
    setShowCreateModal(true);
  };

  const handleDelete = async (id: number) => {
    if (!confirm('确定要删除这个代理吗？')) return;

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

  const showToast = (message: string, type: 'success' | 'error') => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  };

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">代理管理</h2>
          <p className="mt-1 text-sm text-gray-600">管理所有代理规则</p>
        </div>
        <button
          onClick={() => {
            resetForm();
            setShowCreateModal(true);
          }}
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          新建代理
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
                  名称
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  客户端
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  类型
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  端口映射
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  状态
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  流量
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  操作
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {proxies.length === 0 ? (
                <tr>
                  <td colSpan={7} className="px-6 py-12 text-center text-gray-500">
                    暂无代理数据
                  </td>
                </tr>
              ) : (
                proxies.map((proxy) => (
                  <tr key={proxy.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                      {proxy.name}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {getClientName(proxy.client_id)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className="px-2 py-1 text-xs font-medium rounded bg-blue-100 text-blue-800">
                        {(proxy.type || 'tcp').toUpperCase()}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      <div className="text-xs">
                        <div>远程: {proxy.remotePort}</div>
                        <div className="text-gray-400">→ {proxy.localIP}:{proxy.localPort}</div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className={`px-2 py-1 text-xs font-medium rounded ${getStatusBgColor(proxy.enabled)}`}>
                        {proxy.enabled ? '启用' : '禁用'}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      <div className="text-xs">
                        <div>↑ {formatBytes(proxy.totalBytesSent)}</div>
                        <div>↓ {formatBytes(proxy.totalBytesReceived)}</div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                      <button
                        onClick={() => handleToggleEnabled(proxy)}
                        className="text-blue-600 hover:text-blue-900 mr-3"
                      >
                        {proxy.enabled ? '禁用' : '启用'}
                      </button>
                      <button
                        onClick={() => handleEdit(proxy)}
                        className="text-blue-600 hover:text-blue-900 mr-3"
                      >
                        编辑
                      </button>
                      <button
                        onClick={() => handleDelete(proxy.id)}
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

      {/* 创建/编辑代理模态框 */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full flex items-center justify-center">
          <div className="relative p-5 border w-[500px] shadow-lg rounded-md bg-white">
            <h3 className="text-lg font-bold text-gray-900 mb-4">
              {editingProxy ? '编辑代理' : '创建新代理'}
            </h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700">客户端 *</label>
                <select
                  value={formData.client_id}
                  onChange={(e) => setFormData({ ...formData, client_id: e.target.value })}
                  disabled={!!editingProxy}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
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
                <label className="block text-sm font-medium text-gray-700">代理名称 *</label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">代理类型 *</label>
                <select
                  value={formData.type}
                  onChange={(e) => setFormData({ ...formData, type: e.target.value })}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="tcp">TCP</option>
                  <option value="udp">UDP</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">本地IP *</label>
                <input
                  type="text"
                  value={formData.localIP}
                  onChange={(e) => setFormData({ ...formData, localIP: e.target.value })}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700">本地端口 *</label>
                  <input
                    type="number"
                    value={formData.localPort}
                    onChange={(e) => setFormData({ ...formData, localPort: e.target.value })}
                    className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700">远程端口 *</label>
                  <input
                    type="number"
                    value={formData.remotePort}
                    onChange={(e) => setFormData({ ...formData, remotePort: e.target.value })}
                    className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
              </div>
              {editingProxy && (
                <div className="flex items-center">
                  <input
                    type="checkbox"
                    id="enabled"
                    checked={formData.enabled}
                    onChange={(e) => setFormData({ ...formData, enabled: e.target.checked })}
                    className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                  />
                  <label htmlFor="enabled" className="ml-2 block text-sm text-gray-900">
                    启用代理
                  </label>
                </div>
              )}
            </div>
            <div className="mt-6 flex justify-end space-x-2">
              <button
                onClick={() => {
                  setShowCreateModal(false);
                  resetForm();
                }}
                className="px-4 py-2 bg-gray-200 text-gray-800 rounded-md hover:bg-gray-300"
              >
                取消
              </button>
              <button
                onClick={editingProxy ? handleUpdateProxy : handleCreateProxy}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
              >
                {editingProxy ? '更新' : '创建'}
              </button>
            </div>
          </div>
        </div>
      )}

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
