'use client';

import {
  Bell,
  Search,
  Settings,
  User,
  FileText,
  Server,
  ChevronRight,
  X,
  Check,
  AlertTriangle,
  Info,
  CheckCircle2,
  LogOut,
  UserCircle,
  Cog
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { ThemeToggle } from '@/components/theme-toggle';
import { Badge } from '@/components/ui/badge';
import { useLoadedModels, useSystemInfo, useUnreadNotificationCount, useSearch, useNotifications, useMarkNotificationAsRead, useMarkAllNotificationsAsRead, useDismissNotification } from '@/hooks/use-tauri-api';
import { Skeleton } from '@/components/ui/skeleton';
import { RealTimeStatus } from '@/components/ui/real-time-status';
import { useState, useRef, useEffect } from 'react';
import { SearchResult, Notification as NotificationType } from '@/types/inferno';

export function Header() {
  const { data: loadedModels, isLoading: loadedModelsLoading } = useLoadedModels();
  const { data: systemInfo, isLoading: systemLoading } = useSystemInfo();
  const { data: unreadCount, isLoading: notificationsLoading } = useUnreadNotificationCount();

  // Search state
  const [searchQuery, setSearchQuery] = useState('');
  const [showResults, setShowResults] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const searchContainerRef = useRef<HTMLDivElement>(null);

  // Notification state
  const [showNotifications, setShowNotifications] = useState(false);
  const notificationRef = useRef<HTMLDivElement>(null);

  // User menu state
  const [showUserMenu, setShowUserMenu] = useState(false);
  const userMenuRef = useRef<HTMLDivElement>(null);

  // Notification hooks
  const { data: notifications = [], isLoading: notificationsDataLoading } = useNotifications();
  const markAsReadMutation = useMarkNotificationAsRead();
  const markAllAsReadMutation = useMarkAllNotificationsAsRead();
  const dismissMutation = useDismissNotification();

  // Search hook
  const { data: searchData, isLoading: searchLoading } = useSearch(searchQuery, searchQuery.length > 0);
  const searchResults = searchData?.results || [];

  // Close dropdowns when clicking outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (searchContainerRef.current && !searchContainerRef.current.contains(event.target as Node)) {
        setShowResults(false);
        setSelectedIndex(-1);
      }
      if (notificationRef.current && !notificationRef.current.contains(event.target as Node)) {
        setShowNotifications(false);
      }
      if (userMenuRef.current && !userMenuRef.current.contains(event.target as Node)) {
        setShowUserMenu(false);
      }
    }

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Handle search input changes
  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setSearchQuery(value);
    setShowResults(value.length > 0);
    setSelectedIndex(-1);
  };

  // Handle keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!showResults || searchResults.length === 0) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(prev => (prev < searchResults.length - 1 ? prev + 1 : prev));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(prev => (prev > 0 ? prev - 1 : -1));
        break;
      case 'Enter':
        e.preventDefault();
        if (selectedIndex >= 0) {
          handleResultSelect(searchResults[selectedIndex]);
        }
        break;
      case 'Escape':
        setShowResults(false);
        setSelectedIndex(-1);
        searchInputRef.current?.blur();
        break;
    }
  };

  // Handle result selection
  const handleResultSelect = (result: SearchResult) => {
    if (result.url) {
      window.location.href = result.url;
    }
    setShowResults(false);
    setSearchQuery('');
    setSelectedIndex(-1);
  };

  // Get icon for search result type
  const getResultIcon = (type: SearchResult['type']) => {
    switch (type) {
      case 'model':
        return Server;
      case 'batch_job':
        return FileText;
      case 'notification':
        return Bell;
      case 'setting':
        return Settings;
      case 'page':
        return FileText;
      default:
        return FileText;
    }
  };

  // Notification handlers
  const handleNotificationClick = (notification: NotificationType) => {
    if (!notification.read) {
      markAsReadMutation.mutate(notification.id);
    }
    if (notification.action?.url) {
      window.location.href = notification.action.url;
    }
  };

  const handleMarkAllAsRead = () => {
    markAllAsReadMutation.mutate();
  };

  const handleDismissNotification = (notificationId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    dismissMutation.mutate(notificationId);
  };

  // Get notification icon
  const getNotificationIcon = (type: NotificationType['type']) => {
    switch (type) {
      case 'success':
        return CheckCircle2;
      case 'warning':
        return AlertTriangle;
      case 'error':
        return AlertTriangle;
      case 'info':
      default:
        return Info;
    }
  };

  // Get notification color
  const getNotificationColor = (type: NotificationType['type']) => {
    switch (type) {
      case 'success':
        return 'text-green-600';
      case 'warning':
        return 'text-yellow-600';
      case 'error':
        return 'text-red-600';
      case 'info':
      default:
        return 'text-blue-600';
    }
  };

  // Format relative time
  const formatRelativeTime = (timestamp: string) => {
    const now = new Date();
    const time = new Date(timestamp);
    const diffMs = now.getTime() - time.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    return `${diffDays}d ago`;
  };

  // User menu handlers
  const handleProfileClick = () => {
    setShowUserMenu(false);
    // Navigate to profile page
    window.location.href = '/profile';
  };

  const handleSettingsClick = () => {
    setShowUserMenu(false);
    // Navigate to settings page
    window.location.href = '/settings';
  };

  const handleLogout = () => {
    setShowUserMenu(false);
    // Implement logout logic
    if (confirm('Are you sure you want to logout?')) {
      // Clear any stored tokens/session data
      localStorage.clear();
      sessionStorage.clear();
      // Redirect to login or home
      window.location.href = '/';
    }
  };

  return (
    <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="flex h-16 items-center px-6">
        {/* Logo and Title */}
        <div className="flex items-center space-x-3">
          <div className="h-8 w-8 rounded-lg bg-gradient-to-r from-primary-500 to-primary-600 flex items-center justify-center">
            <span className="text-white font-bold text-sm">I</span>
          </div>
          <div>
            <h1 className="text-lg font-semibold text-foreground">Inferno Dashboard</h1>
            <p className="text-xs text-muted-foreground">Enterprise AI/ML Platform</p>
          </div>
        </div>

        {/* Search Bar */}
        <div className="flex-1 max-w-md mx-8" ref={searchContainerRef}>
          <div className="relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <input
              ref={searchInputRef}
              type="text"
              value={searchQuery}
              onChange={handleSearchChange}
              onKeyDown={handleKeyDown}
              onFocus={() => searchQuery.length > 0 && setShowResults(true)}
              placeholder="Search models, jobs, or settings..."
              className="w-full pl-10 pr-4 py-2 text-sm border rounded-md bg-background focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
            />

            {/* Search Results Dropdown */}
            {showResults && (
              <div className="absolute top-full left-0 right-0 mt-1 bg-background border border-border rounded-md shadow-lg max-h-96 overflow-y-auto z-50">
                {searchLoading ? (
                  <div className="p-4">
                    <div className="flex items-center space-x-2">
                      <div className="animate-spin h-4 w-4 border-2 border-primary border-t-transparent rounded-full"></div>
                      <span className="text-sm text-muted-foreground">Searching...</span>
                    </div>
                  </div>
                ) : searchResults.length > 0 ? (
                  <div className="py-2">
                    <div className="px-3 py-1 text-xs font-semibold text-muted-foreground uppercase tracking-wider border-b">
                      {searchResults.length} result{searchResults.length !== 1 ? 's' : ''}
                    </div>
                    {searchResults.map((result, index) => {
                      const Icon = getResultIcon(result.type);
                      return (
                        <button
                          key={result.id}
                          onClick={() => handleResultSelect(result)}
                          className={`w-full px-3 py-2 text-left hover:bg-muted focus:bg-muted focus:outline-none flex items-center space-x-3 ${
                            index === selectedIndex ? 'bg-muted' : ''
                          }`}
                        >
                          <Icon className="h-4 w-4 text-muted-foreground flex-shrink-0" />
                          <div className="flex-1 min-w-0">
                            <div className="text-sm font-medium text-foreground truncate">
                              {result.title}
                            </div>
                            {result.description && (
                              <div className="text-xs text-muted-foreground truncate">
                                {result.description}
                              </div>
                            )}
                          </div>
                          <Badge variant="secondary" className="text-xs">
                            {result.type}
                          </Badge>
                          <ChevronRight className="h-3 w-3 text-muted-foreground" />
                        </button>
                      );
                    })}
                  </div>
                ) : searchQuery.length > 0 ? (
                  <div className="p-4 text-center">
                    <div className="text-sm text-muted-foreground">
                      No results found for "{searchQuery}"
                    </div>
                    <div className="text-xs text-muted-foreground mt-1">
                      Try searching for models, batch jobs, or settings
                    </div>
                  </div>
                ) : null}
              </div>
            )}
          </div>
        </div>

        {/* Status Indicators */}
        <div className="flex items-center space-x-2 mr-4">
          <Badge variant="success" className="text-xs">
            System Online
          </Badge>
          {loadedModelsLoading ? (
            <Skeleton className="h-5 w-20" />
          ) : (
            <Badge variant="secondary" className="text-xs">
              {loadedModels?.length || 0} Models Active
            </Badge>
          )}
        </div>

        {/* Real-time Status */}
        <div className="mr-4">
          <RealTimeStatus />
        </div>

        {/* Action Buttons */}
        <div className="flex items-center space-x-2">
          {/* Notifications */}
          <div className="relative" ref={notificationRef}>
            <Button
              variant="ghost"
              size="icon"
              className="relative"
              onClick={() => setShowNotifications(!showNotifications)}
            >
              <Bell className="h-4 w-4" />
              {notificationsLoading ? (
                <Skeleton className="absolute -top-1 -right-1 h-3 w-3 rounded-full" />
              ) : unreadCount && unreadCount > 0 ? (
                <span className="absolute -top-1 -right-1 h-3 w-3 bg-red-500 rounded-full text-xs text-white flex items-center justify-center">
                  {unreadCount > 9 ? '9+' : unreadCount}
                </span>
              ) : null}
            </Button>

            {/* Notifications Dropdown */}
            {showNotifications && (
              <div className="absolute top-full right-0 mt-1 w-80 bg-background border border-border rounded-md shadow-lg max-h-96 overflow-hidden z-50">
                {/* Header */}
                <div className="flex items-center justify-between p-4 border-b">
                  <h3 className="text-sm font-semibold">Notifications</h3>
                  <div className="flex items-center space-x-2">
                    {unreadCount && unreadCount > 0 && (
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={handleMarkAllAsRead}
                        className="text-xs"
                      >
                        <Check className="h-3 w-3 mr-1" />
                        Mark all read
                      </Button>
                    )}
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => setShowNotifications(false)}
                      className="h-6 w-6"
                    >
                      <X className="h-3 w-3" />
                    </Button>
                  </div>
                </div>

                {/* Notifications List */}
                <div className="max-h-80 overflow-y-auto">
                  {notificationsDataLoading ? (
                    <div className="p-4">
                      <div className="space-y-3">
                        {[...Array(3)].map((_, i) => (
                          <div key={i} className="flex items-start space-x-3">
                            <Skeleton className="h-8 w-8 rounded-full" />
                            <div className="flex-1">
                              <Skeleton className="h-4 w-3/4 mb-1" />
                              <Skeleton className="h-3 w-1/2" />
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  ) : notifications.length > 0 ? (
                    <div className="divide-y">
                      {notifications.slice(0, 10).map((notification) => {
                        const Icon = getNotificationIcon(notification.type);
                        const iconColor = getNotificationColor(notification.type);
                        return (
                          <button
                            key={notification.id}
                            onClick={() => handleNotificationClick(notification)}
                            className={`w-full p-4 text-left hover:bg-muted focus:bg-muted focus:outline-none flex items-start space-x-3 ${
                              !notification.read ? 'bg-primary/5' : ''
                            }`}
                          >
                            <div className={`mt-0.5 ${iconColor}`}>
                              <Icon className="h-4 w-4" />
                            </div>
                            <div className="flex-1 min-w-0">
                              <div className="flex items-start justify-between">
                                <div className="flex-1">
                                  <p className={`text-sm ${
                                    !notification.read ? 'font-semibold' : 'font-medium'
                                  } text-foreground truncate`}>
                                    {notification.title}
                                  </p>
                                  <p className="text-xs text-muted-foreground mt-1 line-clamp-2">
                                    {notification.message}
                                  </p>
                                  <div className="flex items-center justify-between mt-2">
                                    <span className="text-xs text-muted-foreground">
                                      {formatRelativeTime(notification.timestamp)}
                                    </span>
                                    <div className="flex items-center space-x-1">
                                      <Badge variant="outline" className="text-xs">
                                        {notification.source}
                                      </Badge>
                                      {notification.priority === 'high' || notification.priority === 'critical' ? (
                                        <Badge variant="destructive" className="text-xs">
                                          {notification.priority}
                                        </Badge>
                                      ) : null}
                                    </div>
                                  </div>
                                </div>
                                <Button
                                  variant="ghost"
                                  size="icon"
                                  onClick={(e) => handleDismissNotification(notification.id, e)}
                                  className="h-6 w-6 ml-2 opacity-0 group-hover:opacity-100"
                                >
                                  <X className="h-3 w-3" />
                                </Button>
                              </div>
                            </div>
                          </button>
                        );
                      })}
                    </div>
                  ) : (
                    <div className="p-8 text-center">
                      <Bell className="h-8 w-8 mx-auto text-muted-foreground mb-2" />
                      <p className="text-sm text-muted-foreground">No notifications</p>
                      <p className="text-xs text-muted-foreground mt-1">
                        You're all caught up!
                      </p>
                    </div>
                  )}
                </div>

                {/* Footer */}
                {notifications.length > 10 && (
                  <div className="p-3 border-t text-center">
                    <Button variant="ghost" size="sm" className="text-xs">
                      View all notifications
                    </Button>
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Theme Toggle */}
          <ThemeToggle />

          {/* Settings */}
          <Button variant="ghost" size="icon">
            <Settings className="h-4 w-4" />
          </Button>

          {/* User Menu */}
          <div className="relative" ref={userMenuRef}>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setShowUserMenu(!showUserMenu)}
            >
              <User className="h-4 w-4" />
            </Button>

            {/* User Menu Dropdown */}
            {showUserMenu && (
              <div className="absolute top-full right-0 mt-1 w-56 bg-background border border-border rounded-md shadow-lg z-50">
                {/* User Info */}
                <div className="px-4 py-3 border-b">
                  <div className="flex items-center space-x-3">
                    <div className="h-8 w-8 rounded-full bg-gradient-to-r from-primary-500 to-primary-600 flex items-center justify-center">
                      <span className="text-white font-semibold text-sm">U</span>
                    </div>
                    <div>
                      <p className="text-sm font-medium">Admin User</p>
                      <p className="text-xs text-muted-foreground">admin@inferno.ai</p>
                    </div>
                  </div>
                </div>

                {/* Menu Items */}
                <div className="py-2">
                  <button
                    onClick={handleProfileClick}
                    className="w-full px-4 py-2 text-left text-sm hover:bg-muted focus:bg-muted focus:outline-none flex items-center space-x-3"
                  >
                    <UserCircle className="h-4 w-4 text-muted-foreground" />
                    <span>Profile</span>
                  </button>

                  <button
                    onClick={handleSettingsClick}
                    className="w-full px-4 py-2 text-left text-sm hover:bg-muted focus:bg-muted focus:outline-none flex items-center space-x-3"
                  >
                    <Cog className="h-4 w-4 text-muted-foreground" />
                    <span>Settings</span>
                  </button>

                  <div className="border-t my-2"></div>

                  <button
                    onClick={handleLogout}
                    className="w-full px-4 py-2 text-left text-sm hover:bg-muted focus:bg-muted focus:outline-none flex items-center space-x-3 text-red-600"
                  >
                    <LogOut className="h-4 w-4" />
                    <span>Logout</span>
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </header>
  );
}