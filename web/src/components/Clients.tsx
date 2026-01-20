import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Table,
  Button,
  Modal,
  Form,
  Input,
  Tag,
  Space,
  Typography,
  message,
  Popconfirm,
  Card,
  Empty,
  Spin
} from 'antd';
import {
  PlusOutlined,
  DeleteOutlined,
  CloudServerOutlined,
  CopyOutlined
} from '@ant-design/icons';
import { clientApi, type Client } from '../api';

const { Text, Paragraph } = Typography;

interface ClientsProps {
  onRefresh?: () => void;
}

export function Clients({ onRefresh }: ClientsProps) {
  const { t } = useTranslation();
  const [clients, setClients] = useState<Client[]>([]);
  const [loading, setLoading] = useState(true);
  const [showModal, setShowModal] = useState(false);
  const [form] = Form.useForm();

  const loadClients = async () => {
    setLoading(true);
    try {
      const data = await clientApi.list();
      setClients(data);
    } catch (error) {
      console.error('Failed to load clients:', error);
      message.error('Failed to load clients');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadClients();
    const interval = setInterval(loadClients, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleCreate = async () => {
    try {
      const values = await form.validateFields();
      await clientApi.create({
        name: values.name,
        token: values.token || undefined,
      });
      message.success('Client created successfully');
      setShowModal(false);
      form.resetFields();
      loadClients();
      onRefresh?.();
    } catch (error) {
      console.error('Failed to create client:', error);
      message.error('Failed to create client');
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await clientApi.delete(id);
      message.success('Client deleted successfully');
      loadClients();
      onRefresh?.();
    } catch (error) {
      console.error('Failed to delete client:', error);
      message.error('Failed to delete client');
    }
  };

  const copyToken = (token: string) => {
    navigator.clipboard.writeText(token);
    message.success('Token copied to clipboard');
  };

  const columns = [
    {
      title: t('clients.clientName'),
      dataIndex: 'name',
      key: 'name',
      render: (text: string) => <Text strong>{text}</Text>,
    },
    {
      title: 'Token',
      dataIndex: 'token',
      key: 'token',
      render: (token: string) => (
        <Space>
          <Paragraph code copyable={{ text: token, tooltips: ['Copy token', 'Copied!'] }}>
            {token.slice(0, 12)}...
          </Paragraph>
          <Button
            type="text"
            icon={<CopyOutlined />}
            onClick={() => copyToken(token)}
          />
        </Space>
      ),
    },
    {
      title: t('common.status'),
      dataIndex: 'is_online',
      key: 'is_online',
      render: (isOnline: boolean) => (
        <Tag
          color={isOnline ? 'success' : 'default'}
          icon={isOnline ? <CloudServerOutlined /> : undefined}
        >
          {isOnline ? t('common.online') : t('common.offline')}
        </Tag>
      ),
    },
    {
      title: t('clients.created_at'),
      dataIndex: 'created_at',
      key: 'created_at',
      render: (date: string) => new Date(date).toLocaleString(),
    },
    {
      title: t('common.actions'),
      key: 'actions',
      render: (_: any, record: Client) => (
        <Popconfirm
          title={t('clients.deleteConfirm')}
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
        title={t('clients.title')}
        extra={
          <Button
            type="primary"
            icon={<PlusOutlined />}
            onClick={() => setShowModal(true)}
          >
            {t('clients.createClient')}
          </Button>
        }
      >
        {clients.length === 0 ? (
          <Empty description={t('clients.noClients')} />
        ) : (
          <Table
            dataSource={clients}
            columns={columns}
            rowKey="id"
            pagination={false}
          />
        )}
      </Card>

      <Modal
        title={t('clients.modalTitle')}
        open={showModal}
        onOk={handleCreate}
        onCancel={() => {
          setShowModal(false);
          form.resetFields();
        }}
        okText={t('common.create')}
        cancelText={t('common.cancel')}
      >
        <Form form={form} layout="vertical">
          <Form.Item
            name="name"
            label={t('clients.clientName')}
            rules={[{ required: true, message: 'Please enter client name' }]}
          >
            <Input placeholder={t('clients.namePlaceholder')} />
          </Form.Item>
          <Form.Item
            name="token"
            label="Token (Optional)"
          >
            <Input placeholder="Leave empty to auto-generate" />
          </Form.Item>
        </Form>
      </Modal>
    </>
  );
}
