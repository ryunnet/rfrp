import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { dashboardApi, formatBytes, type DashboardStats } from '../api';
import { useAuth } from '../contexts/AuthContext';

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
      // alert('Failed to load dashboard statistics');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadStats();
    // 定期刷新统计数据
    const interval = setInterval(loadStats, 30000); // 30秒刷新一次
    return () => clearInterval(interval);
  }, [user?.id]);

  if (loading) {
    return (
      <div className="loading">
        {t('common.loading')}
      </div>
    );
  }

  if (!stats || !user) {
    return (
      <div className="empty-state">
        No dashboard data available
      </div>
    );
  }

  return (
    <div className="section">
      <div className="section-header">
        <h2>{t('nav.dashboard')}</h2>
      </div>

      {/* 欢迎卡片 */}
      <div
        style={{
          background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
          color: 'white',
          padding: '2rem',
          borderRadius: '8px',
          marginBottom: '1.5rem',
        }}
      >
        <h2 style={{ margin: '0 0 0.5rem 0', fontSize: '1.8rem' }}>
          Welcome back, {user.username}!
        </h2>
        <p style={{ margin: 0, fontSize: '1rem', opacity: 0.9 }}>
          {user.is_admin ? 'Administrator' : 'User'}
        </p>
      </div>

      {/* 统计卡片网格 */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
          gap: '1.5rem',
          marginBottom: '1.5rem',
        }}
      >
        {/* 总客户端数 */}
        <div
          style={{
            background: 'white',
            border: '1px solid #e5e7eb',
            borderRadius: '8px',
            padding: '1.5rem',
            boxShadow: '0 1px 3px rgba(0, 0, 0, 0.1)',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', marginBottom: '1rem' }}>
            <div
              style={{
                width: '48px',
                height: '48px',
                borderRadius: '8px',
                background: 'linear-gradient(135deg, #3b82f6 0%, #2563eb 100%)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                marginRight: '1rem',
                fontSize: '1.5rem',
              }}
            >
              👥
            </div>
            <div>
              <div style={{ fontSize: '0.875rem', color: '#6b7280', marginBottom: '0.25rem' }}>
                Total Clients
              </div>
              <div style={{ fontSize: '2rem', fontWeight: 'bold', color: '#111827' }}>
                {stats.total_clients}
              </div>
            </div>
          </div>
          <div style={{ fontSize: '0.875rem', color: '#6b7280' }}>
            <span style={{ color: '#10b981', fontWeight: 'bold' }}>
              {stats.online_clients}
            </span>{' '}
            online
          </div>
        </div>

        {/* 总代理数 */}
        <div
          style={{
            background: 'white',
            border: '1px solid #e5e7eb',
            borderRadius: '8px',
            padding: '1.5rem',
            boxShadow: '0 1px 3px rgba(0, 0, 0, 0.1)',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', marginBottom: '1rem' }}>
            <div
              style={{
                width: '48px',
                height: '48px',
                borderRadius: '8px',
                background: 'linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                marginRight: '1rem',
                fontSize: '1.5rem',
              }}
            >
              🔗
            </div>
            <div>
              <div style={{ fontSize: '0.875rem', color: '#6b7280', marginBottom: '0.25rem' }}>
                Total Proxies
              </div>
              <div style={{ fontSize: '2rem', fontWeight: 'bold', color: '#111827' }}>
                {stats.total_proxies}
              </div>
            </div>
          </div>
          <div style={{ fontSize: '0.875rem', color: '#6b7280' }}>
            <span style={{ color: '#10b981', fontWeight: 'bold' }}>
              {stats.enabled_proxies}
            </span>{' '}
            enabled
          </div>
        </div>

        {/* 用户流量 */}
        <div
          style={{
            background: 'white',
            border: '1px solid #e5e7eb',
            borderRadius: '8px',
            padding: '1.5rem',
            boxShadow: '0 1px 3px rgba(0, 0, 0, 0.1)',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', marginBottom: '1rem' }}>
            <div
              style={{
                width: '48px',
                height: '48px',
                borderRadius: '8px',
                background: 'linear-gradient(135deg, #10b981 0%, #059669 100%)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                marginRight: '1rem',
                fontSize: '1.5rem',
              }}
            >
              📊
            </div>
            <div>
              <div style={{ fontSize: '0.875rem', color: '#6b7280', marginBottom: '0.25rem' }}>
                My Traffic
              </div>
              <div style={{ fontSize: '1.5rem', fontWeight: 'bold', color: '#111827' }}>
                {formatBytes(stats.user_traffic.total_bytes)}
              </div>
            </div>
          </div>
          <div style={{ fontSize: '0.875rem', color: '#6b7280' }}>
            <span style={{ color: '#3b82f6', fontWeight: 'bold' }}>
              {formatBytes(stats.user_traffic.total_bytes_sent)}
            </span>{' '}
            sent /{' '}
            <span style={{ color: '#8b5cf6', fontWeight: 'bold' }}>
              {formatBytes(stats.user_traffic.total_bytes_received)}
            </span>{' '}
            received
          </div>
        </div>
      </div>

      {/* 快捷操作 */}
      <div
        style={{
          background: 'white',
          border: '1px solid #e5e7eb',
          borderRadius: '8px',
          padding: '1.5rem',
          boxShadow: '0 1px 3px rgba(0, 0, 0, 0.1)',
        }}
      >
        <h3 style={{ margin: '0 0 1rem 0', fontSize: '1.2rem' }}>Quick Actions</h3>
        <div style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap' }}>
          <button
            onClick={() => window.location.hash = '#clients'}
            style={{
              padding: '0.75rem 1.5rem',
              background: 'linear-gradient(135deg, #3b82f6 0%, #2563eb 100%)',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer',
              fontWeight: 'bold',
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem',
            }}
          >
            <span>👥</span>
            <span>Manage Clients</span>
          </button>
          <button
            onClick={() => window.location.hash = '#proxies'}
            style={{
              padding: '0.75rem 1.5rem',
              background: 'linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%)',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer',
              fontWeight: 'bold',
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem',
            }}
          >
            <span>🔗</span>
            <span>Manage Proxies</span>
          </button>
          <button
            onClick={() => window.location.hash = '#traffic'}
            style={{
              padding: '0.75rem 1.5rem',
              background: 'linear-gradient(135deg, #10b981 0%, #059669 100%)',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: 'pointer',
              fontWeight: 'bold',
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem',
            }}
          >
            <span>📊</span>
            <span>View Traffic</span>
          </button>
        </div>
      </div>
    </div>
  );
}
