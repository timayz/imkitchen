import {
  getExpirationStatus,
  getExpirationColor,
  formatDate,
} from '@/lib/utils';
import { cn } from '@/lib/utils';

interface ExpirationAlertProps {
  expirationDate: Date;
  className?: string;
}

export function ExpirationAlert({
  expirationDate,
  className,
}: ExpirationAlertProps) {
  const status = getExpirationStatus(expirationDate);
  const colorClass = getExpirationColor(status);

  const getStatusText = () => {
    switch (status) {
      case 'expired':
        return 'Expired';
      case 'expiring_soon':
        return 'Expires soon';
      case 'expiring_later':
        return 'Expires in a week';
      case 'fresh':
        return 'Fresh';
      default:
        return '';
    }
  };

  return (
    <div className={cn('flex items-center space-x-2', className)}>
      <span
        className={cn('px-2 py-1 rounded-full text-xs font-medium', colorClass)}
      >
        {getStatusText()}
      </span>
      <span className="text-xs text-gray-500">
        {formatDate(expirationDate)}
      </span>
    </div>
  );
}
