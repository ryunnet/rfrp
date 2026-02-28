interface ConfirmDialogProps {
  open: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  variant?: 'danger' | 'warning' | 'info';
  onConfirm: () => void;
  onCancel: () => void;
}

export default function ConfirmDialog({
  open,
  title,
  message,
  confirmText = '确定',
  cancelText = '取消',
  variant = 'danger',
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  if (!open) return null;

  const variantConfig = {
    danger: {
      icon: 'bg-gradient-to-br from-red-500 to-rose-600',
      button: 'bg-gradient-to-r from-red-600 to-rose-600 hover:from-red-700 hover:to-rose-700 shadow-lg shadow-red-500/25',
    },
    warning: {
      icon: 'bg-gradient-to-br from-amber-500 to-orange-600',
      button: 'bg-gradient-to-r from-amber-600 to-orange-600 hover:from-amber-700 hover:to-orange-700 shadow-lg shadow-amber-500/25',
    },
    info: {
      icon: 'bg-primary',
      button: 'bg-primary hover:bg-primary/90 shadow-sm',
    },
  };

  const config = variantConfig[variant];

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50" onClick={onCancel}>
      <div
        className="relative bg-white rounded-2xl shadow-2xl w-full max-w-sm mx-4 transform transition-all"
        onClick={e => e.stopPropagation()}
      >
        <div className="p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className={`w-10 h-10 ${config.icon} rounded-xl flex items-center justify-center`}>
              <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-5 h-5 text-primary-foreground">
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
              </svg>
            </div>
            <h3 className="text-lg font-bold text-foreground">{title}</h3>
          </div>
          <p className="text-sm text-muted-foreground ml-[52px]">{message}</p>
          <div className="mt-6 flex gap-3">
            <button
              onClick={onCancel}
              className="flex-1 px-4 py-2.5 bg-muted text-foreground font-medium rounded-xl hover:bg-accent transition-colors"
            >
              {cancelText}
            </button>
            <button
              onClick={onConfirm}
              className={`flex-1 px-4 py-2.5 text-primary-foreground font-medium rounded-xl transition-all ${config.button}`}
            >
              {confirmText}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
