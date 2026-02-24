// 格式化字节大小
export function formatBytes(bytes: number | undefined | null): string {
  if (!bytes || bytes === 0 || isNaN(bytes)) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

// 格式化日期
export function formatDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

// 格式化日期为短格式
export function formatShortDate(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  });
}

// 复制到剪贴板
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    // 优先使用现代 Clipboard API
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(text);
      return true;
    }

    // 降级方案：使用传统的 execCommand 方法
    const textArea = document.createElement('textarea');
    textArea.value = text;
    textArea.style.position = 'fixed';
    textArea.style.left = '-999999px';
    textArea.style.top = '-999999px';
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();

    const successful = document.execCommand('copy');
    document.body.removeChild(textArea);

    return successful;
  } catch (error) {
    console.error('复制失败:', error);
    return false;
  }
}

// 获取状态颜色
export function getStatusColor(enabled: boolean): string {
  return enabled ? 'text-green-600' : 'text-gray-400';
}

// 获取状态背景色
export function getStatusBgColor(enabled: boolean): string {
  return enabled ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800';
}

// 获取在线状态颜色
export function getOnlineStatusColor(isOnline: boolean): string {
  return isOnline ? 'bg-green-500' : 'bg-gray-400';
}
