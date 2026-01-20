import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Table,
  Button,
  Modal,
  Form,
  Input,
  Checkbox,
  Tag,
  Space,
  message,
  Popconfirm,
  Card,
  Empty,
  Spin,
  List,
  Switch
} from 'antd';
import {
  PlusOutlined,
  DeleteOutlined,
  EditOutlined,
  UserOutlined,
  SafetyOutlined,
  TeamOutlined
} from '@ant-design/icons';
import { userApi, clientApi, type User, type Client } from '../api';
import { useAuth } from '../contexts/AuthContext';

interface UserWithClients extends User {
  clients?: Client[];
}

export const Users = () => {
  const { t } = useTranslation();
  const { isAdmin } = useAuth();
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showClientModal, setShowClientModal] = useState(false);
  const [selectedUser, setSelectedUser] = useState<UserWithClients | null>(null);
  const [createForm] = Form.useForm();
  const [allClients, setAllClients] = useState<Client[]>([]);
  const [assignedClients, setAssignedClients] = useState<string[]>([]);
  const [error, setError] = useState('');

  useEffect(() => {
    if (isAdmin) {
      fetchUsers();
    }
  }, [isAdmin]);

  const fetchUsers = async () => {
    try {
      setLoading(true);
      const data = await userApi.list();
      setUsers(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch users');
    } finally {
      setLoading(false);
    }
  };

  const fetchUserClients = async (user: User) => {
    try {
      const clients = await userApi.getClients(user.id);
      setSelectedUser({ ...user, clients });
      setAssignedClients(clients.map(c => c.id));
    } catch (err) {
      message.error('Failed to fetch user clients');
    }
  };

  const handleCreateUser = async () => {
    try {
      const values = await createForm.validateFields();
      const result = await userApi.create({
        username: values.username,
        password: values.password || undefined,
        is_admin: values.is_admin,
      });
      if (result.generated_password) {
        Modal.success({
          title: t('users.userCreated'),
          content: (
            <div>
              <p><strong>{t('auth.username')}:</strong> {result.username}</p>
              <p><strong>{t('users.generatedPassword')}:</strong> {result.generated_password}</p>
              <p className="text-orange-500 mt-2">{t('users.savePasswordWarning')}</p>
            </div>
          ),
        });
      }
      createForm.resetFields();
      setShowCreateModal(false);
      fetchUsers();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create user');
    }
  };

  const handleDeleteUser = async (userId: number) => {
    try {
      await userApi.delete(userId);
      message.success('User deleted successfully');
      fetchUsers();
    } catch (err) {
      message.error('Failed to delete user');
    }
  };

  const handleToggleAdmin = async (user: User) => {
    try {
      await userApi.update(user.id, { is_admin: !user.is_admin });
      fetchUsers();
    } catch (err) {
      message.error('Failed to update user');
    }
  };

  const handleManageClients = async (user: User) => {
    await fetchUserClients(user);
    try {
      const clients = await clientApi.list();
      setAllClients(clients);
      setShowClientModal(true);
    } catch (err) {
      message.error('Failed to fetch clients');
    }
  };

  const handleToggleClient = async (clientId: string) => {
    if (!selectedUser) return;

    try {
      if (assignedClients.includes(clientId)) {
        await userApi.removeClient(selectedUser.id, clientId);
        setAssignedClients(assignedClients.filter(id => id !== clientId));
      } else {
        await userApi.assignClient(selectedUser.id, clientId);
        setAssignedClients([...assignedClients, clientId]);
      }
      await fetchUserClients(selectedUser);
    } catch (err) {
      message.error('Failed to update client assignment');
    }
  };

  const columns = [
    {
      title: 'ID',
      dataIndex: 'id',
      key: 'id',
      width: 80,
    },
    {
      title: t('users.username'),
      dataIndex: 'username',
      key: 'username',
      render: (text: string) => (
        <Space>
          <UserOutlined />
          <span className="font-medium">{text}</span>
        </Space>
      ),
    },
    {
      title: t('users.role'),
      dataIndex: 'is_admin',
      key: 'is_admin',
      render: (isAdmin: boolean) => (
        <Tag
          color={isAdmin ? 'blue' : 'default'}
          icon={isAdmin ? <SafetyOutlined /> : <UserOutlined />}
        >
          {isAdmin ? t('users.admin') : t('users.user')}
        </Tag>
      ),
    },
    {
      title: t('users.clients'),
      dataIndex: 'client_count',
      key: 'client_count',
      render: (count: number) => (
        <Tag color="green" icon={<TeamOutlined />}>
          {count || 0}
        </Tag>
      ),
    },
    {
      title: t('users.created'),
      dataIndex: 'created_at',
      key: 'created_at',
      render: (date: string) => new Date(date).toLocaleDateString(),
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: any, record: User) => (
        <Space size="small">
          <Button
            icon={<EditOutlined />}
            size="small"
            onClick={() => handleManageClients(record)}
          >
            {t('users.manageClients')}
          </Button>
          <Button
            icon={<SafetyOutlined />}
            size="small"
            onClick={() => handleToggleAdmin(record)}
            disabled={record.username === 'admin'}
          >
            {record.is_admin ? t('users.makeUser') : t('users.makeAdmin')}
          </Button>
          <Popconfirm
            title={t('users.deleteConfirm')}
            onConfirm={() => handleDeleteUser(record.id)}
            okText="Yes"
            cancelText="No"
          >
            <Button
              danger
              icon={<DeleteOutlined />}
              size="small"
              disabled={record.username === 'admin'}
            >
              {t('common.delete')}
            </Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  if (!isAdmin) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Empty
          description={
            <div className="text-center">
              <h2>{t('auth.accessDenied')}</h2>
              <p>{t('auth.noPermission')}</p>
            </div>
          }
        />
      </div>
    );
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Spin size="large" tip={t('common.loading')} />
      </div>
    );
  }

  return (
    <Space direction="vertical" size="large" className="w-full">
      {error && (
        <Card>
          <p className="text-red-500">{error}</p>
        </Card>
      )}

      <Card
        title={t('users.title')}
        extra={
          <Button
            type="primary"
            icon={<PlusOutlined />}
            onClick={() => setShowCreateModal(true)}
          >
            {t('users.createUser')}
          </Button>
        }
      >
        <Table
          dataSource={users}
          columns={columns}
          rowKey="id"
          pagination={false}
        />
      </Card>

      {/* Create User Modal */}
      <Modal
        title={t('users.modalTitle')}
        open={showCreateModal}
        onOk={handleCreateUser}
        onCancel={() => {
          setShowCreateModal(false);
          createForm.resetFields();
        }}
        okText={t('common.create')}
        cancelText={t('common.cancel')}
      >
        <Form
          form={createForm}
          layout="vertical"
        >
          <Form.Item
            name="username"
            label={t('users.username')}
            rules={[{ required: true, message: 'Please enter username' }]}
          >
            <Input placeholder={t('users.usernamePlaceholder')} />
          </Form.Item>
          <Form.Item
            name="password"
            label={t('auth.password')}
          >
            <Input.Password placeholder={t('users.passwordPlaceholder')} />
          </Form.Item>
          <Form.Item
            name="is_admin"
            valuePropName="checked"
          >
            <Checkbox>{t('users.isAdmin')}</Checkbox>
          </Form.Item>
        </Form>
      </Modal>

      {/* Manage Clients Modal */}
      <Modal
        title={t('users.manageClientsTitle', { username: selectedUser?.username })}
        open={showClientModal}
        onCancel={() => {
          setShowClientModal(false);
          setSelectedUser(null);
          fetchUsers();
        }}
        footer={[
          <Button
            key="close"
            type="primary"
            onClick={() => {
              setShowClientModal(false);
              setSelectedUser(null);
              fetchUsers();
            }}
          >
            {t('common.close')}
          </Button>
        ]}
        width={600}
      >
        <List
          dataSource={allClients}
          renderItem={(client) => (
            <List.Item
              actions={[
                <Switch
                  checked={assignedClients.includes(client.id)}
                  onChange={() => handleToggleClient(client.id)}
                />
              ]}
            >
              <List.Item.Meta
                title={<span className="font-medium">{client.name}</span>}
                description={
                  <Space>
                    <span className="text-xs text-gray-500">{client.id}</span>
                    <Tag
                      color={client.is_online ? 'success' : 'default'}
                    >
                      {client.is_online ? t('common.online') : t('common.offline')}
                    </Tag>
                  </Space>
                }
              />
            </List.Item>
          )}
        />
      </Modal>
    </Space>
  );
};
