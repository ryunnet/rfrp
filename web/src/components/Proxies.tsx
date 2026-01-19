import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
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
  const [formData, setFormData] = useState({
    client_id: '',
    name: '',
    type: 'tcp',
    localIP: '127.0.0.1',
    localPort: '',
    remotePort: '',
  });

  const loadProxies = async () => {
    setLoading(true);
    try {
      const data = await proxyApi.list();
      setProxies(data);
    } catch (error) {
      console.error('Failed to load proxies:', error);
      alert(t('proxies.createSuccess'));
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
  }, [t]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await proxyApi.create({
        ...formData,
        localPort: parseInt(formData.localPort),
        remotePort: parseInt(formData.remotePort),
      });
      setShowModal(false);
      setFormData({
        client_id: '',
        name: '',
        type: 'tcp',
        localIP: '127.0.0.1',
        localPort: '',
        remotePort: '',
      });
      loadProxies();
      onRefresh?.();
    } catch (error) {
      console.error('Failed to create proxy:', error);
      alert(t('proxies.createSuccess'));
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm(t('proxies.deleteConfirm'))) return;

    try {
      await proxyApi.delete(id);
      loadProxies();
      onRefresh?.();
    } catch (error) {
      console.error('Failed to delete proxy:', error);
      alert(t('proxies.deleteSuccess'));
    }
  };

  const handleToggle = async (proxy: Proxy) => {
    try {
      await proxyApi.update(proxy.id, { enabled: !proxy.enabled });
      loadProxies();
      onRefresh?.();
    } catch (error) {
      console.error('Failed to toggle proxy:', error);
      alert(t('proxies.updateSuccess'));
    }
  };

  const getClientName = (clientId: string) => {
    const client = clients.find((c) => c.id === clientId);
    return client?.name || 'Unknown';
  };

  if (loading) {
    return <div className="loading">{t('common.loading')}</div>;
  }

  return (
    <div className="section">
      <div className="section-header">
        <h2>{t('proxies.title')}</h2>
        <button className="btn btn-primary" onClick={() => setShowModal(true)}>
          + {t('proxies.createProxy')}
        </button>
      </div>

      {proxies.length === 0 ? (
        <div className="empty-state">{t('proxies.noProxies')}</div>
      ) : (
        <table>
          <thead>
            <tr>
              <th>{t('proxies.proxyName')}</th>
              <th>{t('proxies.client')}</th>
              <th>{t('proxies.proxyType')}</th>
              <th>{t('proxies.targetHost')}:{t('proxies.listenPort')}</th>
              <th>{t('proxies.remotePort')}</th>
              <th>{t('common.status')}</th>
              <th>{t('common.actions')}</th>
            </tr>
          </thead>
          <tbody>
            {proxies.map((proxy) => (
              <tr key={proxy.id}>
                <td>{proxy.name}</td>
                <td>{getClientName(proxy.client_id)}</td>
                <td>{proxy.type.toUpperCase()}</td>
                <td>
                  {proxy.localIP}:{proxy.localPort}
                </td>
                <td>{proxy.remotePort}</td>
                <td>
                  <span
                    className={`badge ${proxy.enabled ? 'badge-success' : 'badge-danger'}`}
                    onClick={() => handleToggle(proxy)}
                    style={{ cursor: 'pointer' }}
                  >
                    {proxy.enabled ? t('proxies.enabled') : t('proxies.disabled')}
                  </span>
                </td>
                <td>
                  <button
                    className="btn btn-danger btn-small"
                    onClick={() => handleDelete(proxy.id)}
                  >
                    {t('common.delete')}
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {showModal && (
        <div className="modal active" onClick={() => setShowModal(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h3>{t('proxies.modalTitle')}</h3>
              <button className="close" onClick={() => setShowModal(false)}>
                ×
              </button>
            </div>
            <form onSubmit={handleCreate}>
              <div className="form-group">
                <label>{t('proxies.client')}</label>
                <select
                  value={formData.client_id}
                  onChange={(e) => setFormData({ ...formData, client_id: e.target.value })}
                  required
                >
                  <option value="">{t('proxies.selectClient')}</option>
                  {clients.map((client) => (
                    <option key={client.id} value={client.id}>
                      {client.name}
                    </option>
                  ))}
                </select>
              </div>
              <div className="form-group">
                <label>{t('proxies.proxyName')}</label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  required
                />
              </div>
              <div className="form-group">
                <label>{t('proxies.proxyType')}</label>
                <select
                  value={formData.type}
                  onChange={(e) => setFormData({ ...formData, type: e.target.value })}
                >
                  <option value="tcp">{t('proxies.tcp')}</option>
                </select>
              </div>
              <div className="form-group">
                <label>{t('proxies.targetHost')}</label>
                <input
                  type="text"
                  value={formData.localIP}
                  onChange={(e) => setFormData({ ...formData, localIP: e.target.value })}
                  required
                />
              </div>
              <div className="form-group">
                <label>{t('proxies.listenPort')}</label>
                <input
                  type="number"
                  value={formData.localPort}
                  onChange={(e) => setFormData({ ...formData, localPort: e.target.value })}
                  required
                />
              </div>
              <div className="form-group">
                <label>{t('proxies.targetPort')}</label>
                <input
                  type="number"
                  value={formData.remotePort}
                  onChange={(e) => setFormData({ ...formData, remotePort: e.target.value })}
                  required
                />
              </div>
              <div className="form-actions">
                <button type="button" className="btn" onClick={() => setShowModal(false)}>
                  {t('common.cancel')}
                </button>
                <button type="submit" className="btn btn-primary">
                  {t('common.create')}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
