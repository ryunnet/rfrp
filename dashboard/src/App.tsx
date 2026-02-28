import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider } from './contexts/AuthContext';
import { ToastProvider } from './contexts/ToastContext';
import ProtectedRoute from './components/ProtectedRoute';
import Layout from './components/Layout';
import Login from './pages/Login';
import Register from './pages/Register';
import Dashboard from './pages/Dashboard';
import Clients from './pages/Clients';
import Proxies from './pages/Proxies';
import Users from './pages/Users';
import Traffic from './pages/Traffic';
import Settings from './pages/Settings';
import Nodes from './pages/Nodes';
import Subscriptions from './pages/Subscriptions';
import UserSubscriptions from './pages/UserSubscriptions';
import MySubscription from './pages/MySubscription';

function App() {
  return (
    <BrowserRouter>
      <ToastProvider>
        <AuthProvider>
          <Routes>
            <Route path="/login" element={<Login />} />
            <Route path="/register" element={<Register />} />
            <Route
              path="/*"
              element={
                <ProtectedRoute>
                  <Layout>
                    <Routes>
                      <Route path="/" element={<Dashboard />} />
                      <Route path="/clients" element={<Clients />} />
                      <Route path="/proxies" element={<Proxies />} />
                      <Route path="/traffic" element={<Traffic />} />
                      <Route
                        path="/users"
                        element={
                          <ProtectedRoute requireAdmin>
                            <Users />
                          </ProtectedRoute>
                        }
                      />
                      <Route
                        path="/settings"
                        element={
                          <ProtectedRoute requireAdmin>
                            <Settings />
                          </ProtectedRoute>
                        }
                      />
                      <Route path="/nodes" element={<Nodes />} />
                      <Route path="/my-subscription" element={<MySubscription />} />
                      <Route
                        path="/subscriptions"
                        element={
                          <ProtectedRoute requireAdmin>
                            <Subscriptions />
                          </ProtectedRoute>
                        }
                      />
                      <Route
                        path="/user-subscriptions"
                        element={
                          <ProtectedRoute requireAdmin>
                            <UserSubscriptions />
                          </ProtectedRoute>
                        }
                      />
                      <Route path="*" element={<Navigate to="/" replace />} />
                    </Routes>
                  </Layout>
                </ProtectedRoute>
              }
            />
          </Routes>
        </AuthProvider>
      </ToastProvider>
    </BrowserRouter>
  );
}

export default App;
