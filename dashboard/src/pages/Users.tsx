import { useEffect, useState } from 'react';
import { userService, clientService } from '../lib/services';
import type { UserWithClientCount, Client } from '../lib/types';
import { formatDate } from '../lib/utils';

export default function Users() {
  const [users, setUsers] = useState<UserWithClientCount[]>([]);
  const [clients, setClients] = useState<Client[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showBindModal, setShowBindModal] = useState(false);
  const [selectedUser, setSelectedUser] = useState<UserWithClientCount | null>(null);
  const [userClients, setUserClients] = useState<Client[]>([]);
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' } | null>(null);
  const [formData, setFormData] = useState({
    username: '',
    password: '',
    is_admin: false,
  });

  useEffect(() => {
    loadUsers();
    loadClients();
  }, []);

  const loadUsers = async () => {
    try {
      setLoading(true);
      const response = await userService.getUsers();
      if (response.success && response.data) {
        setUsers(response.data);
      }
    } catch (error) {
      console.error('加载用户失败:', error);
      showToast('加载失败', 'error');
    } finally {
      setLoading(false);
    }
  };

  const loadClients = async () => {
    try {
      const response = await clientService.getClients();
      if (response.success && response.data) {
        setClients(response.data);
      }
    } catch (error) {
      console.error('加载客户端失败:', error);
    }
  };

  const resetForm = () => {
    setFormData({ username: '', password: '', is_admin: false });
  };

  const handleCreateUser = async () => {
    if (!formData.username) {
      showToast('请输入用户名', 'error');
      return;
    }

    try {
      const response = await userService.createUser({
        username: formData.username,
        password: formData.password || undefined,
        is_admin: formData.is_admin,
      });
      if (response.success) {
        showToast('用户创建成功', 'success');
        resetForm();
        setShowCreateModal(false);
        loadUsers();
      } else {
        showToast(response.message || '创建失败', 'error');
      }
    } catch (error) {
      console.error('创建用户失败:', error);
      showToast('创建失败', 'error');
    }
  };

  const handleDeleteUser = async (id: number) => {
    if (!confirm('确定要删除这个用户吗？')) return;

    try {
      const response = await userService.deleteUser(id);
      if (response.success) {
        showToast('用户删除成功', 'success');
        loadUsers();
      } else {
        showToast(response.message || '删除失败', 'error');
      }
    } catch (error) {
      console.error('删除用户失败:', error);
      showToast('删除失败', 'error');
    }
  };

  const handleToggleAdmin = async (user: UserWithClientCount) => {
    try {
      const response = await userService.updateUser(user.id, {
        is_admin: !user.is_admin,
      });
      if (response.success) {
        showToast(`用户已${user.is_admin ? '取消管理员权限' : '设为管理员'}`, 'success');
        loadUsers();
      } else {
        showToast(response.message || '更新失败', 'error');
      }
    } catch (error) {
      console.error('更新用户失败:', error);
      showToast('更新失败', 'error');
    }
  };

  const handleManageClients = async (user: UserWithClientCount) => {
    setSelectedUser(user);
    try {
      const response = await userService.getUserClients(user.id);
      if (response.success && response.data) {
        setUserClients(response.data);
      }
    } catch (error) {
      console.error('加载用户客户端失败:', error);
    }
    setShowBindModal(true);
  };

  const handleAssignClient = async (clientId: number) => {
    if (!selectedUser) return;

    try {
      const response = await userService.assignClient(selectedUser.id, clientId);
      if (response.success) {
        showToast('客户端绑定成功', 'success');
        // 重新加载用户的客户端列表
        const res = await userService.getUserClients(selectedUser.id);
        if (res.success && res.data) {
          setUserClients(res.data);
        }
        loadUsers();
      } else {
        showToast(response.message || '绑定失败', 'error');
      }
    } catch (error) {
      console.error('绑定客户端失败:', error);
      showToast('绑定失败', 'error');
    }
  };

  const handleRemoveClient = async (clientId: number) => {
    if (!selectedUser) return;

    try {
      const response = await userService.removeClient(selectedUser.id, clientId);
      if (response.success) {
        showToast('客户端解绑成功', 'success');
        const res = await userService.getUserClients(selectedUser.id);
        if (res.success && res.data) {
          setUserClients(res.data);
        }
        loadUsers();
      } else {
        showToast(response.message || '解绑失败', 'error');
      }
    } catch (error) {
      console.error('解绑客户端失败:', error);
      showToast('解绑失败', 'error');
    }
  };

  const showToast = (message: string, type: 'success' | 'error') => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  };

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">用户管理</h2>
          <p className="mt-1 text-sm text-gray-600">管理系统用户和权限</p>
        </div>
        <button
          onClick={() => {
            resetForm();
            setShowCreateModal(true);
          }}
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          新建用户
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
                  用户名
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  角色
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  绑定客户端
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
              {users.length === 0 ? (
                <tr>
                  <td colSpan={5} className="px-6 py-12 text-center text-gray-500">
                    暂无用户数据
                  </td>
                </tr>
              ) : (
                users.map((user) => (
                  <tr key={user.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                      {user.username}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      {user.is_admin ? (
                        <span className="px-2 py-1 text-xs font-medium rounded bg-blue-100 text-blue-800">
                          管理员
                        </span>
                      ) : (
                        <span className="px-2 py-1 text-xs font-medium rounded bg-gray-100 text-gray-800">
                          普通用户
                        </span>
                      )}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {user.client_count} 个
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                      {formatDate(user.created_at)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                      <button
                        onClick={() => handleManageClients(user)}
                        className="text-blue-600 hover:text-blue-900 mr-3"
                      >
                        管理客户端
                      </button>
                      <button
                        onClick={() => handleToggleAdmin(user)}
                        className="text-purple-600 hover:text-purple-900 mr-3"
                      >
                        {user.is_admin ? '取消管理员' : '设为管理员'}
                      </button>
                      <button
                        onClick={() => handleDeleteUser(user.id)}
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

      {/* 创建用户模态框 */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full flex items-center justify-center">
          <div className="relative p-5 border w-96 shadow-lg rounded-md bg-white">
            <h3 className="text-lg font-bold text-gray-900 mb-4">创建新用户</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700">用户名 *</label>
                <input
                  type="text"
                  value={formData.username}
                  onChange={(e) => setFormData({ ...formData, username: e.target.value })}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700">密码</label>
                <input
                  type="password"
                  value={formData.password}
                  onChange={(e) => setFormData({ ...formData, password: e.target.value })}
                  placeholder="留空自动生成"
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div className="flex items-center">
                <input
                  type="checkbox"
                  id="is_admin"
                  checked={formData.is_admin}
                  onChange={(e) => setFormData({ ...formData, is_admin: e.target.checked })}
                  className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                />
                <label htmlFor="is_admin" className="ml-2 block text-sm text-gray-900">
                  管理员
                </label>
              </div>
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
                onClick={handleCreateUser}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
              >
                创建
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 管理客户端绑定模态框 */}
      {showBindModal && selectedUser && (
        <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full flex items-center justify-center">
          <div className="relative p-5 border w-[600px] shadow-lg rounded-md bg-white max-h-[80vh] overflow-y-auto">
            <h3 className="text-lg font-bold text-gray-900 mb-4">
              管理用户客户端 - {selectedUser.username}
            </h3>

            <div className="space-y-4">
              <div>
                <h4 className="text-sm font-medium text-gray-700 mb-2">已绑定的客户端</h4>
                {userClients.length === 0 ? (
                  <p className="text-sm text-gray-500">暂无绑定客户端</p>
                ) : (
                  <div className="space-y-2">
                    {userClients.map((client) => (
                      <div
                        key={client.id}
                        className="flex items-center justify-between p-3 bg-gray-50 rounded-md"
                      >
                        <div className="flex items-center">
                          <span className={`h-2 w-2 rounded-full mr-2 ${
                            client.is_online ? 'bg-green-500' : 'bg-gray-400'
                          }`}></span>
                          <span className="text-sm font-medium">{client.name}</span>
                        </div>
                        <button
                          onClick={() => handleRemoveClient(client.id)}
                          className="text-red-600 hover:text-red-900 text-sm"
                        >
                          解绑
                        </button>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              <div>
                <h4 className="text-sm font-medium text-gray-700 mb-2">绑定新客户端</h4>
                <div className="space-y-2 max-h-48 overflow-y-auto">
                  {clients
                    .filter((c) => !userClients.find((uc) => uc.id === c.id))
                    .map((client) => (
                      <div
                        key={client.id}
                        className="flex items-center justify-between p-3 bg-gray-50 rounded-md hover:bg-gray-100"
                      >
                        <div className="flex items-center">
                          <span className={`h-2 w-2 rounded-full mr-2 ${
                            client.is_online ? 'bg-green-500' : 'bg-gray-400'
                          }`}></span>
                          <span className="text-sm font-medium">{client.name}</span>
                        </div>
                        <button
                          onClick={() => handleAssignClient(client.id)}
                          className="text-blue-600 hover:text-blue-900 text-sm"
                        >
                          绑定
                        </button>
                      </div>
                    ))}
                </div>
              </div>
            </div>

            <div className="mt-6 flex justify-end">
              <button
                onClick={() => {
                  setShowBindModal(false);
                  setSelectedUser(null);
                  setUserClients([]);
                }}
                className="px-4 py-2 bg-gray-200 text-gray-800 rounded-md hover:bg-gray-300"
              >
                关闭
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
