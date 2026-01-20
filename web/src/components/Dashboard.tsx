import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Card, Row, Col, Statistic, Button, Space, Typography, Spin, Empty } from 'antd';
import {
  UserOutlined,
  LinkOutlined,
  BarChartOutlined,
  ArrowUpOutlined,
  ArrowDownOutlined} from '@ant-design/icons';
import { dashboardApi, formatBytes, type DashboardStats } from '../api';
import { useAuth } from '../contexts/AuthContext';

const { Title } = Typography;

interface DashboardProps {
  onRefresh?: () => void;
}

export function Dashboard({ }: DashboardProps) {
  const { t } = useTranslation();
  const { user } = useAuth();
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [loading, setLoading] = useState(true);

  const loadStats = async () => {
    if (!user) return;

    setLoading(true);
    try {
      const data = await dashboardApi.getStats(user.id);
      setStats(data);
    } catch (error) {
      console.error('Failed to load dashboard stats:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadStats();
    const interval = setInterval(loadStats, 30000);
    return () => clearInterval(interval);
  }, [user?.id]);

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Spin size="large" tip={t('common.loading')} />
      </div>
    );
  }

  if (!stats || !user) {
    return (
      <Empty description="No dashboard data available" />
    );
  }

  return (
    <Space direction="vertical" size="large" className="w-full">
      {/* Welcome Card */}
      <Card className="bg-gradient-to-r from-primary-500 to-purple-600 border-0">
        <Title level={2} className="!mb-2 !text-white">
          Welcome back, {user.username}!
        </Title>
        <p className="text-white/80 text-lg mb-0">
          {user.is_admin ? 'Administrator' : 'User'}
        </p>
      </Card>

      {/* Statistics Cards */}
      <Row gutter={[16, 16]}>
        <Col xs={24} sm={12} lg={8}>
          <Card>
            <Statistic
              title={<span className="text-gray-600">Total Clients</span>}
              value={stats.total_clients}
              prefix={<UserOutlined className="text-blue-500" />}
              suffix={
                <span className="text-sm text-gray-500">
                  <span className="text-green-500 font-semibold">{stats.online_clients}</span> online
                </span>
              }
            />
          </Card>
        </Col>

        <Col xs={24} sm={12} lg={8}>
          <Card>
            <Statistic
              title={<span className="text-gray-600">Total Proxies</span>}
              value={stats.total_proxies}
              prefix={<LinkOutlined className="text-purple-500" />}
              suffix={
                <span className="text-sm text-gray-500">
                  <span className="text-green-500 font-semibold">{stats.enabled_proxies}</span> enabled
                </span>
              }
            />
          </Card>
        </Col>

        <Col xs={24} sm={12} lg={8}>
          <Card>
            <Statistic
              title={<span className="text-gray-600">My Traffic</span>}
              value={formatBytes(stats.user_traffic.total_bytes)}
              prefix={<BarChartOutlined className="text-green-500" />}
            />
            <div className="mt-2 text-sm text-gray-600">
              <ArrowUpOutlined className="text-blue-500" /> {formatBytes(stats.user_traffic.total_bytes_sent)} sent /{' '}
              <ArrowDownOutlined className="text-purple-500" /> {formatBytes(stats.user_traffic.total_bytes_received)} received
            </div>
          </Card>
        </Col>
      </Row>

      {/* Quick Actions */}
      <Card title={<Title level={4} className="!mb-0">Quick Actions</Title>}>
        <Space wrap>
          <Button
            type="primary"
            size="large"
            icon={<UserOutlined />}
            onClick={() => window.location.hash = '#clients'}
            className="h-12"
          >
            Manage Clients
          </Button>
          <Button
            type="primary"
            size="large"
            icon={<LinkOutlined />}
            onClick={() => window.location.hash = '#proxies'}
            className="h-12"
            style={{ background: 'linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%)' }}
          >
            Manage Proxies
          </Button>
          <Button
            type="primary"
            size="large"
            icon={<BarChartOutlined />}
            onClick={() => window.location.hash = '#traffic'}
            className="h-12"
            style={{ background: 'linear-gradient(135deg, #10b981 0%, #059669 100%)' }}
          >
            View Traffic
          </Button>
        </Space>
      </Card>
    </Space>
  );
}
