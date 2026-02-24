import { useEffect, useState } from 'react';
import { nodeService } from '../lib/services';
import type { Node } from '../lib/types';
import { formatDate } from '../lib/utils';
import { useToast } from '../contexts/ToastContext';
import ConfirmDialog from '../components/ConfirmDialog';
import { TableSkeleton } from '../components/Skeleton';

export default function Nodes() {
  const { showToast } = useToast();
  const [nodes, setNodes] = useState<Node[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [showCommandModal, setShowCommandModal] = useState(false);
  const [createdNodeInfo, setCreatedNodeInfo] = useState<{ name: string; secret: string } | null>(null);
  const [editingNode, setEditingNode] = useState<Node | null>(null);
  const [commandNode, setCommandNode] = useState<Node | null>(null);
  const [controllerUrl, setControllerUrl] = useState('');
  const [formData, setFormData] = useState({
    name: '',
    url: '',
    secret: '',
    region: '',
    description: '',
    tunnelAddr: '',
    tunnelPort: '7000',
    tunnelProtocol: 'quic',
  });
  const [confirmDialog, setConfirmDialog] = useState<{ open: boolean; title: string; message: string; onConfirm: () => void }>({ open: false, title: '', message: '', onConfirm: () => {} });
  const [testingId, setTestingId] = useState<number | null>(null);

  useEffect(() => {
    loadNodes();
  }, []);

  const loadNodes = async () => {
    try {
      setLoading(true);
      const response = await nodeService.getNodes();
      if (response.success && response.data) {
        setNodes(response.data);
      }
    } catch (error) {
      console.error('加载节点失败:', error);
      showToast('加载失败', 'error');
    } finally {
      setLoading(false);
    }
  };

  const handleCreateNode = async () => {
    if (!formData.name.trim()) {
      showToast('请填写名称', 'error');
      return;
    }

    try {
      const response = await nodeService.createNode({
        name: formData.name,
        url: formData.url || '',
        secret: formData.secret || undefined,
        region: formData.region || undefined,
        description: formData.description || undefined,
        tunnelAddr: formData.tunnelAddr || undefined,
        tunnelPort: formData.tunnelPort ? parseInt(formData.tunnelPort) : undefined,
        tunnelProtocol: formData.tunnelProtocol || undefined,
      });
      if (response.success) {
        showToast('节点创建成功', 'success');
        setCreatedNodeInfo({
          name: response.data!.name,
          secret: response.data!.secret,
        });
        setControllerUrl(`${window.location.hostname}:3100`);
        setShowCreateModal(false);
        setShowCommandModal(true);
        resetForm();
        loadNodes();
      } else {
        showToast(response.message || '创建失败', 'error');
      }
    } catch (error) {
      console.error('创建节点失败:', error);
      showToast('创建失败', 'error');
    }
  };

  const handleEditNode = async () => {
    if (!editingNode) return;

    try {
      const response = await nodeService.updateNode(editingNode.id, {
        name: formData.name || undefined,
        url: formData.url || undefined,
        secret: formData.secret || undefined,
        region: formData.region,
        description: formData.description,
        tunnelAddr: formData.tunnelAddr || undefined,
        tunnelPort: formData.tunnelPort ? parseInt(formData.tunnelPort) : undefined,
        tunnelProtocol: formData.tunnelProtocol || undefined,
      });
      if (response.success) {
        showToast('节点更新成功', 'success');
        resetForm();
        setShowEditModal(false);
        setEditingNode(null);
        loadNodes();
      } else {
        showToast(response.message || '更新失败', 'error');
      }
    } catch (error) {
      console.error('更新节点失败:', error);
      showToast('更新失败', 'error');
    }
  };

  const handleDeleteNode = (node: Node) => {
    setConfirmDialog({
      open: true,
      title: '删除节点',
      message: `确定要删除节点 "${node.name}" 吗？`,
      onConfirm: async () => {
        try {
          const response = await nodeService.deleteNode(node.id);
          if (response.success) {
            showToast('节点已删除', 'success');
            loadNodes();
          } else {
            showToast(response.message || '删除失败', 'error');
          }
        } catch (error) {
          console.error('删除节点失败:', error);
          showToast('删除失败', 'error');
        }
        setConfirmDialog(prev => ({ ...prev, open: false }));
      },
    });
  };

  const handleTestConnection = async (node: Node) => {
    setTestingId(node.id);
    try {
      const response = await nodeService.testConnection(node.id);
      if (response.success && response.data) {
        if (response.data.online) {
          showToast(`节点 "${node.name}" 连接正常，已连接 ${response.data.connected_clients} 个客户端`, 'success');
        } else {
          showToast(`节点 "${node.name}" 连接失败: ${response.data.error || '未知错误'}`, 'error');
        }
      }
    } catch (error) {
      console.error('测试连接失败:', error);
      showToast('测试连接失败', 'error');
    } finally {
      setTestingId(null);
    }
  };

  const openEditModal = (node: Node) => {
    setEditingNode(node);
    setFormData({
      name: node.name,
      url: node.url,
      secret: node.secret,
      region: node.region || '',
      description: node.description || '',
      tunnelAddr: node.tunnelAddr || '',
      tunnelPort: String(node.tunnelPort || 7000),
      tunnelProtocol: node.tunnelProtocol || 'quic',
    });
    setShowEditModal(true);
  };

  const resetForm = () => {
    setFormData({ name: '', url: '', secret: '', region: '', description: '', tunnelAddr: '', tunnelPort: '7000', tunnelProtocol: 'quic' });
  };

  const tunnelFields = (
    <>
      <div className="border-t border-gray-100 pt-4 mt-4">
        <h3 className="text-sm font-semibold text-gray-900 mb-3">隧道连接配置</h3>
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1.5">隧道地址 *</label>
        <input
          type="text"
          value={formData.tunnelAddr}
          onChange={(e) => setFormData({ ...formData, tunnelAddr: e.target.value })}
          className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white"
          placeholder="客户端连接的公网地址，例如：1.2.3.4"
        />
      </div>
      <div className="grid grid-cols-2 gap-3">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1.5">隧道端口</label>
          <input
            type="number"
            value={formData.tunnelPort}
            onChange={(e) => setFormData({ ...formData, tunnelPort: e.target.value })}
            className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white"
            placeholder="7000"
          />
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1.5">隧道协议</label>
          <select
            value={formData.tunnelProtocol}
            onChange={(e) => setFormData({ ...formData, tunnelProtocol: e.target.value })}
            className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white"
          >
            <option value="quic">QUIC</option>
            <option value="kcp">KCP</option>
          </select>
        </div>
      </div>
    </>
  );

  const copyToClipboard = (text: string, label: string) => {
    navigator.clipboard.writeText(text).then(() => {
      showToast(`${label}已复制到剪贴板`, 'success');
    }).catch(() => {
      showToast('复制失败，请手动复制', 'error');
    });
  };

  const getStartupCommand = (node?: Node | { name: string; secret: string }) => {
    if (!node) return '';
    const url = controllerUrl || `${window.location.hostname}:3100`;
    return `agent server --controller-url http://${url} --token ${node.secret}`;
  };

  const handleShowCommand = (node: Node) => {
    setCommandNode(node);
    setControllerUrl(`${window.location.hostname}:3100`);
    setShowCommandModal(true);
  };

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">节点管理</h2>
          <p className="mt-1 text-sm text-gray-500">管理和监控所有代理节点</p>
        </div>
        <button
          onClick={() => { resetForm(); setShowCreateModal(true); }}
          className="inline-flex items-center gap-2 px-5 py-2.5 bg-gradient-to-r from-blue-600 to-indigo-600 text-white text-sm font-medium rounded-xl hover:from-blue-700 hover:to-indigo-700 focus:outline-none focus:ring-2 focus:ring-blue-500/40 shadow-lg shadow-blue-500/25 transition-all duration-200"
        >
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-4 h-4">
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
          </svg>
          添加节点
        </button>
      </div>

      {loading ? (
        <TableSkeleton rows={3} cols={7} />
      ) : (
        <div className="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden">
          <table className="min-w-full">
            <thead>
              <tr className="bg-gradient-to-r from-gray-50 to-gray-100/50">
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  名称
                </th>
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  地区
                </th>
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  隧道地址
                </th>
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  协议
                </th>
                <th className="px-6 py-4 text-left text-xs font-semibold text-gray-600 uppercase tracking-wider">
                  状态
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
              {nodes.length === 0 ? (
                <tr>
                  <td colSpan={7} className="px-6 py-16 text-center">
                    <div className="flex flex-col items-center gap-3">
                      <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-8 h-8 text-gray-400">
                          <path strokeLinecap="round" strokeLinejoin="round" d="M5.25 14.25h13.5m-13.5 0a3 3 0 01-3-3m3 3a3 3 0 100 6h13.5a3 3 0 100-6m-16.5-3a3 3 0 013-3h13.5a3 3 0 013 3m-19.5 0a4.5 4.5 0 01.9-2.7L5.737 5.1a3.375 3.375 0 012.7-1.35h7.126c1.062 0 2.062.5 2.7 1.35l2.587 3.45a4.5 4.5 0 01.9 2.7m0 0a3 3 0 01-3 3m0 3h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008zm-3 6h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008z" />
                        </svg>
                      </div>
                      <p className="text-gray-500">暂无节点</p>
                      <p className="text-sm text-gray-400">点击"添加节点"来添加第一个节点</p>
                    </div>
                  </td>
                </tr>
              ) : (
                nodes.map((node) => (
                  <tr key={node.id} className="hover:bg-gray-50/50 transition-colors">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center gap-3">
                        <div className="w-10 h-10 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-xl flex items-center justify-center text-white text-sm font-semibold shadow-sm">
                          {node.name.charAt(0).toUpperCase()}
                        </div>
                        <div>
                          <div className="text-sm font-semibold text-gray-900">{node.name}</div>
                          {node.description && (
                            <div className="text-xs text-gray-500">{node.description}</div>
                          )}
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      {node.region ? (
                        <span className="inline-flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-lg bg-blue-50 text-blue-700">
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-3.5 h-3.5">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M15 10.5a3 3 0 11-6 0 3 3 0 016 0z" />
                            <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 10.5c0 7.142-7.5 11.25-7.5 11.25S4.5 17.642 4.5 10.5a7.5 7.5 0 1115 0z" />
                          </svg>
                          {node.region}
                        </span>
                      ) : (
                        <span className="text-xs text-gray-400">-</span>
                      )}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className="text-sm text-gray-600 font-mono">
                        {node.tunnelAddr ? `${node.tunnelAddr}:${node.tunnelPort}` : node.url}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className="inline-flex items-center px-2.5 py-1 rounded-lg text-xs font-semibold bg-gray-100 text-gray-700 uppercase">
                        {node.tunnelProtocol || 'quic'}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-semibold ${
                        node.isOnline
                          ? 'bg-gradient-to-r from-green-50 to-emerald-50 text-green-700'
                          : 'bg-red-50 text-red-700'
                      }`}>
                        <span className={`w-1.5 h-1.5 rounded-full ${node.isOnline ? 'bg-green-500' : 'bg-red-400'}`}></span>
                        {node.isOnline ? '在线' : '离线'}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {formatDate(node.created_at)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-right">
                      <div className="flex items-center justify-end gap-1">
                        <button
                          onClick={() => handleShowCommand(node)}
                          className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                        >
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-3.5 h-3.5">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M6.75 7.5l3 2.25-3 2.25m4.5 0h3m-9 8.25h13.5A2.25 2.25 0 0021 18V6a2.25 2.25 0 00-2.25-2.25H5.25A2.25 2.25 0 003 6v12a2.25 2.25 0 002.25 2.25z" />
                          </svg>
                          启动命令
                        </button>
                        <button
                          onClick={() => handleTestConnection(node)}
                          disabled={testingId === node.id}
                          className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-green-600 hover:bg-green-50 rounded-lg transition-colors disabled:opacity-50"
                        >
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-3.5 h-3.5">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M9.348 14.652a3.75 3.75 0 010-5.304m5.304 0a3.75 3.75 0 010 5.304m-7.425 2.121a6.75 6.75 0 010-9.546m9.546 0a6.75 6.75 0 010 9.546M5.106 18.894c-3.808-3.807-3.808-9.98 0-13.788m13.788 0c3.808 3.807 3.808 9.98 0 13.788M12 12h.008v.008H12V12zm.375 0a.375.375 0 11-.75 0 .375.375 0 01.75 0z" />
                          </svg>
                          {testingId === node.id ? '测试中...' : '测试'}
                        </button>
                        <button
                          onClick={() => openEditModal(node)}
                          className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-indigo-600 hover:bg-indigo-50 rounded-lg transition-colors"
                        >
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-3.5 h-3.5">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0115.75 21H5.25A2.25 2.25 0 013 18.75V8.25A2.25 2.25 0 015.25 6H10" />
                          </svg>
                          编辑
                        </button>
                        <button
                          onClick={() => handleDeleteNode(node)}
                          className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                        >
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-3.5 h-3.5">
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

      {/* 创建节点弹窗 */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm overflow-y-auto h-full w-full flex items-center justify-center z-50">
          <div className="relative bg-white rounded-2xl shadow-2xl w-full max-w-md mx-4 max-h-[90vh] overflow-y-auto transform transition-all">
            <div className="p-6">
              <div className="flex items-center gap-3 mb-6">
                <div className="w-10 h-10 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-xl flex items-center justify-center">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-5 h-5 text-white">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
                  </svg>
                </div>
                <div>
                  <h3 className="text-lg font-bold text-gray-900">添加节点</h3>
                  <p className="text-sm text-gray-500">配置新的代理节点</p>
                </div>
              </div>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">名称 *</label>
                  <input
                    type="text"
                    value={formData.name}
                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white"
                    placeholder="例如：US-East-1"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">节点密钥 (Token)</label>
                  <input
                    type="text"
                    value={formData.secret}
                    onChange={(e) => setFormData({ ...formData, secret: e.target.value })}
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white"
                    placeholder="留空则系统自动生成"
                  />
                  <p className="text-xs text-gray-500 mt-1.5">节点启动时通过此密钥向 Controller 注册，留空将自动生成</p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">地区</label>
                  <input
                    type="text"
                    value={formData.region}
                    onChange={(e) => setFormData({ ...formData, region: e.target.value })}
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white"
                    placeholder="例如：华东"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">描述</label>
                  <input
                    type="text"
                    value={formData.description}
                    onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white"
                    placeholder="可选描述"
                  />
                </div>
                {tunnelFields}
              </div>
              <div className="mt-6 flex gap-3">
                <button
                  onClick={() => setShowCreateModal(false)}
                  className="flex-1 px-4 py-2.5 bg-gray-100 text-gray-700 font-medium rounded-xl hover:bg-gray-200 transition-colors"
                >
                  取消
                </button>
                <button
                  onClick={handleCreateNode}
                  className="flex-1 px-4 py-2.5 bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-medium rounded-xl hover:from-blue-700 hover:to-indigo-700 shadow-lg shadow-blue-500/25 transition-all"
                >
                  创建
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 编辑节点弹窗 */}
      {showEditModal && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm overflow-y-auto h-full w-full flex items-center justify-center z-50">
          <div className="relative bg-white rounded-2xl shadow-2xl w-full max-w-md mx-4 max-h-[90vh] overflow-y-auto transform transition-all">
            <div className="p-6">
              <div className="flex items-center gap-3 mb-6">
                <div className="w-10 h-10 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-xl flex items-center justify-center">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-5 h-5 text-white">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0115.75 21H5.25A2.25 2.25 0 013 18.75V8.25A2.25 2.25 0 015.25 6H10" />
                  </svg>
                </div>
                <div>
                  <h3 className="text-lg font-bold text-gray-900">编辑节点</h3>
                  <p className="text-sm text-gray-500">修改节点配置信息</p>
                </div>
              </div>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">名称</label>
                  <input
                    type="text"
                    value={formData.name}
                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all bg-gray-50/50 hover:bg-white"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">内部 API 地址</label>
                  <input
                    type="text"
                    value={formData.url}
                    onChange={(e) => setFormData({ ...formData, url: e.target.value })}
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all bg-gray-50/50 hover:bg-white"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">共享密钥</label>
                  <input
                    type="password"
                    value={formData.secret}
                    onChange={(e) => setFormData({ ...formData, secret: e.target.value })}
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all bg-gray-50/50 hover:bg-white"
                    placeholder="留空则不修改"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">地区</label>
                  <input
                    type="text"
                    value={formData.region}
                    onChange={(e) => setFormData({ ...formData, region: e.target.value })}
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all bg-gray-50/50 hover:bg-white"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1.5">描述</label>
                  <input
                    type="text"
                    value={formData.description}
                    onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                    className="w-full px-4 py-3 border border-gray-200 rounded-xl text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all bg-gray-50/50 hover:bg-white"
                  />
                </div>
                {tunnelFields}
              </div>
              <div className="mt-6 flex gap-3">
                <button
                  onClick={() => { setShowEditModal(false); setEditingNode(null); }}
                  className="flex-1 px-4 py-2.5 bg-gray-100 text-gray-700 font-medium rounded-xl hover:bg-gray-200 transition-colors"
                >
                  取消
                </button>
                <button
                  onClick={handleEditNode}
                  className="flex-1 px-4 py-2.5 bg-gradient-to-r from-indigo-600 to-purple-600 text-white font-medium rounded-xl hover:from-indigo-700 hover:to-purple-700 shadow-lg shadow-indigo-500/25 transition-all"
                >
                  保存
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 启动命令弹窗 */}
      {showCommandModal && (createdNodeInfo || commandNode) && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm overflow-y-auto h-full w-full flex items-center justify-center z-50">
          <div className="relative bg-white rounded-2xl shadow-2xl w-full max-w-lg mx-4 max-h-[90vh] overflow-y-auto transform transition-all">
            <div className="p-6">
              <div className="flex items-center gap-3 mb-6">
                <div className="w-10 h-10 bg-gradient-to-br from-green-500 to-emerald-600 rounded-xl flex items-center justify-center">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-5 h-5 text-white">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M6.75 7.5l3 2.25-3 2.25m4.5 0h3m-9 8.25h13.5A2.25 2.25 0 0021 18V6a2.25 2.25 0 00-2.25-2.25H5.25A2.25 2.25 0 003 6v12a2.25 2.25 0 002.25 2.25z" />
                  </svg>
                </div>
                <div>
                  <h3 className="text-lg font-bold text-gray-900">
                    {createdNodeInfo ? '节点创建成功' : '节点启动命令'}
                  </h3>
                  <p className="text-sm text-gray-500">
                    {createdNodeInfo
                      ? '请在目标服务器上执行以下启动命令'
                      : `节点 "${commandNode?.name}" 的启动命令`}
                  </p>
                </div>
              </div>

              {/* Controller 地址 */}
              <div className="mb-4">
                <label className="block text-sm font-medium text-gray-700 mb-2">Controller 地址</label>
                <input
                  type="text"
                  value={controllerUrl}
                  onChange={(e) => setControllerUrl(e.target.value)}
                  placeholder="例如: 192.168.1.100:3100"
                  className="w-full px-4 py-2.5 border border-gray-200 rounded-xl text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all bg-gray-50/50 hover:bg-white font-mono text-sm"
                />
                <p className="mt-1.5 text-xs text-gray-500">
                  修改为 Agent 可以访问的 Controller 地址（IP:端口）
                </p>
              </div>

              {/* 启动命令 */}
              <div className="mb-4">
                <div className="flex items-center justify-between mb-2">
                  <label className="text-sm font-medium text-gray-700">启动命令</label>
                  <button
                    onClick={() => copyToClipboard(getStartupCommand(createdNodeInfo || commandNode!), '启动命令')}
                    className="inline-flex items-center gap-1.5 text-xs font-medium text-blue-600 hover:text-blue-800 transition-colors"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-3.5 h-3.5">
                      <path strokeLinecap="round" strokeLinejoin="round" d="M15.666 3.888A2.25 2.25 0 0013.5 2.25h-3c-1.03 0-1.9.693-2.166 1.638m7.332 0c.055.194.084.4.084.612v0a.75.75 0 01-.75.75H9.75a.75.75 0 01-.75-.75v0c0-.212.03-.418.084-.612m7.332 0c.646.049 1.288.11 1.927.184 1.1.128 1.907 1.077 1.907 2.185V19.5a2.25 2.25 0 01-2.25 2.25H6.75A2.25 2.25 0 014.5 19.5V6.257c0-1.108.806-2.057 1.907-2.185a48.208 48.208 0 011.927-.184" />
                    </svg>
                    复制
                  </button>
                </div>
                <pre className="bg-gray-900 text-green-400 rounded-xl px-4 py-3 text-sm font-mono overflow-x-auto">{getStartupCommand(createdNodeInfo || commandNode!)}</pre>
              </div>

              {/* 提示 */}
              <div className="bg-amber-50 border border-amber-200 rounded-xl px-4 py-3 mb-6">
                <div className="flex items-start gap-2">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={2} stroke="currentColor" className="w-4 h-4 text-amber-600 mt-0.5 flex-shrink-0">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
                  </svg>
                  <p className="text-sm text-amber-800">
                    <strong>提示：</strong>节点启动后会自动向 Controller 注册。
                    注册成功后可在节点列表中查看状态。
                  </p>
                </div>
              </div>

              <div className="flex justify-end">
                <button
                  onClick={() => {
                    setShowCommandModal(false);
                    setCreatedNodeInfo(null);
                    setCommandNode(null);
                  }}
                  className="px-5 py-2.5 bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-medium rounded-xl hover:from-blue-700 hover:to-indigo-700 shadow-lg shadow-blue-500/25 transition-all"
                >
                  知道了
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
        onConfirm={confirmDialog.onConfirm}
        onCancel={() => setConfirmDialog(prev => ({ ...prev, open: false }))}
      />
    </div>
  );
}
