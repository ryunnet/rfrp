import { useState, useEffect, useRef } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { authService } from '../lib/services';
import { Server, User, Lock, Eye, EyeOff, ArrowRight, Loader2, AlertCircle, Shield } from 'lucide-react';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '../components/ui/card';
import { Alert, AlertDescription } from '../components/ui/alert';

const REMEMBER_KEY = 'rfrp_remember_username';

export default function Login() {
  const navigate = useNavigate();
  const { login } = useAuth();
  const usernameRef = useRef<HTMLInputElement>(null);
  const passwordRef = useRef<HTMLInputElement>(null);

  const savedUsername = localStorage.getItem(REMEMBER_KEY) || '';
  const [username, setUsername] = useState(savedUsername);
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [rememberUsername, setRememberUsername] = useState(!!savedUsername);
  const [error, setError] = useState('');
  const [shakeError, setShakeError] = useState(false);
  const [loading, setLoading] = useState(false);
  const [mounted, setMounted] = useState(false);
  const [registerEnabled, setRegisterEnabled] = useState(false);

  // 入场动画
  useEffect(() => {
    requestAnimationFrame(() => setMounted(true));
    authService.getRegisterStatus().then(res => {
      if (res.success && res.data) {
        setRegisterEnabled(res.data.enabled);
      }
    }).catch(() => {});
  }, []);

  // 自动聚焦：有记住的用户名则聚焦密码框，否则聚焦用户名框
  useEffect(() => {
    const timer = setTimeout(() => {
      if (savedUsername) {
        passwordRef.current?.focus();
      } else {
        usernameRef.current?.focus();
      }
    }, 300);
    return () => clearTimeout(timer);
  }, [savedUsername]);

  const triggerShake = () => {
    setShakeError(true);
    setTimeout(() => setShakeError(false), 500);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setLoading(true);

    try {
      const response = await authService.login({ username, password });
      if (response.success && response.data) {
        // 记住用户名
        if (rememberUsername) {
          localStorage.setItem(REMEMBER_KEY, username);
        } else {
          localStorage.removeItem(REMEMBER_KEY);
        }

        const { token, user } = response.data;
        login(token, {
          id: user.id,
          username: user.username,
          is_admin: user.is_admin,
          created_at: '',
          updated_at: '',
          totalBytesSent: 0,
          totalBytesReceived: 0,
          trafficQuotaGb: null,
          remainingQuotaGb: null,
          trafficResetCycle: 'none',
          lastResetAt: null,
          isTrafficExceeded: false,
          maxPortCount: null,
          allowedPortRange: null,
        });
        navigate('/');
      } else {
        setError(response.message || '登录失败');
        triggerShake();
      }
    } catch (err) {
      console.error('登录错误:', err);
      setError('登录失败，请检查用户名和密码');
      triggerShake();
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8 relative overflow-hidden" style={{ background: 'linear-gradient(135deg, hsl(210 100% 12%), hsl(210 100% 28%), hsl(189 94% 30%))' }}>
      {/* 背景装饰 */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute -top-40 -right-40 w-[500px] h-[500px] rounded-full blur-3xl login-float" style={{ background: 'hsl(210 100% 45% / 0.2)' }} />
        <div className="absolute -bottom-40 -left-40 w-[400px] h-[400px] rounded-full blur-3xl login-float-delayed" style={{ background: 'hsl(189 94% 43% / 0.2)' }} />
        <div className="absolute top-1/3 right-1/4 w-[300px] h-[300px] rounded-full blur-3xl login-float-slow" style={{ background: 'hsl(172 66% 50% / 0.15)' }} />
        {/* 网格纹理 */}
        <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNDAiIGhlaWdodD0iNDAiIHZpZXdCb3g9IjAgMCA0MCA0MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48Y2lyY2xlIGN4PSIxIiBjeT0iMSIgcj0iMC41IiBmaWxsPSJyZ2JhKDI1NSwyNTUsMjU1LDAuMDMpIi8+PC9zdmc+')] opacity-60" />
      </div>

      <div className={`relative max-w-md w-full space-y-6 transition-all duration-700 ease-out ${mounted ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-6'}`}>
        {/* Logo 和标题 */}
        <div className="text-center space-y-4">
          <div className="mx-auto w-20 h-20 rounded-3xl flex items-center justify-center shadow-2xl animate-gradient login-logo-glow" style={{ background: 'linear-gradient(135deg, hsl(210 100% 45%), hsl(189 94% 43%))' }}>
            <Server className="w-10 h-10 text-primary-foreground drop-shadow-lg" strokeWidth={2.5} />
          </div>
          <div>
            <h1 className="text-4xl font-bold text-primary-foreground mb-2 tracking-tight">
              RFRP
            </h1>
            <p className="text-primary-foreground/80 text-sm font-medium tracking-wide">
              高性能内网穿透服务
            </p>
          </div>
        </div>

        {/* 登录卡片 */}
        <Card className={`p-2 border-border/10 bg-card/[0.07] backdrop-blur-xl shadow-2xl shadow-black/20 ${shakeError ? 'login-shake' : ''}`}>
          <CardHeader className="space-y-1 pb-4">
            <CardTitle className="text-2xl text-center text-primary-foreground/95">
              欢迎回来
            </CardTitle>
            <CardDescription className="text-center text-primary-foreground/50">
              请登录您的账户以继续
            </CardDescription>
          </CardHeader>

          <CardContent className="space-y-4">
            {/* 错误提示 */}
            {error && (
              <Alert variant="destructive" className="flex items-center gap-3 bg-red-500/10 border-red-500/30 text-red-300 login-fade-in">
                <AlertCircle className="w-5 h-5 shrink-0" />
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}

            {/* 登录表单 */}
            <form className="space-y-4" onSubmit={handleSubmit}>
              {/* 用户名输入框 */}
              <div className="space-y-2">
                <Label htmlFor="username" className="text-primary-foreground/70 text-xs font-medium uppercase tracking-wider">
                  用户名
                </Label>
                <div className="relative group">
                  <div className="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none">
                    <User className="w-4 h-4 text-primary-foreground/40 group-focus-within:text-primary-foreground/60 transition-colors" />
                  </div>
                  <Input
                    ref={usernameRef}
                    id="username"
                    name="username"
                    type="text"
                    required
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                    className="pl-10 bg-card/[0.06] border-border/10 text-primary-foreground placeholder:text-primary-foreground/25 hover:bg-card/[0.09] focus-visible:bg-card/[0.09] focus-visible:ring-primary/30 focus-visible:border-primary/50 transition-all"
                    placeholder="请输入用户名"
                    disabled={loading}
                    autoComplete="username"
                  />
                </div>
              </div>

              {/* 密码输入框 */}
              <div className="space-y-2">
                <Label htmlFor="password" className="text-primary-foreground/70 text-xs font-medium uppercase tracking-wider">
                  密码
                </Label>
                <div className="relative group">
                  <div className="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none">
                    <Lock className="w-4 h-4 text-primary-foreground/40 group-focus-within:text-primary-foreground/60 transition-colors" />
                  </div>
                  <Input
                    ref={passwordRef}
                    id="password"
                    name="password"
                    type={showPassword ? 'text' : 'password'}
                    required
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="pl-10 pr-12 bg-card/[0.06] border-border/10 text-primary-foreground placeholder:text-primary-foreground/25 hover:bg-card/[0.09] focus-visible:bg-card/[0.09] focus-visible:ring-primary/30 focus-visible:border-primary/50 transition-all"
                    placeholder="请输入密码"
                    disabled={loading}
                    autoComplete="current-password"
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword(!showPassword)}
                    className="absolute inset-y-0 right-0 flex items-center pr-3.5 text-primary-foreground/30 hover:text-primary-foreground/60 transition-colors"
                    disabled={loading}
                    tabIndex={-1}
                  >
                    {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                  </button>
                </div>
              </div>

              {/* 记住用户名 */}
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="remember"
                  checked={rememberUsername}
                  onChange={(e) => setRememberUsername(e.target.checked)}
                  className="w-3.5 h-3.5 rounded border-border/20 bg-card/[0.06] text-primary focus:ring-primary/30 focus:ring-offset-0 cursor-pointer"
                  disabled={loading}
                />
                <label htmlFor="remember" className="text-xs text-primary-foreground/40 cursor-pointer select-none">
                  记住用户名
                </label>
              </div>

              {/* 登录按钮 */}
              <Button
                type="submit"
                disabled={loading}
                className="w-full text-white hover:opacity-90 shadow-sm transition-all duration-300"
                style={{ background: 'linear-gradient(135deg, hsl(210 100% 45%), hsl(189 94% 43%))' }}
                size="lg"
              >
                {loading ? (
                  <>
                    <Loader2 className="w-5 h-5 animate-spin" />
                    <span>登录中...</span>
                  </>
                ) : (
                  <>
                    <span>登录</span>
                    <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-0.5" />
                  </>
                )}
              </Button>
            </form>
          </CardContent>

          <CardFooter className="flex flex-col space-y-2">
            <div className="w-full border-t border-border/[0.06] pt-4 space-y-3">
              {registerEnabled && (
                <p className="text-center text-sm text-primary-foreground/50">
                  没有账号？{' '}
                  <Link to="/register" className="text-primary-foreground/80 hover:text-primary-foreground font-medium transition-colors">
                    立即注册
                  </Link>
                </p>
              )}
              <p className="text-center text-xs text-primary-foreground/25 flex items-center justify-center gap-1.5">
                <Shield className="w-3 h-3" />
                安全登录 · 数据加密传输
              </p>
            </div>
          </CardFooter>
        </Card>

        {/* 版权信息 */}
        <p className="text-center text-sm text-primary-foreground/30">
          &copy; {new Date().getFullYear()} RFRP. All rights reserved.
        </p>
      </div>
    </div>
  );
}
