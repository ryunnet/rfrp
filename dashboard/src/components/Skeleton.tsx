interface SkeletonProps {
  className?: string;
}

function SkeletonBlock({ className = '' }: SkeletonProps) {
  return (
    <div className={`animate-pulse bg-gray-200 rounded-lg ${className}`} />
  );
}

export function TableSkeleton({ rows = 5, cols = 5 }: { rows?: number; cols?: number }) {
  return (
    <div className="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden">
      <div className="bg-gradient-to-r from-gray-50 to-gray-100/50 px-6 py-4 flex gap-6">
        {Array.from({ length: cols }).map((_, i) => (
          <SkeletonBlock key={i} className="h-4 flex-1" />
        ))}
      </div>
      <div className="divide-y divide-gray-100">
        {Array.from({ length: rows }).map((_, i) => (
          <div key={i} className="px-6 py-4 flex items-center gap-6">
            {Array.from({ length: cols }).map((_, j) => (
              <SkeletonBlock key={j} className={`h-4 flex-1 ${j === 0 ? 'max-w-[80px]' : ''}`} />
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}

export function CardSkeleton() {
  return (
    <div className="bg-white rounded-2xl p-5 shadow-sm border border-gray-100 animate-pulse">
      <div className="flex items-center justify-between">
        <div className="space-y-3 flex-1">
          <SkeletonBlock className="h-4 w-20" />
          <SkeletonBlock className="h-8 w-16" />
        </div>
        <SkeletonBlock className="h-12 w-12 rounded-xl" />
      </div>
    </div>
  );
}

export function DashboardSkeleton() {
  return (
    <div className="space-y-6">
      <SkeletonBlock className="h-28 rounded-2xl" />
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <CardSkeleton key={i} />
        ))}
      </div>
      <div className="bg-white rounded-2xl shadow-sm border border-gray-100 p-6 space-y-4">
        <SkeletonBlock className="h-6 w-32" />
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-3">
          {Array.from({ length: 3 }).map((_, i) => (
            <SkeletonBlock key={i} className="h-28 rounded-xl" />
          ))}
        </div>
      </div>
    </div>
  );
}

export default SkeletonBlock;
