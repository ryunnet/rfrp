import { useState, useEffect, useRef } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { authService } from '../lib/services';
import { User, Lock, Eye, EyeOff, ArrowRight, Loader2, AlertCircle, CheckCircle2, Shield } from 'lucide-react';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { Alert, AlertDescription } from '../components/ui/alert';

export default function Register() {
  const navigate = useNavigate();
  const { login } = useAuth();
  const usernameRef = useRef<HTMLInputElement>(null);

  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState('');
  const [shakeError, setShakeError] = useState(false);
  const [loading, setLoading] = useState(false);
  const [mounted, setMounted] = useState(false);
  const [registrationEnabled, setRegistrationEnabled] = useState<boolean | null>(null);

  useEffect(() => {
    requestAnimationFrame(() => setMounted(true));
    checkRegistrationStatus();
  }, []);

  useEffect(() => {
    if (registrationEnabled === true) {
      setTimeout(() => usernameRef.current?.focus(), 300);
    }
  }, [registrationEnabled]);

  const checkRegistrationStatus = async () => {
    try {
      const response = await authService.getRegisterStatus();
      if (response.success && response.data) {
        setRegistrationEnabled(response.data.enabled);
      } else {
        setRegistrationEnabled(false);
      }
    } catch {
      setRegistrationEnabled(false);
    }
  };

  const triggerShake = () => {
    setShakeError(true);
    setTimeout(() => setShakeError(false), 500);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    const trimmedUsername = username.trim();
    if (trimmedUsername.length < 3 || trimmedUsername.length > 20) {
      setError('用户名长度需要 3-20 个字符');
      triggerShake();
      return;
    }

    if (password.length < 6) {
      setError('密码长度不能少于 6 个字符');
      triggerShake();
      return;
    }

    if (password !== confirmPassword) {
      setError('两次输入的密码不一致');
      triggerShake();
      return;
    }

    setLoading(true);

    try {
      const response = await authService.register({ username: trimmedUsername, password });
      if (response.success && response.data) {
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
        setError(response.message || '注册失败');
        triggerShake();
      }
    } catch (err) {
      console.error('注册错误:', err);
      setError('注册失败，请稍后重试');
      triggerShake();
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8 relative overflow-hidden login-bg-animated" style={{ background: 'linear-gradient(135deg, hsl(222 60% 8%), hsl(210 100% 14%), hsl(210 100% 22%), hsl(189 80% 18%), hsl(210 100% 14%), hsl(222 60% 8%))' }}>
      {/* 背景装饰 */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute -top-40 -right-40 w-[500px] h-[500px] rounded-full blur-[100px] login-float" style={{ background: 'hsl(210 100% 50% / 0.15)' }} />
        <div className="absolute -bottom-40 -left-40 w-[400px] h-[400px] rounded-full blur-[100px] login-float-delayed" style={{ background: 'hsl(189 94% 43% / 0.15)' }} />
        <div className="absolute top-1/4 right-1/4 w-[350px] h-[350px] rounded-full blur-[80px] login-float-slow" style={{ background: 'hsl(263 70% 58% / 0.08)' }} />
        <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNDAiIGhlaWdodD0iNDAiIHZpZXdCb3g9IjAgMCA0MCA0MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48Y2lyY2xlIGN4PSIxIiBjeT0iMSIgcj0iMC41IiBmaWxsPSJyZ2JhKDI1NSwyNTUsMjU1LDAuMDMpIi8+PC9zdmc+')] opacity-60" />
      </div>

      <div className={`relative w-full max-w-[400px] transition-all duration-700 ease-out ${mounted ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-8'}`}>
        {/* Logo */}
        <div className="text-center mb-8 login-stagger-1">
          <div className="mx-auto w-16 h-16 rounded-2xl flex items-center justify-center mb-4 login-logo-glow" style={{ background: 'linear-gradient(135deg, #0f172a, #1e3a5f)', border: '1px solid rgba(56, 189, 248, 0.2)' }}>
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" fill="none" className="w-9 h-9">
              <path d="M8 14 L36 14 L36 9 L48 18 L36 27 L36 22 L8 22Z" fill="#38bdf8" opacity="0.95"/>
              <path d="M56 50 L28 50 L28 55 L16 46 L28 37 L28 42 L56 42Z" fill="#818cf8" opacity="0.95"/>
              <circle cx="32" cy="32" r="2.5" fill="#e2e8f0" opacity="0.7"/>
            </svg>
          </div>
          <h1 className="text-3xl font-bold text-white tracking-tight">RFRP</h1>
          <p className="text-white/40 text-sm mt-1">高性能内网穿透服务</p>
        </div>

        {/* 注册标题 */}
        <div className="mb-8 login-stagger-1">
          <h2 className="text-2xl font-semibold text-white/95 mb-1.5">创建账号</h2>
          <p className="text-white/40 text-sm">注册一个新账号以开始使用</p>
        </div>

        {registrationEnabled === null ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="w-6 h-6 animate-spin text-white/50" />
          </div>
        ) : registrationEnabled === false ? (
          <Alert className="flex items-center gap-3 bg-amber-500/10 border-amber-500/30 text-amber-300">
            <AlertCircle className="w-5 h-5 shrink-0" />
            <AlertDescription>注册功能暂未开放，请联系管理员</AlertDescription>
          </Alert>
        ) : (
          <>
            {/* 错误提示 */}
            {error && (
              <Alert variant="destructive" className="flex items-center gap-3 mb-6 bg-red-500/10 border-red-500/30 text-red-300 login-fade-in">
                <AlertCircle className="w-5 h-5 shrink-0" />
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}

            <form className={`space-y-5 ${shakeError ? 'login-shake' : ''}`} onSubmit={handleSubmit}>
              {/* 用户名 */}
              <div className="space-y-2 login-stagger-2">
                <Label htmlFor="username" className="text-white/50 text-xs font-medium uppercase tracking-wider">
                  用户名
                </Label>
                <div className="relative group login-input-glow rounded-lg transition-all duration-300">
                  <div className="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none">
                    <User className="w-4 h-4 text-white/25 group-focus-within:text-cyan-400/70 transition-colors duration-300" />
                  </div>
                  <Input
                    ref={usernameRef}
                    id="username"
                    name="username"
                    type="text"
                    required
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                    className="h-11 pl-10 rounded-lg bg-white/[0.06] border-white/[0.08] text-white placeholder:text-white/20 hover:bg-white/[0.09] focus-visible:bg-white/[0.09] focus-visible:ring-0 focus-visible:border-transparent transition-all"
                    placeholder="3-20 个字符"
                    disabled={loading}
                    autoComplete="username"
                  />
                </div>
              </div>

              {/* 密码 */}
              <div className="space-y-2 login-stagger-3">
                <Label htmlFor="password" className="text-white/50 text-xs font-medium uppercase tracking-wider">
                  密码
                </Label>
                <div className="relative group login-input-glow rounded-lg transition-all duration-300">
                  <div className="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none">
                    <Lock className="w-4 h-4 text-white/25 group-focus-within:text-cyan-400/70 transition-colors duration-300" />
                  </div>
                  <Input
                    id="password"
                    name="password"
                    type={showPassword ? 'text' : 'password'}
                    required
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="h-11 pl-10 pr-12 rounded-lg bg-white/[0.06] border-white/[0.08] text-white placeholder:text-white/20 hover:bg-white/[0.09] focus-visible:bg-white/[0.09] focus-visible:ring-0 focus-visible:border-transparent transition-all"
                    placeholder="至少 6 个字符"
                    disabled={loading}
                    autoComplete="new-password"
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword(!showPassword)}
                    className="absolute inset-y-0 right-0 flex items-center pr-3.5 text-white/20 hover:text-white/50 transition-colors"
                    disabled={loading}
                    tabIndex={-1}
                  >
                    {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                  </button>
                </div>
              </div>

              {/* 确认密码 */}
              <div className="space-y-2 login-stagger-4">
                <Label htmlFor="confirmPassword" className="text-white/50 text-xs font-medium uppercase tracking-wider">
                  确认密码
                </Label>
                <div className="relative group login-input-glow rounded-lg transition-all duration-300">
                  <div className="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none">
                    <CheckCircle2 className="w-4 h-4 text-white/25 group-focus-within:text-cyan-400/70 transition-colors duration-300" />
                  </div>
                  <Input
                    id="confirmPassword"
                    name="confirmPassword"
                    type={showPassword ? 'text' : 'password'}
                    required
                    value={confirmPassword}
                    onChange={(e) => setConfirmPassword(e.target.value)}
                    className="h-11 pl-10 rounded-lg bg-white/[0.06] border-white/[0.08] text-white placeholder:text-white/20 hover:bg-white/[0.09] focus-visible:bg-white/[0.09] focus-visible:ring-0 focus-visible:border-transparent transition-all"
                    placeholder="再次输入密码"
                    disabled={loading}
                    autoComplete="new-password"
                  />
                </div>
              </div>

              {/* 注册按钮 */}
              <div className="login-stagger-5 pt-1">
                <Button
                  type="submit"
                  disabled={loading}
                  className="w-full h-11 text-white hover:opacity-90 shadow-lg transition-all duration-300 login-btn-glow rounded-lg font-medium"
                  style={{ background: 'linear-gradient(135deg, hsl(210 100% 45%), hsl(189 94% 43%))' }}
                >
                  {loading ? (
                    <>
                      <Loader2 className="w-5 h-5 animate-spin" />
                      <span>注册中...</span>
                    </>
                  ) : (
                    <>
                      <span>注册</span>
                      <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-0.5" />
                    </>
                  )}
                </Button>
              </div>
            </form>
          </>
        )}

        {/* 底部链接 */}
        <div className="mt-8 space-y-4 login-stagger-6">
          <div className="h-px bg-gradient-to-r from-transparent via-white/[0.08] to-transparent" />

          <p className="text-center text-sm text-white/35">
            已有账号？{' '}
            <Link to="/login" className="text-cyan-400/70 hover:text-cyan-400 font-medium transition-colors">
              返回登录
            </Link>
          </p>

          <p className="text-center text-xs text-white/15 flex items-center justify-center gap-1.5">
            <Shield className="w-3 h-3" />
            安全注册 · 数据加密传输
          </p>
        </div>

        {/* 版权 */}
        <p className="text-center text-xs text-white/15 mt-8 login-stagger-6">
          &copy; {new Date().getFullYear()} RFRP. All rights reserved.
        </p>
      </div>
    </div>
  );
}
