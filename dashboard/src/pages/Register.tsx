import { useState, useEffect, useRef } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { authService } from '../lib/services';
import { Server, User, Lock, Eye, EyeOff, ArrowRight, Loader2, AlertCircle, CheckCircle2 } from 'lucide-react';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '../components/ui/card';
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
    <div
      className="min-h-screen flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8 relative overflow-hidden login-bg-animated"
      style={{ background: 'linear-gradient(135deg, hsl(222 60% 8%), hsl(210 100% 14%), hsl(210 100% 22%), hsl(189 80% 18%), hsl(210 100% 14%), hsl(222 60% 8%))' }}
    >
      {/* 背景装饰 - 多层浮动光斑 */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute -top-40 -right-40 w-[500px] h-[500px] rounded-full blur-[100px] login-float" style={{ background: 'hsl(210 100% 50% / 0.18)' }} />
        <div className="absolute -bottom-40 -left-40 w-[400px] h-[400px] rounded-full blur-[100px] login-float-delayed" style={{ background: 'hsl(189 94% 43% / 0.18)' }} />
        <div className="absolute top-1/4 right-1/4 w-[350px] h-[350px] rounded-full blur-[80px] login-float-slow" style={{ background: 'hsl(172 66% 50% / 0.12)' }} />
        <div className="absolute bottom-1/4 left-1/3 w-[250px] h-[250px] rounded-full blur-[80px] login-float-alt" style={{ background: 'hsl(230 80% 60% / 0.1)' }} />
        <div className="absolute top-1/2 right-1/3 w-[200px] h-[200px] rounded-full blur-[60px] login-float-alt-delayed" style={{ background: 'hsl(200 90% 55% / 0.08)' }} />
        {/* 网格纹理 */}
        <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNDAiIGhlaWdodD0iNDAiIHZpZXdCb3g9IjAgMCA0MCA0MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48Y2lyY2xlIGN4PSIxIiBjeT0iMSIgcj0iMC41IiBmaWxsPSJyZ2JhKDI1NSwyNTUsMjU1LDAuMDMpIi8+PC9zdmc+')] opacity-60" />
      </div>

      <div className={`relative max-w-md w-full space-y-6 transition-all duration-700 ease-out ${mounted ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-6'}`}>
        {/* Logo 和标题 */}
        <div className="text-center space-y-4 login-stagger-1">
          <div className="mx-auto w-20 h-20 rounded-3xl flex items-center justify-center shadow-2xl animate-gradient login-logo-glow" style={{ background: 'linear-gradient(135deg, hsl(210 100% 45%), hsl(189 94% 43%))' }}>
            <Server className="w-10 h-10 text-white drop-shadow-lg" strokeWidth={2.5} />
          </div>
          <div>
            <h1 className="text-4xl font-bold text-white mb-2 tracking-tight">
              RFRP
            </h1>
            <p className="text-white/60 text-sm font-medium tracking-wide">
              高性能内网穿透服务
            </p>
          </div>
        </div>

        {/* 注册卡片 */}
        <Card
          className={`login-card-glow p-2 border border-white/[0.12] border-t-white/[0.2] bg-white/[0.06] backdrop-blur-2xl shadow-[0_8px_32px_rgba(0,0,0,0.3),0_0_0_1px_rgba(255,255,255,0.05)] ${shakeError ? 'login-shake' : ''} login-stagger-2`}
        >
          <CardHeader className="space-y-1 pb-4">
            <CardTitle className="text-2xl text-center text-white/95">
              创建账号
            </CardTitle>
            <CardDescription className="text-center text-white/45">
              注册一个新账号以开始使用
            </CardDescription>
          </CardHeader>

          <CardContent className="space-y-4">
            {registrationEnabled === null ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="w-6 h-6 animate-spin text-white/50" />
              </div>
            ) : registrationEnabled === false ? (
              <Alert className="flex items-center gap-3 bg-amber-500/10 border-amber-500/30 text-amber-300">
                <AlertCircle className="w-5 h-5 shrink-0" />
                <AlertDescription>注册功能暂未开放，请联系管理员</AlertDescription>
              </Alert>
            ) : (
              <>
                {error && (
                  <Alert variant="destructive" className="flex items-center gap-3 bg-red-500/10 border-red-500/30 text-red-300 login-fade-in">
                    <AlertCircle className="w-5 h-5 shrink-0" />
                    <AlertDescription>{error}</AlertDescription>
                  </Alert>
                )}

                <form className="space-y-4" onSubmit={handleSubmit}>
                  {/* 用户名 */}
                  <div className="space-y-2 login-stagger-3">
                    <Label htmlFor="username" className="text-white/60 text-xs font-medium uppercase tracking-wider">
                      用户名
                    </Label>
                    <div className="relative group login-input-glow rounded-md transition-all duration-300">
                      <div className="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none">
                        <User className="w-4 h-4 text-white/30 group-focus-within:text-cyan-400/70 transition-colors duration-300" />
                      </div>
                      <Input
                        ref={usernameRef}
                        id="username"
                        name="username"
                        type="text"
                        required
                        value={username}
                        onChange={(e) => setUsername(e.target.value)}
                        className="pl-10 bg-white/[0.07] border-white/[0.1] text-white placeholder:text-white/25 hover:bg-white/[0.1] focus-visible:bg-white/[0.1] focus-visible:ring-0 focus-visible:border-transparent transition-all"
                        placeholder="3-20 个字符"
                        disabled={loading}
                        autoComplete="username"
                      />
                    </div>
                  </div>

                  {/* 密码 */}
                  <div className="space-y-2 login-stagger-4">
                    <Label htmlFor="password" className="text-white/60 text-xs font-medium uppercase tracking-wider">
                      密码
                    </Label>
                    <div className="relative group login-input-glow rounded-md transition-all duration-300">
                      <div className="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none">
                        <Lock className="w-4 h-4 text-white/30 group-focus-within:text-cyan-400/70 transition-colors duration-300" />
                      </div>
                      <Input
                        id="password"
                        name="password"
                        type={showPassword ? 'text' : 'password'}
                        required
                        value={password}
                        onChange={(e) => setPassword(e.target.value)}
                        className="pl-10 pr-12 bg-white/[0.07] border-white/[0.1] text-white placeholder:text-white/25 hover:bg-white/[0.1] focus-visible:bg-white/[0.1] focus-visible:ring-0 focus-visible:border-transparent transition-all"
                        placeholder="至少 6 个字符"
                        disabled={loading}
                        autoComplete="new-password"
                      />
                      <button
                        type="button"
                        onClick={() => setShowPassword(!showPassword)}
                        className="absolute inset-y-0 right-0 flex items-center pr-3.5 text-white/25 hover:text-white/60 transition-colors"
                        disabled={loading}
                        tabIndex={-1}
                      >
                        {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                      </button>
                    </div>
                  </div>

                  {/* 确认密码 */}
                  <div className="space-y-2 login-stagger-5">
                    <Label htmlFor="confirmPassword" className="text-white/60 text-xs font-medium uppercase tracking-wider">
                      确认密码
                    </Label>
                    <div className="relative group login-input-glow rounded-md transition-all duration-300">
                      <div className="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none">
                        <CheckCircle2 className="w-4 h-4 text-white/30 group-focus-within:text-cyan-400/70 transition-colors duration-300" />
                      </div>
                      <Input
                        id="confirmPassword"
                        name="confirmPassword"
                        type={showPassword ? 'text' : 'password'}
                        required
                        value={confirmPassword}
                        onChange={(e) => setConfirmPassword(e.target.value)}
                        className="pl-10 bg-white/[0.07] border-white/[0.1] text-white placeholder:text-white/25 hover:bg-white/[0.1] focus-visible:bg-white/[0.1] focus-visible:ring-0 focus-visible:border-transparent transition-all"
                        placeholder="再次输入密码"
                        disabled={loading}
                        autoComplete="new-password"
                      />
                    </div>
                  </div>

                  {/* 注册按钮 */}
                  <div className="login-stagger-6">
                    <Button
                      type="submit"
                      disabled={loading}
                      className="w-full text-white hover:opacity-90 shadow-lg transition-all duration-300 login-btn-glow"
                      style={{ background: 'linear-gradient(135deg, hsl(210 100% 45%), hsl(189 94% 43%))' }}
                      size="lg"
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
          </CardContent>

          <CardFooter className="flex flex-col space-y-2 login-stagger-6">
            <div className="w-full border-t border-white/[0.06] pt-4">
              <p className="text-center text-sm text-white/45">
                已有账号？{' '}
                <Link to="/login" className="text-white/75 hover:text-white font-medium transition-colors">
                  返回登录
                </Link>
              </p>
            </div>
          </CardFooter>
        </Card>

        {/* 版权信息 */}
        <p className="text-center text-sm text-white/25 login-stagger-6">
          &copy; {new Date().getFullYear()} RFRP. All rights reserved.
        </p>
      </div>
    </div>
  );
}
