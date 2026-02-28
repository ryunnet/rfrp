import { useState, useEffect, useRef } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { authService } from '../lib/services';
import { User, Lock, Eye, EyeOff, ArrowRight, Loader2, AlertCircle, Shield, Zap, Globe, Layers } from 'lucide-react';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
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

  useEffect(() => {
    requestAnimationFrame(() => setMounted(true));
    authService.getRegisterStatus().then(res => {
      if (res.success && res.data) {
        setRegisterEnabled(res.data.enabled);
      }
    }).catch(() => {});
  }, []);

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
    <div className="min-h-screen flex login-bg-animated" style={{ background: 'linear-gradient(135deg, hsl(222 60% 8%), hsl(210 100% 14%), hsl(210 100% 22%), hsl(189 80% 18%), hsl(210 100% 14%), hsl(222 60% 8%))' }}>
      {/* 全局背景装饰 */}
      <div className="fixed inset-0 pointer-events-none overflow-hidden">
        <div className="absolute -top-40 -right-40 w-[500px] h-[500px] rounded-full blur-[100px] login-float" style={{ background: 'hsl(210 100% 50% / 0.15)' }} />
        <div className="absolute -bottom-40 -left-40 w-[400px] h-[400px] rounded-full blur-[100px] login-float-delayed" style={{ background: 'hsl(189 94% 43% / 0.15)' }} />
        <div className="absolute top-1/4 right-1/4 w-[350px] h-[350px] rounded-full blur-[80px] login-float-slow" style={{ background: 'hsl(263 70% 58% / 0.08)' }} />
        <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNDAiIGhlaWdodD0iNDAiIHZpZXdCb3g9IjAgMCA0MCA0MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48Y2lyY2xlIGN4PSIxIiBjeT0iMSIgcj0iMC41IiBmaWxsPSJyZ2JhKDI1NSwyNTUsMjU1LDAuMDMpIi8+PC9zdmc+')] opacity-60" />
      </div>

      {/* ===== 左侧品牌面板 (桌面端) ===== */}
      <div className="hidden lg:flex lg:w-[55%] relative items-center justify-center p-12">
        {/* 装饰性网络线条 */}
        <svg className="absolute inset-0 w-full h-full opacity-[0.04]" xmlns="http://www.w3.org/2000/svg">
          <defs>
            <pattern id="grid" width="60" height="60" patternUnits="userSpaceOnUse">
              <path d="M 60 0 L 0 0 0 60" fill="none" stroke="white" strokeWidth="0.5"/>
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#grid)" />
        </svg>

        <div className={`relative z-10 max-w-lg transition-all duration-1000 ease-out ${mounted ? 'opacity-100 translate-x-0' : 'opacity-0 -translate-x-12'}`}>
          {/* Logo + 品牌 */}
          <div className="mb-12">
            <div className="w-16 h-16 rounded-2xl flex items-center justify-center mb-6 login-logo-glow" style={{ background: 'linear-gradient(135deg, #0f172a, #1e3a5f)', border: '1px solid rgba(56, 189, 248, 0.2)' }}>
              <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" fill="none" className="w-9 h-9">
                <path d="M8 14 L36 14 L36 9 L48 18 L36 27 L36 22 L8 22Z" fill="#38bdf8" opacity="0.95"/>
                <path d="M56 50 L28 50 L28 55 L16 46 L28 37 L28 42 L56 42Z" fill="#818cf8" opacity="0.95"/>
                <circle cx="32" cy="32" r="2.5" fill="#e2e8f0" opacity="0.7"/>
              </svg>
            </div>
            <h1 className="text-5xl font-bold text-white mb-3 tracking-tight">RFRP</h1>
            <p className="text-xl text-white/50 font-light">高性能反向代理 · 内网穿透平台</p>
          </div>

          {/* 特性卡片 */}
          <div className="space-y-4">
            {[
              { icon: Zap, color: '#38bdf8', title: '极速穿透', desc: '基于 QUIC/KCP 协议，低延迟高吞吐' },
              { icon: Globe, title: '多节点部署', color: '#818cf8', desc: '灵活分布式架构，就近接入加速' },
              { icon: Layers, title: '统一管理', color: '#34d399', desc: '可视化仪表盘，流量配额一目了然' },
            ].map((feat, i) => (
              <div
                key={feat.title}
                className="flex items-start gap-4 p-4 rounded-xl transition-all duration-300 hover:bg-white/[0.04]"
                style={{ animationDelay: `${0.3 + i * 0.15}s` }}
              >
                <div className="w-10 h-10 rounded-lg flex items-center justify-center shrink-0" style={{ background: `${feat.color}15`, border: `1px solid ${feat.color}30` }}>
                  <feat.icon className="w-5 h-5" style={{ color: feat.color }} />
                </div>
                <div>
                  <h3 className="text-white/90 font-medium text-sm mb-0.5">{feat.title}</h3>
                  <p className="text-white/35 text-sm leading-relaxed">{feat.desc}</p>
                </div>
              </div>
            ))}
          </div>

          {/* 装饰性连接动画 */}
          <div className="mt-12 flex items-center gap-3 text-white/20 text-xs">
            <div className="flex items-center gap-1.5">
              <span className="w-2 h-2 rounded-full bg-emerald-400/60 animate-pulse" />
              <span>服务运行中</span>
            </div>
            <span>·</span>
            <span>安全加密传输</span>
            <span>·</span>
            <span>99.9% 可用性</span>
          </div>
        </div>
      </div>

      {/* ===== 右侧登录区域 ===== */}
      <div className="w-full lg:w-[45%] flex items-center justify-center p-6 sm:p-8 relative">
        {/* 右侧面板背景 */}
        <div className="absolute inset-0 bg-white/[0.02] backdrop-blur-sm hidden lg:block" style={{ borderLeft: '1px solid rgba(255,255,255,0.06)' }} />

        <div className={`relative z-10 w-full max-w-[400px] transition-all duration-700 ease-out ${mounted ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-8'}`}>
          {/* 移动端 Logo */}
          <div className="lg:hidden text-center mb-8 login-stagger-1">
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

          {/* 登录标题 */}
          <div className="mb-8 login-stagger-1">
            <h2 className="text-2xl font-semibold text-white/95 mb-1.5">欢迎回来</h2>
            <p className="text-white/40 text-sm">请登录您的账户以继续</p>
          </div>

          {/* 错误提示 */}
          {error && (
            <Alert variant="destructive" className="login-alert flex items-center gap-3 mb-6 login-fade-in">
              <AlertCircle className="w-5 h-5 shrink-0" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {/* 登录表单 */}
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
                  placeholder="请输入用户名"
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
                  ref={passwordRef}
                  id="password"
                  name="password"
                  type={showPassword ? 'text' : 'password'}
                  required
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="h-11 pl-10 pr-12 rounded-lg bg-white/[0.06] border-white/[0.08] text-white placeholder:text-white/20 hover:bg-white/[0.09] focus-visible:bg-white/[0.09] focus-visible:ring-0 focus-visible:border-transparent transition-all"
                  placeholder="请输入密码"
                  disabled={loading}
                  autoComplete="current-password"
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

            {/* 记住用户名 */}
            <div className="flex items-center gap-2 login-stagger-4">
              <input
                type="checkbox"
                id="remember"
                checked={rememberUsername}
                onChange={(e) => setRememberUsername(e.target.checked)}
                className="login-checkbox"
                disabled={loading}
              />
              <label htmlFor="remember" className="text-xs text-white/30 cursor-pointer select-none">
                记住用户名
              </label>
            </div>

            {/* 登录按钮 */}
            <div className="login-stagger-4 pt-1">
              <Button
                type="submit"
                disabled={loading}
                className="w-full h-11 text-white hover:opacity-90 shadow-lg transition-all duration-300 login-btn-glow rounded-lg font-medium"
                style={{ background: 'linear-gradient(135deg, hsl(210 100% 45%), hsl(189 94% 43%))' }}
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
            </div>
          </form>

          {/* 底部链接 */}
          <div className="mt-8 space-y-4 login-stagger-5">
            <div className="h-px bg-gradient-to-r from-transparent via-white/[0.08] to-transparent" />

            {registerEnabled && (
              <p className="text-center text-sm text-white/35">
                没有账号？{' '}
                <Link to="/register" className="text-cyan-400/70 hover:text-cyan-400 font-medium transition-colors">
                  立即注册
                </Link>
              </p>
            )}

            <p className="text-center text-xs text-white/15 flex items-center justify-center gap-1.5">
              <Shield className="w-3 h-3" />
              安全登录 · 数据加密传输
            </p>
          </div>

          {/* 版权 */}
          <p className="text-center text-xs text-white/15 mt-8 login-stagger-6">
            &copy; {new Date().getFullYear()} RFRP. All rights reserved.
          </p>
        </div>
      </div>
    </div>
  );
}
