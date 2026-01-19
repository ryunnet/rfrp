import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Clients, Proxies, Traffic, Dashboard, LanguageSwitcher } from './components'
import { AuthProvider, useAuth } from './contexts/AuthContext'
import { Login } from './pages/Login'
import { Users } from './pages/Users'
import './App.css'

type TabType = 'dashboard' | 'clients' | 'proxies' | 'traffic' | 'users'

function AppContent() {
  const { t } = useTranslation()
  const { user, loading, logout, isAuthenticated, isAdmin } = useAuth()
  const [activeTab, setActiveTab] = useState<TabType>('dashboard')
  const [refreshKey, setRefreshKey] = useState(0)

  const handleRefresh = () => {
    setRefreshKey(prev => prev + 1)
  }

  if (loading) {
    return (
      <div style={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        height: '100vh',
        background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
        color: 'white',
        fontSize: '1.5rem',
      }}>
        {t('common.loading')}
      </div>
    )
  }

  if (!isAuthenticated) {
    return <Login />
  }

  return (
    <div className="app-container">
      <aside className="sidebar">
        <div className="sidebar-header">
          <h1>RFRP</h1>
          <p>{t('nav.dashboard')}</p>
          <div style={{
            marginTop: '0.5rem',
            padding: '0.5rem',
            background: 'rgba(255, 255, 255, 0.1)',
            borderRadius: '4px',
            fontSize: '0.875rem',
          }}>
            <div>{t('auth.username')}: {user?.username}</div>
            <div style={{
              fontSize: '0.75rem',
              color: 'rgba(255, 255, 255, 0.7)',
            }}>
              {isAdmin ? t('users.admin') : t('users.user')}
            </div>
          </div>
        </div>

        <nav className="sidebar-nav">
          <button
            className={`nav-item ${activeTab === 'dashboard' ? 'active' : ''}`}
            onClick={() => setActiveTab('dashboard')}
          >
            <span className="nav-icon">🏠</span>
            <span className="nav-text">{t('nav.dashboard')}</span>
          </button>
          <button
            className={`nav-item ${activeTab === 'clients' ? 'active' : ''}`}
            onClick={() => setActiveTab('clients')}
          >
            <span className="nav-icon">👥</span>
            <span className="nav-text">{t('nav.clients')}</span>
          </button>
          <button
            className={`nav-item ${activeTab === 'proxies' ? 'active' : ''}`}
            onClick={() => setActiveTab('proxies')}
          >
            <span className="nav-icon">🔗</span>
            <span className="nav-text">{t('nav.proxies')}</span>
          </button>
          <button
            className={`nav-item ${activeTab === 'traffic' ? 'active' : ''}`}
            onClick={() => setActiveTab('traffic')}
          >
            <span className="nav-icon">📊</span>
            <span className="nav-text">Traffic</span>
          </button>
          {isAdmin && (
            <button
              className={`nav-item ${activeTab === 'users' ? 'active' : ''}`}
              onClick={() => setActiveTab('users')}
            >
              <span className="nav-icon">👤</span>
              <span className="nav-text">{t('nav.users')}</span>
            </button>
          )}
        </nav>

        <div className="sidebar-footer">
          <LanguageSwitcher />
          <button
            onClick={logout}
            style={{
              width: '100%',
              padding: '0.5rem',
              background: 'rgba(255, 255, 255, 0.1)',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              marginTop: '0.5rem',
              marginBottom: '0.5rem',
            }}
          >
            {t('auth.logout')}
          </button>
          <p className="version">v0.1.0</p>
        </div>
      </aside>

      <main className="main-content">
        <div className="content-header">
          <h2>{t(`${activeTab}.title`)}</h2>
        </div>

        <div className="content-body">
          {activeTab === 'dashboard' && (
            <Dashboard key={`dashboard-${refreshKey}`} onRefresh={handleRefresh} />
          )}
          {activeTab === 'clients' && (
            <Clients key={`clients-${refreshKey}`} onRefresh={handleRefresh} />
          )}
          {activeTab === 'proxies' && (
            <Proxies key={`proxies-${refreshKey}`} onRefresh={handleRefresh} />
          )}
          {activeTab === 'traffic' && (
            <Traffic key={`traffic-${refreshKey}`} onRefresh={handleRefresh} />
          )}
          {activeTab === 'users' && isAdmin && (
            <Users key={`users-${refreshKey}`} />
          )}
        </div>
      </main>
    </div>
  )
}

function App() {
  return (
    <AuthProvider>
      <AppContent />
    </AuthProvider>
  )
}

export default App
