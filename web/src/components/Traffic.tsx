import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Card,
  Row,
  Col,
  Statistic,
  Table,
  Select,
  Typography,
  Space,
  Spin,
  Empty
} from 'antd';
import {
  ArrowUpOutlined,
  ArrowDownOutlined,
  BarChartOutlined
} from '@ant-design/icons';
import { trafficApi, formatBytes, type TrafficOverview } from '../api';
import { useAuth } from '../contexts/AuthContext';

const { Title } = Typography;

interface TrafficProps {
  onRefresh?: () => void;
}

export function Traffic({ }: TrafficProps) {
  const { t } = useTranslation();
  const { user } = useAuth();
  const [traffic, setTraffic] = useState<TrafficOverview | null>(null);
  const [loading, setLoading] = useState(true);
  const [days, setDays] = useState(30);

  const loadTraffic = async () => {
    setLoading(true);
    try {
      const data = user?.is_admin
        ? await trafficApi.getOverview(days)
        : await trafficApi.getUserTraffic(user!.id, days);
      setTraffic(data);
    } catch (error) {
      console.error('Failed to load traffic:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadTraffic();
    const interval = setInterval(loadTraffic, 30000);
    return () => clearInterval(interval);
  }, [days, user?.id]);

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Spin size="large" tip={t('common.loading')} />
      </div>
    );
  }

  if (!traffic) {
    return <Empty description="No traffic data available" />;
  }

  const proxyColumns = [
    {
      title: 'Proxy Name',
      dataIndex: 'proxy_name',
      key: 'proxy_name',
      render: (text: string) => <span className="font-medium">{text}</span>,
    },
    {
      title: 'Client',
      dataIndex: 'client_name',
      key: 'client_name',
    },
    {
      title: 'Sent',
      dataIndex: 'total_bytes_sent',
      key: 'total_bytes_sent',
      render: (bytes: number) => formatBytes(bytes),
      align: 'right' as const,
    },
    {
      title: 'Received',
      dataIndex: 'total_bytes_received',
      key: 'total_bytes_received',
      render: (bytes: number) => formatBytes(bytes),
      align: 'right' as const,
    },
    {
      title: 'Total',
      dataIndex: 'total_bytes',
      key: 'total_bytes',
      render: (bytes: number) => <strong>{formatBytes(bytes)}</strong>,
      align: 'right' as const,
    },
  ];

  const clientColumns = [
    {
      title: 'Client Name',
      dataIndex: 'client_name',
      key: 'client_name',
      render: (text: string) => <span className="font-medium">{text}</span>,
    },
    {
      title: 'Sent',
      dataIndex: 'total_bytes_sent',
      key: 'total_bytes_sent',
      render: (bytes: number) => formatBytes(bytes),
      align: 'right' as const,
    },
    {
      title: 'Received',
      dataIndex: 'total_bytes_received',
      key: 'total_bytes_received',
      render: (bytes: number) => formatBytes(bytes),
      align: 'right' as const,
    },
    {
      title: 'Total',
      dataIndex: 'total_bytes',
      key: 'total_bytes',
      render: (bytes: number) => <strong>{formatBytes(bytes)}</strong>,
      align: 'right' as const,
    },
  ];

  const userColumns = [
    {
      title: 'Username',
      dataIndex: 'username',
      key: 'username',
      render: (text: string) => <span className="font-medium">{text}</span>,
    },
    {
      title: 'Sent',
      dataIndex: 'total_bytes_sent',
      key: 'total_bytes_sent',
      render: (bytes: number) => formatBytes(bytes),
      align: 'right' as const,
    },
    {
      title: 'Received',
      dataIndex: 'total_bytes_received',
      key: 'total_bytes_received',
      render: (bytes: number) => formatBytes(bytes),
      align: 'right' as const,
    },
    {
      title: 'Total',
      dataIndex: 'total_bytes',
      key: 'total_bytes',
      render: (bytes: number) => <strong>{formatBytes(bytes)}</strong>,
      align: 'right' as const,
    },
  ];

  const dailyColumns = [
    {
      title: 'Date',
      dataIndex: 'date',
      key: 'date',
      render: (text: string) => <span className="font-mono text-sm">{text}</span>,
    },
    {
      title: 'Sent',
      dataIndex: 'total_bytes_sent',
      key: 'total_bytes_sent',
      render: (bytes: number) => formatBytes(bytes),
      align: 'right' as const,
    },
    {
      title: 'Received',
      dataIndex: 'total_bytes_received',
      key: 'total_bytes_received',
      render: (bytes: number) => formatBytes(bytes),
      align: 'right' as const,
    },
    {
      title: 'Total',
      dataIndex: 'total_bytes',
      key: 'total_bytes',
      render: (bytes: number) => <strong>{formatBytes(bytes)}</strong>,
      align: 'right' as const,
    },
  ];

  return (
    <Space direction="vertical" size="large" className="w-full">
      {/* Header with Select */}
      <div className="flex justify-between items-center">
        <Title level={3} className="!mb-0">
          {user?.is_admin ? t('traffic.title') : `${t('traffic.title')} - ${user?.username}`}
        </Title>
        <Select
          value={days}
          onChange={setDays}
          style={{ width: 150 }}
        >
          <Select.Option value={7}>Last 7 days</Select.Option>
          <Select.Option value={30}>Last 30 days</Select.Option>
          <Select.Option value={90}>Last 90 days</Select.Option>
        </Select>
      </div>

      {/* Total Traffic Card */}
      <Card className="bg-gradient-to-r from-primary-500 to-purple-600 border-0">
        <Title level={4} className="!mb-4 !text-white">
          {user?.is_admin ? 'Total Traffic' : 'My Traffic'}
        </Title>
        <Row gutter={16}>
          <Col xs={24} sm={8}>
            <Statistic
              title={<span className="text-white/80">Sent</span>}
              value={formatBytes(traffic.total_traffic.total_bytes_sent)}
              valueStyle={{ color: 'white' }}
              prefix={<ArrowUpOutlined />}
            />
          </Col>
          <Col xs={24} sm={8}>
            <Statistic
              title={<span className="text-white/80">Received</span>}
              value={formatBytes(traffic.total_traffic.total_bytes_received)}
              valueStyle={{ color: 'white' }}
              prefix={<ArrowDownOutlined />}
            />
          </Col>
          <Col xs={24} sm={8}>
            <Statistic
              title={<span className="text-white/80">Total</span>}
              value={formatBytes(traffic.total_traffic.total_bytes)}
              valueStyle={{ color: 'white' }}
              prefix={<BarChartOutlined />}
            />
          </Col>
        </Row>
      </Card>

      {/* Traffic by Proxy */}
      <Card title={<Title level={4} className="!mb-0">Traffic by Proxy</Title>}>
        {traffic.by_proxy.length === 0 ? (
          <Empty description="No proxy traffic data" />
        ) : (
          <Table
            dataSource={traffic.by_proxy}
            columns={proxyColumns}
            rowKey="proxy_id"
            pagination={false}
            size="small"
          />
        )}
      </Card>

      {/* Traffic by Client */}
      <Card title={<Title level={4} className="!mb-0">Traffic by Client</Title>}>
        {traffic.by_client.length === 0 ? (
          <Empty description="No client traffic data" />
        ) : (
          <Table
            dataSource={traffic.by_client}
            columns={clientColumns}
            rowKey="client_id"
            pagination={false}
            size="small"
          />
        )}
      </Card>

      {/* Traffic by User - Only show for admin */}
      {user?.is_admin && traffic.by_user.length > 0 && (
        <Card title={<Title level={4} className="!mb-0">Traffic by User</Title>}>
          <Table
            dataSource={traffic.by_user}
            columns={userColumns}
            rowKey="user_id"
            pagination={false}
            size="small"
          />
        </Card>
      )}

      {/* Daily Traffic */}
      <Card title={<Title level={4} className="!mb-0">Daily Traffic (Last {days} Days)</Title>}>
        {traffic.daily_traffic.length === 0 ? (
          <Empty description="No daily traffic data" />
        ) : (
          <Table
            dataSource={traffic.daily_traffic}
            columns={dailyColumns}
            rowKey={(record) => record.date}
            pagination={{ pageSize: 10 }}
            size="small"
          />
        )}
      </Card>
    </Space>
  );
}
