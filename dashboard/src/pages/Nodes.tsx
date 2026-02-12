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
    if (!formData.name.trim() || !formData.secret.trim()) {
      showToast('请填写名称和密钥', 'error');
      return;
    }

    try {
      const response = await nodeService.createNode({
        name: formData.name,
        url: formData.url || '',
        secret: formData.secret,
        region: formData.region || undefined,
        description: formData.description || undefined,
        tunnelAddr: formData.tunnelAddr || undefined,
        tunnelPort: formData.tunnelPort ? parseInt(formData.tunnelPort) : undefined,
        tunnelProtocol: formData.tunnelProtocol || undefined,
      });
      if (response.success) {
        showToast('节点创建成功', 'success');
        setCreatedNodeInfo({
          name: formData.name,
          secret: formData.secret,
        });
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
      <div className="border-t border-gray-200 pt-4 mt-4">
        <h3 className="text-sm font-medium text-gray-900 mb-3">隧道连接配置</h3>
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">隧道地址 *</label>
        <input
          type="text"
          value={formData.tunnelAddr}
          onChange={(e) => setFormData({ ...formData, tunnelAddr: e.target.value })}
          className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
          placeholder="客户端连接的公网地址，例如：1.2.3.4"
        />
      </div>
      <div className="grid grid-cols-2 gap-3">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">隧道端口</label>
          <input
            type="number"
            value={formData.tunnelPort}
            onChange={(e) => setFormData({ ...formData, tunnelPort: e.target.value })}
            className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
            placeholder="7000"
          />
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">隧道协议</label>
          <select
            value={formData.tunnelProtocol}
            onChange={(e) => setFormData({ ...formData, tunnelProtocol: e.target.value })}
            className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
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

  const getStartupCommand = () => {
    if (!createdNodeInfo) return '';
    const controllerHost = window.location.hostname;
    return `agent server --controller-url http://${controllerHost}:3100 --token ${createdNodeInfo.secret}`;
  };

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-gray-900">节点管理</h1>
        <button
          onClick={() => { resetForm(); setShowCreateModal(true); }}
          className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors"
        >
          添加节点
        </button>
      </div>

      {loading ? (
        <TableSkeleton rows={3} cols={6} />
      ) : nodes.length === 0 ? (
        <div className="text-center py-12 text-gray-500">
          <p className="text-lg">暂无节点</p>
          <p className="text-sm mt-2">点击"添加节点"来添加第一个节点</p>
        </div>
      ) : (
        <div className="bg-white shadow rounded-lg overflow-hidden">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">名称</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">地区</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">隧道地址</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">协议</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">状态</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">创建时间</th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">操作</th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {nodes.map((node) => (
                <tr key={node.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm font-medium text-gray-900">{node.name}</div>
                    {node.description && (
                      <div className="text-xs text-gray-500">{node.description}</div>
                    )}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {node.region || '-'}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 font-mono">
                    {node.tunnelAddr ? `${node.tunnelAddr}:${node.tunnelPort}` : node.url}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800 uppercase">
                      {node.tunnelProtocol || 'quic'}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                      node.isOnline
                        ? 'bg-green-100 text-green-800'
                        : 'bg-red-100 text-red-800'
                    }`}>
                      {node.isOnline ? '在线' : '离线'}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {formatDate(node.created_at)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-2">
                    <button
                      onClick={() => handleTestConnection(node)}
                      disabled={testingId === node.id}
                      className="text-green-600 hover:text-green-900 disabled:opacity-50"
                    >
                      {testingId === node.id ? '测试中...' : '测试'}
                    </button>
                    <button
                      onClick={() => openEditModal(node)}
                      className="text-indigo-600 hover:text-indigo-900"
                    >
                      编辑
                    </button>
                    <button
                      onClick={() => handleDeleteNode(node)}
                      className="text-red-600 hover:text-red-900"
                    >
                      删除
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* 创建节点弹窗 */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-md max-h-[90vh] overflow-y-auto">
            <h2 className="text-lg font-bold mb-4">添加节点</h2>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">名称 *</label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="例如：US-East-1"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">节点密钥 (Token) *</label>
                <input
                  type="text"
                  value={formData.secret}
                  onChange={(e) => setFormData({ ...formData, secret: e.target.value })}
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="节点注册密钥，用于 --token 参数"
                />
                <p className="text-xs text-gray-500 mt-1">节点启动时通过此密钥向 Controller 注册</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">地区</label>
                <input
                  type="text"
                  value={formData.region}
                  onChange={(e) => setFormData({ ...formData, region: e.target.value })}
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="例如：华东"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">描述</label>
                <input
                  type="text"
                  value={formData.description}
                  onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="可选描述"
                />
              </div>
              {tunnelFields}
            </div>
            <div className="flex justify-end space-x-3 mt-6">
              <button
                onClick={() => setShowCreateModal(false)}
                className="px-4 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200"
              >
                取消
              </button>
              <button
                onClick={handleCreateNode}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
              >
                创建
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 编辑节点弹窗 */}
      {showEditModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-md max-h-[90vh] overflow-y-auto">
            <h2 className="text-lg font-bold mb-4">编辑节点</h2>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">名称</label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">内部 API 地址</label>
                <input
                  type="text"
                  value={formData.url}
                  onChange={(e) => setFormData({ ...formData, url: e.target.value })}
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">共享密钥</label>
                <input
                  type="password"
                  value={formData.secret}
                  onChange={(e) => setFormData({ ...formData, secret: e.target.value })}
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="留空则不修改"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">地区</label>
                <input
                  type="text"
                  value={formData.region}
                  onChange={(e) => setFormData({ ...formData, region: e.target.value })}
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">描述</label>
                <input
                  type="text"
                  value={formData.description}
                  onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              {tunnelFields}
            </div>
            <div className="flex justify-end space-x-3 mt-6">
              <button
                onClick={() => { setShowEditModal(false); setEditingNode(null); }}
                className="px-4 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200"
              >
                取消
              </button>
              <button
                onClick={handleEditNode}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
              >
                保存
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 启动命令弹窗 */}
      {showCommandModal && createdNodeInfo && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-lg max-h-[90vh] overflow-y-auto">
            <h2 className="text-lg font-bold mb-2">节点创建成功</h2>
            <p className="text-sm text-gray-600 mb-4">
              请在目标服务器上执行以下启动命令。
            </p>

            {/* 启动命令 */}
            <div className="mb-4">
              <div className="flex items-center justify-between mb-1">
                <label className="text-sm font-medium text-gray-700">启动命令</label>
                <button
                  onClick={() => copyToClipboard(getStartupCommand(), '启动命令')}
                  className="text-xs text-blue-600 hover:text-blue-800"
                >
                  复制
                </button>
              </div>
              <pre className="bg-gray-900 text-green-400 rounded-lg px-4 py-3 text-sm font-mono overflow-x-auto">{getStartupCommand()}</pre>
            </div>

            {/* 提示 */}
            <div className="bg-yellow-50 border border-yellow-200 rounded-lg px-4 py-3 mb-4">
              <p className="text-sm text-yellow-800">
                <strong>提示：</strong>节点启动后会自动向 Controller 注册。
                注册成功后可在节点列表中查看状态。
              </p>
            </div>

            <div className="flex justify-end">
              <button
                onClick={() => { setShowCommandModal(false); setCreatedNodeInfo(null); }}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
              >
                知道了
              </button>
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
