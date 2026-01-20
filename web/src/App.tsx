import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Layout, Menu, Button, Avatar, Typography, Spin, ConfigProvider } from 'antd'
import {
  DashboardOutlined,
  UserOutlined,
  LinkOutlined,
  BarChartOutlined,
  TeamOutlined,
  LogoutOutlined} from '@ant-design/icons'
import { Clients, Proxies, Traffic, Dashboard, LanguageSwitcher } from './components'
import { AuthProvider, useAuth } from './contexts/AuthContext'
import { Login } from './pages/Login'
import { Users } from './pages/Users'

const { Header, Content, Sider } = Layout
const { Title } = Typography

type TabType = 'dashboard' | 'clients' | 'proxies' | 'traffic' | 'users'

function AppContent() {
  const { t } = useTranslation()
  const { user, loading, logout, isAuthenticated, isAdmin } = useAuth()
  const [activeTab, setActiveTab] = useState<TabType>('dashboard')
  const [refreshKey, setRefreshKey] = useState(0)
  const [collapsed, setCollapsed] = useState(false)

  const handleRefresh = () => {
    setRefreshKey(prev => prev + 1)
  }

  if (loading) {
    return (
      <div className="h-screen flex items-center justify-center bg-gradient-to-br from-primary-500 to-purple-600">
        <Spin size="large" tip="Loading..." />
      </div>
    )
  }

  if (!isAuthenticated) {
    return <Login />
  }

  const menuItems = [
    {
      key: 'dashboard',
      icon: <DashboardOutlined />,
      label: t('nav.dashboard'),
    },
    {
      key: 'clients',
      icon: <UserOutlined />,
      label: t('nav.clients'),
    },
    {
      key: 'proxies',
      icon: <LinkOutlined />,
      label: t('nav.proxies'),
    },
    {
      key: 'traffic',
      icon: <BarChartOutlined />,
      label: 'Traffic',
    },
    ...(isAdmin ? [{
      key: 'users',
      icon: <TeamOutlined />,
      label: t('nav.users'),
    }] : []),
  ]

  return (
    <ConfigProvider
      theme={{
        token: {
          colorPrimary: '#667eea',
          borderRadius: 8,
        },
      }}
    >
      <Layout style={{ minHeight: '100vh', maxHeight: '100vh', overflow: 'hidden' }}>
        <Sider
          collapsible
          collapsed={collapsed}
          onCollapse={setCollapsed}
          style={{
            background: 'linear-gradient(180deg, #667eea 0%, #764ba2 100%)',
            height: '100vh',
            display: 'flex',
            flexDirection: 'column',
          }}
        >
          <div className="p-6 text-white flex-shrink-0">
            <Title level={3} className="!mb-1 !text-white">
              {collapsed ? 'R' : 'RFRP'}
            </Title>
            {!collapsed && (
              <p className="text-sm opacity-80">{t('nav.dashboard')}</p>
            )}
            {!collapsed && user && (
              <div className="mt-4 p-3 bg-white/10 rounded-lg">
                <div className="flex items-center gap-2">
                  <Avatar icon={<UserOutlined />} size="small" />
                  <span className="text-sm font-medium">{user.username}</span>
                </div>
                <div className="text-xs opacity-70 mt-1">
                  {isAdmin ? t('users.admin') : t('users.user')}
                </div>
              </div>
            )}
          </div>

          <div className="flex-1 overflow-y-auto overflow-x-hidden">
            <Menu
              theme="dark"
              mode="inline"
              selectedKeys={[activeTab]}
              items={menuItems}
              onClick={({ key }) => setActiveTab(key as TabType)}
              className="border-0 bg-transparent"
            />
          </div>

          <div className="p-4 text-white flex-shrink-0">
            {!collapsed && (
              <>
                <div className="flex gap-2 mb-3">
                  <LanguageSwitcher />
                </div>
                <Button
                  icon={<LogoutOutlined />}
                  onClick={logout}
                  block
                  className="bg-white/10 border-0 text-white hover:bg-white/20"
                >
                  {t('auth.logout')}
                </Button>
                <p className="text-xs text-center opacity-60 mt-3">v0.1.0</p>
              </>
            )}
            {collapsed && (
              <Button
                icon={<LogoutOutlined />}
                onClick={logout}
                block
                className="bg-white/10 border-0 text-white hover:bg-white/20"
              />
            )}
          </div>
        </Sider>

        <Layout style={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
          <Header className="bg-white shadow-sm px-8 flex items-center flex-shrink-0">
            <Title level={3} className="!mb-0 text-gray-800">
              {t(`${activeTab}.title`)}
            </Title>
          </Header>

          <Content className="p-8" style={{ flex: 1, overflowY: 'auto', overflowX: 'hidden' }}>
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
          </Content>
        </Layout>
      </Layout>
    </ConfigProvider>
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
