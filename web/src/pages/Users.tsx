import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
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
  const [newUsername, setNewUsername] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [newIsAdmin, setNewIsAdmin] = useState(false);
  const [_generatedPassword, setGeneratedPassword] = useState('');
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
      setError(err instanceof Error ? err.message : t('users.createSuccess'));
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
      setError(err instanceof Error ? err.message : t('users.assignFailed'));
    }
  };

  const handleCreateUser = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const result = await userApi.create({
        username: newUsername,
        password: newPassword || undefined,
        is_admin: newIsAdmin,
      });
      if (result.generated_password) {
        setGeneratedPassword(result.generated_password);
        alert(`${t('users.userCreated')}\n\n${t('auth.username')}: ${result.username}\n${t('users.generatedPassword')}: ${result.generated_password}\n\n${t('users.savePasswordWarning')}`);
      }
      setNewUsername('');
      setNewPassword('');
      setNewIsAdmin(false);
      setShowCreateModal(false);
      fetchUsers();
    } catch (err) {
      setError(err instanceof Error ? err.message : t('users.createSuccess'));
    }
  };

  const handleDeleteUser = async (userId: number) => {
    if (!confirm(t('users.deleteConfirm'))) return;
    try {
      await userApi.delete(userId);
      fetchUsers();
    } catch (err) {
      setError(err instanceof Error ? err.message : t('users.deleteSuccess'));
    }
  };

  const handleToggleAdmin = async (user: User) => {
    try {
      await userApi.update(user.id, { is_admin: !user.is_admin });
      fetchUsers();
    } catch (err) {
      setError(err instanceof Error ? err.message : t('users.updateSuccess'));
    }
  };

  const handleManageClients = async (user: User) => {
    await fetchUserClients(user);
    try {
      const clients = await clientApi.list();
      setAllClients(clients);
      setShowClientModal(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : t('clients.noClients'));
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
      setError(err instanceof Error ? err.message : t('users.assignFailed'));
    }
  };

  if (!isAdmin) {
    return (
      <div style={{ padding: '2rem', textAlign: 'center' }}>
        <h2>{t('auth.accessDenied')}</h2>
        <p>{t('auth.noPermission')}</p>
      </div>
    );
  }

  if (loading) {
    return <div style={{ padding: '2rem', textAlign: 'center' }}>{t('common.loading')}</div>;
  }

  return (
    <div style={{ padding: '2rem' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '2rem' }}>
        <h2>{t('users.title')}</h2>
        <button
          onClick={() => setShowCreateModal(true)}
          style={{
            padding: '0.5rem 1rem',
            background: '#667eea',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          {t('users.createUser')}
        </button>
      </div>

      {error && (
        <div style={{
          padding: '1rem',
          marginBottom: '1rem',
          background: '#fee',
          color: '#c33',
          borderRadius: '4px',
        }}>
          {error}
        </div>
      )}

      <div style={{
        background: 'white',
        borderRadius: '8px',
        overflow: 'hidden',
        boxShadow: '0 2px 4px rgba(0, 0, 0, 0.1)',
      }}>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead style={{ background: '#f5f5f5' }}>
            <tr>
              <th style={{ padding: '1rem', textAlign: 'left', borderBottom: '1px solid #ddd' }}>{t('users.id')}</th>
              <th style={{ padding: '1rem', textAlign: 'left', borderBottom: '1px solid #ddd' }}>{t('users.username')}</th>
              <th style={{ padding: '1rem', textAlign: 'left', borderBottom: '1px solid #ddd' }}>{t('users.role')}</th>
              <th style={{ padding: '1rem', textAlign: 'left', borderBottom: '1px solid #ddd' }}>{t('users.clients')}</th>
              <th style={{ padding: '1rem', textAlign: 'left', borderBottom: '1px solid #ddd' }}>{t('users.created')}</th>
              <th style={{ padding: '1rem', textAlign: 'left', borderBottom: '1px solid #ddd' }}>{t('common.actions')}</th>
            </tr>
          </thead>
          <tbody>
            {users.map((user) => (
              <tr key={user.id} style={{ borderBottom: '1px solid #eee' }}>
                <td style={{ padding: '1rem' }}>{user.id}</td>
                <td style={{ padding: '1rem' }}>{user.username}</td>
                <td style={{ padding: '1rem' }}>
                  <span style={{
                    padding: '0.25rem 0.5rem',
                    borderRadius: '4px',
                    fontSize: '0.75rem',
                    background: user.is_admin ? '#e3f2fd' : '#f5f5f5',
                    color: user.is_admin ? '#1976d2' : '#666',
                  }}>
                    {user.is_admin ? t('users.admin') : t('users.user')}
                  </span>
                </td>
                <td style={{ padding: '1rem' }}>{user.client_count || 0}</td>
                <td style={{ padding: '1rem' }}>{new Date(user.created_at).toLocaleDateString()}</td>
                <td style={{ padding: '1rem' }}>
                  <button
                    onClick={() => handleManageClients(user)}
                    style={{
                      padding: '0.25rem 0.5rem',
                      marginRight: '0.5rem',
                      background: '#4caf50',
                      color: 'white',
                      border: 'none',
                      borderRadius: '4px',
                      cursor: 'pointer',
                      fontSize: '0.875rem',
                    }}
                  >
                    {t('users.manageClients')}
                  </button>
                  <button
                    onClick={() => handleToggleAdmin(user)}
                    disabled={user.username === 'admin'}
                    style={{
                      padding: '0.25rem 0.5rem',
                      marginRight: '0.5rem',
                      background: user.is_admin ? '#ff9800' : '#2196f3',
                      color: 'white',
                      border: 'none',
                      borderRadius: '4px',
                      cursor: user.username === 'admin' ? 'not-allowed' : 'pointer',
                      fontSize: '0.875rem',
                    }}
                  >
                    {user.is_admin ? t('users.makeUser') : t('users.makeAdmin')}
                  </button>
                  <button
                    onClick={() => handleDeleteUser(user.id)}
                    disabled={user.username === 'admin'}
                    style={{
                      padding: '0.25rem 0.5rem',
                      background: '#f44336',
                      color: 'white',
                      border: 'none',
                      borderRadius: '4px',
                      cursor: user.username === 'admin' ? 'not-allowed' : 'pointer',
                      fontSize: '0.875rem',
                    }}
                  >
                    {t('common.delete')}
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Create User Modal */}
      {showCreateModal && (
        <div style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: 'rgba(0, 0, 0, 0.5)',
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          zIndex: 1000,
        }}>
          <div style={{
            background: 'white',
            padding: '2rem',
            borderRadius: '8px',
            width: '100%',
            maxWidth: '400px',
          }}>
            <h3 style={{ marginBottom: '1rem' }}>{t('users.modalTitle')}</h3>
            <form onSubmit={handleCreateUser}>
              <div style={{ marginBottom: '1rem' }}>
                <label style={{ display: 'block', marginBottom: '0.5rem' }}>{t('users.username')}</label>
                <input
                  type="text"
                  value={newUsername}
                  onChange={(e) => setNewUsername(e.target.value)}
                  required
                  placeholder={t('users.usernamePlaceholder')}
                  style={{
                    width: '100%',
                    padding: '0.5rem',
                    border: '1px solid #ddd',
                    borderRadius: '4px',
                    boxSizing: 'border-box',
                  }}
                />
              </div>
              <div style={{ marginBottom: '1rem' }}>
                <label style={{ display: 'block', marginBottom: '0.5rem' }}>{t('auth.password')}</label>
                <input
                  type="password"
                  value={newPassword}
                  onChange={(e) => setNewPassword(e.target.value)}
                  placeholder={t('users.passwordPlaceholder')}
                  style={{
                    width: '100%',
                    padding: '0.5rem',
                    border: '1px solid #ddd',
                    borderRadius: '4px',
                    boxSizing: 'border-box',
                  }}
                />
              </div>
              <div style={{ marginBottom: '1rem' }}>
                <label style={{ display: 'flex', alignItems: 'center' }}>
                  <input
                    type="checkbox"
                    checked={newIsAdmin}
                    onChange={(e) => setNewIsAdmin(e.target.checked)}
                    style={{ marginRight: '0.5rem' }}
                  />
                  {t('users.isAdmin')}
                </label>
              </div>
              <div style={{ display: 'flex', gap: '0.5rem' }}>
                <button
                  type="submit"
                  style={{
                    flex: 1,
                    padding: '0.5rem',
                    background: '#667eea',
                    color: 'white',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer',
                  }}
                >
                  {t('common.create')}
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setShowCreateModal(false);
                    setNewUsername('');
                    setNewPassword('');
                    setNewIsAdmin(false);
                  }}
                  style={{
                    flex: 1,
                    padding: '0.5rem',
                    background: '#999',
                    color: 'white',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer',
                  }}
                >
                  {t('common.cancel')}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Manage Clients Modal */}
      {showClientModal && selectedUser && (
        <div style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: 'rgba(0, 0, 0, 0.5)',
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          zIndex: 1000,
        }}>
          <div style={{
            background: 'white',
            padding: '2rem',
            borderRadius: '8px',
            width: '100%',
            maxWidth: '500px',
            maxHeight: '80vh',
            overflow: 'auto',
          }}>
            <h3 style={{ marginBottom: '1rem' }}>{t('users.manageClientsTitle', { username: selectedUser.username })}</h3>
            <div style={{ marginBottom: '1rem' }}>
              {allClients.map((client) => (
                <label key={client.id} style={{
                  display: 'flex',
                  alignItems: 'center',
                  padding: '0.5rem',
                  borderBottom: '1px solid #eee',
                  cursor: 'pointer',
                }}>
                  <input
                    type="checkbox"
                    checked={assignedClients.includes(client.id)}
                    onChange={() => handleToggleClient(client.id)}
                    style={{ marginRight: '0.5rem' }}
                  />
                  <div style={{ flex: 1 }}>
                    <div style={{ fontWeight: '500' }}>{client.name}</div>
                    <div style={{ fontSize: '0.875rem', color: '#666' }}>{client.id}</div>
                  </div>
                  <span style={{
                    padding: '0.25rem 0.5rem',
                    borderRadius: '4px',
                    fontSize: '0.75rem',
                    background: client.is_online ? '#e8f5e9' : '#ffebee',
                    color: client.is_online ? '#2e7d32' : '#c62828',
                  }}>
                    {client.is_online ? t('common.online') : t('common.offline')}
                  </span>
                </label>
              ))}
            </div>
            <button
              onClick={() => {
                setShowClientModal(false);
                setSelectedUser(null);
                fetchUsers();
              }}
              style={{
                width: '100%',
                padding: '0.5rem',
                background: '#667eea',
                color: 'white',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              {t('common.close')}
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
