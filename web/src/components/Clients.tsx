import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { clientApi, type Client } from '../api';

interface ClientsProps {
  onRefresh?: () => void;
}

export function Clients({ onRefresh }: ClientsProps) {
  const { t } = useTranslation();
  const [clients, setClients] = useState<Client[]>([]);
  const [loading, setLoading] = useState(true);
  const [showModal, setShowModal] = useState(false);
  const [newClientName, setNewClientName] = useState('');
  const [newClientToken, setNewClientToken] = useState('');

  const loadClients = async () => {
    setLoading(true);
    try {
      const data = await clientApi.list();
      setClients(data);
    } catch (error) {
      console.error('Failed to load clients:', error);
      alert(t('clients.createSuccess'));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadClients();
    // 定期刷新在线状态
    const interval = setInterval(loadClients, 5000);
    return () => clearInterval(interval);
  }, [t]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await clientApi.create({
        name: newClientName,
        token: newClientToken || undefined,
      });
      setShowModal(false);
      setNewClientName('');
      setNewClientToken('');
      loadClients();
      onRefresh?.();
    } catch (error) {
      console.error('Failed to create client:', error);
      alert(t('clients.createSuccess'));
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm(t('clients.deleteConfirm'))) return;

    try {
      await clientApi.delete(id);
      loadClients();
      onRefresh?.();
    } catch (error) {
      console.error('Failed to delete client:', error);
      alert(t('clients.deleteSuccess'));
    }
  };

  if (loading) {
    return <div className="loading">{t('common.loading')}</div>;
  }

  return (
    <>
      <div className="section">
        <div className="section-header">
          <h2>{t('clients.title')}</h2>
          <button className="btn btn-primary" onClick={() => setShowModal(true)}>
            + {t('clients.createClient')}
          </button>
        </div>

        {clients.length === 0 ? (
          <div className="empty-state">{t('clients.noClients')}</div>
        ) : (
          <table>
            <thead>
              <tr>
                <th>{t('clients.clientName')}</th>
                <th>Token</th>
                <th>{t('common.status')}</th>
                <th>{t('clients.created_at')}</th>
                <th>{t('common.actions')}</th>
              </tr>
            </thead>
            <tbody>
              {clients.map((client) => (
                <tr key={client.id}>
                  <td>{client.name}</td>
                  <td>
                    <code className="token">{client.token}</code>
                  </td>
                  <td>
                    <span className={`status ${client.is_online ? 'online' : 'offline'}`}>
                      <span className={`online-dot ${client.is_online ? 'online' : 'offline'}`}></span>
                      {client.is_online ? t('common.online') : t('common.offline')}
                    </span>
                  </td>
                  <td>{new Date(client.created_at).toLocaleString()}</td>
                  <td>
                    <button
                      className="btn btn-danger btn-small"
                      onClick={() => handleDelete(client.id)}
                    >
                      {t('common.delete')}
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {showModal && (
        <div className="modal active" onClick={() => setShowModal(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h3>{t('clients.modalTitle')}</h3>
              <button className="close" onClick={() => setShowModal(false)}>
                ×
              </button>
            </div>
            <form onSubmit={handleCreate}>
              <div className="form-group">
                <label>{t('clients.clientName')}</label>
                <input
                  type="text"
                  value={newClientName}
                  onChange={(e) => setNewClientName(e.target.value)}
                  required
                  placeholder={t('clients.namePlaceholder')}
                />
              </div>
              <div className="form-group">
                <label>Token ({t('clients.namePlaceholder')})</label>
                <input
                  type="text"
                  value={newClientToken}
                  onChange={(e) => setNewClientToken(e.target.value)}
                  placeholder={t('clients.namePlaceholder')}
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
    </>
  );
}
