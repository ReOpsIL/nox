import React, { useState, useEffect } from 'react';
import './NotificationCenter.css';

interface Notification {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  title: string;
  message: string;
  timestamp: Date;
  read?: boolean;
  autoClose?: boolean;
}

interface NotificationCenterProps {
  notifications: Notification[];
  onDismiss: (id: string) => void;
}

/**
 * NotificationCenter component for displaying system notifications
 */
const NotificationCenter: React.FC<NotificationCenterProps> = ({ notifications, onDismiss }) => {
  const [expanded, setExpanded] = useState(false);
  const [unreadCount, setUnreadCount] = useState(0);

  // Calculate unread count
  useEffect(() => {
    const count = notifications.filter(n => !n.read).length;
    setUnreadCount(count);
    
    // Update document title if there are unread notifications
    if (count > 0) {
      document.title = `(${count}) Nox Dashboard`;
    } else {
      document.title = 'Nox Dashboard';
    }
  }, [notifications]);

  // Auto-close notifications
  useEffect(() => {
    const autoCloseNotifications = notifications.filter(n => n.autoClose);
    
    if (autoCloseNotifications.length > 0) {
      const timers = autoCloseNotifications.map(notification => {
        return setTimeout(() => {
          onDismiss(notification.id);
        }, 5000); // Auto-close after 5 seconds
      });
      
      return () => {
        timers.forEach(timer => clearTimeout(timer));
      };
    }
  }, [notifications, onDismiss]);

  const toggleExpanded = () => {
    setExpanded(!expanded);
  };

  const handleDismiss = (id: string, event: React.MouseEvent) => {
    event.stopPropagation();
    onDismiss(id);
  };

  const getNotificationIcon = (type: string) => {
    switch (type) {
      case 'success':
        return '✓';
      case 'warning':
        return '⚠';
      case 'error':
        return '✗';
      default:
        return 'ℹ';
    }
  };

  const formatTimestamp = (timestamp: Date) => {
    const now = new Date();
    const diff = now.getTime() - new Date(timestamp).getTime();
    
    // Less than a minute
    if (diff < 60000) {
      return 'Just now';
    }
    
    // Less than an hour
    if (diff < 3600000) {
      const minutes = Math.floor(diff / 60000);
      return `${minutes} minute${minutes !== 1 ? 's' : ''} ago`;
    }
    
    // Less than a day
    if (diff < 86400000) {
      const hours = Math.floor(diff / 3600000);
      return `${hours} hour${hours !== 1 ? 's' : ''} ago`;
    }
    
    // Format as date
    return new Date(timestamp).toLocaleString();
  };

  return (
    <div className={`notification-center ${expanded ? 'expanded' : ''}`}>
      <div className="notification-toggle" onClick={toggleExpanded}>
        {unreadCount > 0 && (
          <span className="notification-badge">{unreadCount}</span>
        )}
        <span className="notification-icon">🔔</span>
      </div>
      
      {expanded && (
        <div className="notification-panel">
          <div className="notification-header">
            <h3>Notifications</h3>
            {notifications.length > 0 && (
              <button 
                className="clear-all-btn"
                onClick={() => notifications.forEach(n => onDismiss(n.id))}
              >
                Clear All
              </button>
            )}
          </div>
          
          <div className="notification-list">
            {notifications.length === 0 ? (
              <div className="no-notifications">
                No notifications
              </div>
            ) : (
              notifications.map(notification => (
                <div 
                  key={notification.id} 
                  className={`notification-item ${notification.type} ${notification.read ? 'read' : 'unread'}`}
                >
                  <div className="notification-icon">
                    {getNotificationIcon(notification.type)}
                  </div>
                  <div className="notification-content">
                    <div className="notification-title">{notification.title}</div>
                    <div className="notification-message">{notification.message}</div>
                    <div className="notification-time">{formatTimestamp(notification.timestamp)}</div>
                  </div>
                  <button 
                    className="notification-dismiss"
                    onClick={(e) => handleDismiss(notification.id, e)}
                  >
                    ×
                  </button>
                </div>
              ))
            )}
          </div>
        </div>
      )}
    </div>
  );
};

export default NotificationCenter;