import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { trafficApi, formatBytes, type TrafficOverview } from '../api';
import { useAuth } from '../contexts/AuthContext';

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
      // 根据用户加载流量统计
      // 管理员可以看到全局流量，普通用户只看到自己的流量
      const data = user?.is_admin
        ? await trafficApi.getOverview(days)
        : await trafficApi.getUserTraffic(user!.id, days);
      setTraffic(data);
    } catch (error) {
      console.error('Failed to load traffic:', error);
      alert('Failed to load traffic statistics');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadTraffic();
    // 定期刷新统计数据
    const interval = setInterval(loadTraffic, 30000); // 30秒刷新一次
    return () => clearInterval(interval);
  }, [days, user?.id]);

  if (loading) {
    return <div className="loading">{t('common.loading')}</div>;
  }

  if (!traffic) {
    return <div className="empty-state">No traffic data available</div>;
  }

  return (
    <div className="section">
      <div className="section-header">
        <h2>
          {user?.is_admin ? t('traffic.title') : `${t('traffic.title')} - ${user?.username}`}
        </h2>
        <select
          value={days}
          onChange={(e) => setDays(Number(e.target.value))}
          style={{
            padding: '0.5rem',
            borderRadius: '4px',
            border: '1px solid #ddd',
          }}
        >
          <option value={7}>Last 7 days</option>
          <option value={30}>Last 30 days</option>
          <option value={90}>Last 90 days</option>
        </select>
      </div>

      {/* Total Traffic Card */}
      <div
        style={{
          background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
          color: 'white',
          padding: '1.5rem',
          borderRadius: '8px',
          marginBottom: '1.5rem',
        }}
      >
        <h3 style={{ margin: '0 0 1rem 0', fontSize: '1.2rem' }}>
          {user?.is_admin ? 'Total Traffic' : 'My Traffic'}
        </h3>
        <div style={{ display: 'flex', gap: '2rem', flexWrap: 'wrap' }}>
          <div>
            <div style={{ fontSize: '0.875rem', opacity: 0.9 }}>Sent</div>
            <div style={{ fontSize: '1.5rem', fontWeight: 'bold' }}>
              {formatBytes(traffic.total_traffic.total_bytes_sent)}
            </div>
          </div>
          <div>
            <div style={{ fontSize: '0.875rem', opacity: 0.9 }}>Received</div>
            <div style={{ fontSize: '1.5rem', fontWeight: 'bold' }}>
              {formatBytes(traffic.total_traffic.total_bytes_received)}
            </div>
          </div>
          <div>
            <div style={{ fontSize: '0.875rem', opacity: 0.9 }}>Total</div>
            <div style={{ fontSize: '1.5rem', fontWeight: 'bold' }}>
              {formatBytes(traffic.total_traffic.total_bytes)}
            </div>
          </div>
        </div>
      </div>

      {/* Traffic by Proxy */}
      <div style={{ marginBottom: '1.5rem' }}>
        <h3 style={{ marginBottom: '1rem' }}>Traffic by Proxy</h3>
        {traffic.by_proxy.length === 0 ? (
          <div className="empty-state">No proxy traffic data</div>
        ) : (
          <table>
            <thead>
              <tr>
                <th>Proxy Name</th>
                <th>Client</th>
                <th>Sent</th>
                <th>Received</th>
                <th>Total</th>
              </tr>
            </thead>
            <tbody>
              {traffic.by_proxy.map((proxy) => (
                <tr key={proxy.proxy_id}>
                  <td>{proxy.proxy_name}</td>
                  <td>{proxy.client_name}</td>
                  <td>{formatBytes(proxy.total_bytes_sent)}</td>
                  <td>{formatBytes(proxy.total_bytes_received)}</td>
                  <td>
                    <strong>{formatBytes(proxy.total_bytes)}</strong>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Traffic by Client */}
      <div style={{ marginBottom: '1.5rem' }}>
        <h3 style={{ marginBottom: '1rem' }}>Traffic by Client</h3>
        {traffic.by_client.length === 0 ? (
          <div className="empty-state">No client traffic data</div>
        ) : (
          <table>
            <thead>
              <tr>
                <th>Client Name</th>
                <th>Sent</th>
                <th>Received</th>
                <th>Total</th>
              </tr>
            </thead>
            <tbody>
              {traffic.by_client.map((client) => (
                <tr key={client.client_id}>
                  <td>{client.client_name}</td>
                  <td>{formatBytes(client.total_bytes_sent)}</td>
                  <td>{formatBytes(client.total_bytes_received)}</td>
                  <td>
                    <strong>{formatBytes(client.total_bytes)}</strong>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Traffic by User - Only show for admin */}
      {user?.is_admin && traffic.by_user.length > 0 && (
        <div style={{ marginBottom: '1.5rem' }}>
          <h3 style={{ marginBottom: '1rem' }}>Traffic by User</h3>
          <table>
            <thead>
              <tr>
                <th>Username</th>
                <th>Sent</th>
                <th>Received</th>
                <th>Total</th>
              </tr>
            </thead>
            <tbody>
              {traffic.by_user.map((user) => (
                <tr key={user.user_id}>
                  <td>{user.username}</td>
                  <td>{formatBytes(user.total_bytes_sent)}</td>
                  <td>{formatBytes(user.total_bytes_received)}</td>
                  <td>
                    <strong>{formatBytes(user.total_bytes)}</strong>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Daily Traffic */}
      <div>
        <h3 style={{ marginBottom: '1rem' }}>Daily Traffic (Last {days} Days)</h3>
        {traffic.daily_traffic.length === 0 ? (
          <div className="empty-state">No daily traffic data</div>
        ) : (
          <table>
            <thead>
              <tr>
                <th>Date</th>
                <th>Sent</th>
                <th>Received</th>
                <th>Total</th>
              </tr>
            </thead>
            <tbody>
              {traffic.daily_traffic.map((day, index) => (
                <tr key={`${day.date}-${index}`}>
                  <td>{day.date}</td>
                  <td>{formatBytes(day.total_bytes_sent)}</td>
                  <td>{formatBytes(day.total_bytes_received)}</td>
                  <td>
                    <strong>{formatBytes(day.total_bytes)}</strong>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
