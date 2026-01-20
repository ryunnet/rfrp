import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Form, Input, Button, Alert, Card, ConfigProvider, Space } from 'antd';
import { UserOutlined, LockOutlined } from '@ant-design/icons';
import { useAuth } from '../contexts/AuthContext';
import { LanguageSwitcher } from '../components/LanguageSwitcher';

export const Login = () => {
  const { t } = useTranslation();
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const { login } = useAuth();
  const [form] = Form.useForm();

  const handleSubmit = async (values: { username: string; password: string }) => {
    setError('');
    setLoading(true);

    try {
      await login(values.username, values.password);
    } catch (err) {
      setError(err instanceof Error ? err.message : t('auth.loginFailed'));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-primary-500 to-purple-600 relative p-4">
      <div className="absolute top-4 right-4 z-10">
        <LanguageSwitcher />
      </div>

      <ConfigProvider
        theme={{
          token: {
            colorPrimary: '#667eea',
            borderRadius: 8,
          },
        }}
      >
        <Card
          className="w-full max-w-md shadow-2xl"
          styles={{
            body: { padding: '2rem' }
          }}
        >
          <Space direction="vertical" size="large" className="w-full">
            <div className="text-center">
              <h1 className="text-4xl font-bold bg-gradient-to-r from-primary-500 to-purple-600 bg-clip-text text-transparent mb-2">
                RFRP
              </h1>
              <p className="text-gray-500">{t('auth.loginTitle')}</p>
            </div>

            {error && (
              <Alert
                message={error}
                type="error"
                showIcon
                closable
                onClose={() => setError('')}
              />
            )}

            <Form
              form={form}
              name="login"
              onFinish={handleSubmit}
              layout="vertical"
              size="large"
            >
              <Form.Item
                name="username"
                rules={[{ required: true, message: t('auth.username') }]}
              >
                <Input
                  prefix={<UserOutlined />}
                  placeholder={t('auth.username')}
                />
              </Form.Item>

              <Form.Item
                name="password"
                rules={[{ required: true, message: t('auth.password') }]}
              >
                <Input.Password
                  prefix={<LockOutlined />}
                  placeholder={t('auth.password')}
                />
              </Form.Item>

              <Form.Item>
                <Button
                  type="primary"
                  htmlType="submit"
                  loading={loading}
                  block
                  className="h-12 font-semibold"
                >
                  {loading ? t('common.loading') : t('auth.loginButton')}
                </Button>
              </Form.Item>
            </Form>
          </Space>
        </Card>
      </ConfigProvider>
    </div>
  );
};
