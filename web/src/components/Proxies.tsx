import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Table,
  Button,
  Modal,
  Form,
  Input,
  Select,
  InputNumber,
  Tag,
  Space,
  message,
  Popconfirm,
  Card,
  Empty,
  Spin,
  Switch
} from 'antd';
import {
  PlusOutlined,
  DeleteOutlined,
  LinkOutlined,
  CheckCircleOutlined,
  CloseCircleOutlined
} from '@ant-design/icons';
import { proxyApi, clientApi, type Proxy, type Client } from '../api';

interface ProxiesProps {
  onRefresh?: () => void;
}

export function Proxies({ onRefresh }: ProxiesProps) {
  const { t } = useTranslation();
  const [proxies, setProxies] = useState<Proxy[]>([]);
  const [clients, setClients] = useState<Client[]>([]);
  const [loading, setLoading] = useState(true);
  const [showModal, setShowModal] = useState(false);
  const [form] = Form.useForm();

  const loadProxies = async () => {
    setLoading(true);
    try {
      const data = await proxyApi.list();
      setProxies(data);
    } catch (error) {
      console.error('Failed to load proxies:', error);
      message.error('Failed to load proxies');
    } finally {
      setLoading(false);
    }
  };

  const loadClients = async () => {
    try {
      const data = await clientApi.list();
      setClients(data);
    } catch (error) {
      console.error('Failed to load clients:', error);
    }
  };

  useEffect(() => {
    loadProxies();
    loadClients();
  }, []);

  const handleCreate = async () => {
    try {
      const values = await form.validateFields();
      await proxyApi.create({
        ...values,
        localPort: parseInt(values.localPort),
        remotePort: parseInt(values.remotePort),
      });
      message.success('Proxy created successfully');
      setShowModal(false);
      form.resetFields();
      loadProxies();
      onRefresh?.();
    } catch (error) {
      console.error('Failed to create proxy:', error);
      message.error('Failed to create proxy');
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await proxyApi.delete(id);
      message.success('Proxy deleted successfully');
      loadProxies();
      onRefresh?.();
    } catch (error) {
      console.error('Failed to delete proxy:', error);
      message.error('Failed to delete proxy');
    }
  };

  const handleToggle = async (proxy: Proxy) => {
    try {
      await proxyApi.update(proxy.id, { enabled: !proxy.enabled });
      loadProxies();
      onRefresh?.();
    } catch (error) {
      console.error('Failed to toggle proxy:', error);
      message.error('Failed to toggle proxy');
    }
  };

  const getClientName = (clientId: string) => {
    const client = clients.find((c) => c.id === clientId);
    return client?.name || 'Unknown';
  };

  const columns = [
    {
      title: t('proxies.proxyName'),
      dataIndex: 'name',
      key: 'name',
      render: (text: string) => (
        <Space>
          <LinkOutlined />
          <span className="font-medium">{text}</span>
        </Space>
      ),
    },
    {
      title: t('proxies.client'),
      dataIndex: 'client_id',
      key: 'client_id',
      render: (clientId: string) => getClientName(clientId),
    },
    {
      title: t('proxies.proxyType'),
      dataIndex: 'type',
      key: 'type',
      render: (type: string) => (
        <Tag color={type === 'tcp' ? 'blue' : 'orange'}>{type.toUpperCase()}</Tag>
      ),
    },
    {
      title: 'Target',
      key: 'target',
      render: (_: any, record: Proxy) => (
        <span className="font-mono text-sm">
          {record.localIP}:{record.localPort}
        </span>
      ),
    },
    {
      title: t('proxies.remotePort'),
      dataIndex: 'remotePort',
      key: 'remotePort',
      render: (port: number) => <span className="font-mono text-sm">{port}</span>,
    },
    {
      title: t('common.status'),
      dataIndex: 'enabled',
      key: 'enabled',
      render: (enabled: boolean, record: Proxy) => (
        <Switch
          checked={enabled}
          onChange={() => handleToggle(record)}
          checkedChildren={<CheckCircleOutlined />}
          unCheckedChildren={<CloseCircleOutlined />}
        />
      ),
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: any, record: Proxy) => (
        <Popconfirm
          title={t('proxies.deleteConfirm')}
          onConfirm={() => handleDelete(record.id)}
          okText="Yes"
          cancelText="No"
        >
          <Button danger icon={<DeleteOutlined />} size="small">
            {t('common.delete')}
          </Button>
        </Popconfirm>
      ),
    },
  ];

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Spin size="large" tip={t('common.loading')} />
      </div>
    );
  }

  return (
    <>
      <Card
        title={t('proxies.title')}
        extra={
          <Button
            type="primary"
            icon={<PlusOutlined />}
            onClick={() => setShowModal(true)}
          >
            {t('proxies.createProxy')}
          </Button>
        }
      >
        {proxies.length === 0 ? (
          <Empty description={t('proxies.noProxies')} />
        ) : (
          <Table
            dataSource={proxies}
            columns={columns}
            rowKey="id"
            pagination={false}
          />
        )}
      </Card>

      <Modal
        title={t('proxies.modalTitle')}
        open={showModal}
        onOk={handleCreate}
        onCancel={() => {
          setShowModal(false);
          form.resetFields();
        }}
        okText={t('common.create')}
        cancelText={t('common.cancel')}
        width={600}
      >
        <Form
          form={form}
          layout="vertical"
          initialValues={{
            type: 'tcp',
            localIP: '127.0.0.1',
          }}
        >
          <Form.Item
            name="client_id"
            label={t('proxies.client')}
            rules={[{ required: true, message: 'Please select a client' }]}
          >
            <Select placeholder={t('proxies.selectClient')}>
              {clients.map((client) => (
                <Select.Option key={client.id} value={client.id}>
                  {client.name}
                </Select.Option>
              ))}
            </Select>
          </Form.Item>

          <Form.Item
            name="name"
            label={t('proxies.proxyName')}
            rules={[{ required: true, message: 'Please enter proxy name' }]}
          >
            <Input placeholder="Enter proxy name" />
          </Form.Item>

          <Form.Item
            name="type"
            label={t('proxies.proxyType')}
            rules={[{ required: true }]}
          >
            <Select>
              <Select.Option value="tcp">TCP</Select.Option>
              <Select.Option value="udp">UDP</Select.Option>
            </Select>
          </Form.Item>

          <Form.Item
            name="localIP"
            label={t('proxies.targetHost')}
            rules={[{ required: true }]}
          >
            <Input />
          </Form.Item>

          <Space className="w-full">
            <Form.Item
              name="localPort"
              label={t('proxies.listenPort')}
              rules={[{ required: true }]}
              className="mb-0 flex-1"
            >
              <InputNumber min={1} max={65535} className="w-full" />
            </Form.Item>

            <Form.Item
              name="remotePort"
              label={t('proxies.targetPort')}
              rules={[{ required: true }]}
              className="mb-0 flex-1"
            >
              <InputNumber min={1} max={65535} className="w-full" />
            </Form.Item>
          </Space>
        </Form>
      </Modal>
    </>
  );
}
