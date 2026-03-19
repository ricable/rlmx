/**
 * NotificationService — 3-tier priority notifications with fatigue prevention.
 *
 * Works in demo mode by default, logging notifications to console.
 * When integrated with a real push-notification library (e.g.
 * @react-native-firebase/messaging or react-native-push-notification),
 * replace the `dispatchNotification` method.
 */

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type NotificationPriority = 'critical' | 'actionable' | 'informational';

export interface RuVixNotification {
  id: string;
  title: string;
  body: string;
  priority: NotificationPriority;
  timestamp: number;
  domain?: string;
  actionUrl?: string;
  read: boolean;
  responded: boolean;
}

export interface FatigueMetrics {
  totalSent: number;
  totalResponded: number;
  responseRate: number;
  suppressedCount: number;
  lastResetTimestamp: number;
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const FATIGUE_WINDOW_MS = 24 * 60 * 60 * 1000; // 24 hours
const FATIGUE_THRESHOLD = 0.3; // suppress if response rate drops below 30%
const MIN_NOTIFICATIONS_BEFORE_FATIGUE = 5; // need at least 5 before checking

// ---------------------------------------------------------------------------
// NotificationService
// ---------------------------------------------------------------------------

export class NotificationService {
  private notifications: RuVixNotification[] = [];
  private fatigueMetrics: FatigueMetrics = {
    totalSent: 0,
    totalResponded: 0,
    responseRate: 1.0,
    suppressedCount: 0,
    lastResetTimestamp: Date.now(),
  };
  private listeners: Array<(n: RuVixNotification) => void> = [];

  // -----------------------------------------------------------------------
  // Subscription
  // -----------------------------------------------------------------------

  onNotification(cb: (n: RuVixNotification) => void): () => void {
    this.listeners.push(cb);
    return () => {
      this.listeners = this.listeners.filter((l) => l !== cb);
    };
  }

  // -----------------------------------------------------------------------
  // Fatigue prevention
  // -----------------------------------------------------------------------

  private shouldSuppress(priority: NotificationPriority): boolean {
    // Never suppress critical notifications
    if (priority === 'critical') return false;

    // Reset window if stale
    if (Date.now() - this.fatigueMetrics.lastResetTimestamp > FATIGUE_WINDOW_MS) {
      this.fatigueMetrics = {
        totalSent: 0,
        totalResponded: 0,
        responseRate: 1.0,
        suppressedCount: 0,
        lastResetTimestamp: Date.now(),
      };
      return false;
    }

    if (this.fatigueMetrics.totalSent < MIN_NOTIFICATIONS_BEFORE_FATIGUE) {
      return false;
    }

    return this.fatigueMetrics.responseRate < FATIGUE_THRESHOLD;
  }

  markAsResponded(notificationId: string): void {
    const n = this.notifications.find((x) => x.id === notificationId);
    if (n && !n.responded) {
      n.responded = true;
      this.fatigueMetrics.totalResponded += 1;
      this.fatigueMetrics.responseRate =
        this.fatigueMetrics.totalResponded / this.fatigueMetrics.totalSent;
    }
  }

  markAsRead(notificationId: string): void {
    const n = this.notifications.find((x) => x.id === notificationId);
    if (n) n.read = true;
  }

  getFatigueMetrics(): FatigueMetrics {
    return { ...this.fatigueMetrics };
  }

  // -----------------------------------------------------------------------
  // Core dispatch
  // -----------------------------------------------------------------------

  private send(
    title: string,
    body: string,
    priority: NotificationPriority,
    domain?: string,
    actionUrl?: string,
  ): RuVixNotification | null {
    if (this.shouldSuppress(priority)) {
      this.fatigueMetrics.suppressedCount += 1;
      return null;
    }

    const notification: RuVixNotification = {
      id: generateId(),
      title,
      body,
      priority,
      timestamp: Date.now(),
      domain,
      actionUrl,
      read: false,
      responded: false,
    };

    this.notifications.push(notification);
    this.fatigueMetrics.totalSent += 1;
    this.fatigueMetrics.responseRate =
      this.fatigueMetrics.totalSent > 0
        ? this.fatigueMetrics.totalResponded / this.fatigueMetrics.totalSent
        : 1.0;

    // Notify listeners
    for (const cb of this.listeners) {
      try {
        cb(notification);
      } catch {
        // Listener errors should not break the service
      }
    }

    // In a real app, this would call the native push notification API
    this.dispatchNotification(notification);

    return notification;
  }

  /**
   * Override this method with real push notification logic.
   * Default implementation logs to console (demo mode).
   */
  protected dispatchNotification(n: RuVixNotification): void {
    const icon =
      n.priority === 'critical'
        ? '[!!!]'
        : n.priority === 'actionable'
          ? '[!]'
          : '[i]';
    // Using console.log here is intentional for demo mode; in library code
    // this would be replaced by the native notification API.
    // eslint-disable-next-line no-console
    console.log(`${icon} ${n.title}: ${n.body}`);
  }

  // -----------------------------------------------------------------------
  // Convenience methods
  // -----------------------------------------------------------------------

  scheduleDailyBriefing(): RuVixNotification | null {
    const hour = new Date().getHours();
    const greeting =
      hour < 12 ? 'Good morning' : hour < 17 ? 'Good afternoon' : 'Good evening';

    const unreadCount = this.notifications.filter((n) => !n.read).length;
    const body =
      unreadCount > 0
        ? `You have ${unreadCount} unread notification${unreadCount > 1 ? 's' : ''}. Your agents are running smoothly.`
        : 'All clear! Your agents are running smoothly.';

    return this.send(
      `${greeting}! Daily Briefing`,
      body,
      'informational',
      'briefing',
    );
  }

  notifySavingsFound(amount: number, source: string): RuVixNotification | null {
    return this.send(
      'Savings Found!',
      `Your Finance agent found $${amount.toFixed(2)} in potential savings from ${source}.`,
      'actionable',
      'Finance',
      'ruvix://savings',
    );
  }

  notifyAgentProgress(
    domain: string,
    status: 'started' | 'completed' | 'error',
  ): RuVixNotification | null {
    const priority: NotificationPriority =
      status === 'error' ? 'critical' : 'informational';

    const statusText =
      status === 'started'
        ? 'has started working'
        : status === 'completed'
          ? 'has completed its task'
          : 'encountered an error';

    return this.send(
      `${domain} Agent Update`,
      `Your ${domain} agent ${statusText}.`,
      priority,
      domain,
    );
  }

  sendCritical(title: string, body: string, domain?: string): RuVixNotification | null {
    return this.send(title, body, 'critical', domain);
  }

  sendActionable(
    title: string,
    body: string,
    domain?: string,
    actionUrl?: string,
  ): RuVixNotification | null {
    return this.send(title, body, 'actionable', domain, actionUrl);
  }

  sendInformational(title: string, body: string, domain?: string): RuVixNotification | null {
    return this.send(title, body, 'informational', domain);
  }

  // -----------------------------------------------------------------------
  // Queries
  // -----------------------------------------------------------------------

  getAll(): RuVixNotification[] {
    return [...this.notifications];
  }

  getUnread(): RuVixNotification[] {
    return this.notifications.filter((n) => !n.read);
  }

  getByPriority(priority: NotificationPriority): RuVixNotification[] {
    return this.notifications.filter((n) => n.priority === priority);
  }

  clear(): void {
    this.notifications = [];
  }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function generateId(): string {
  return `notif-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

// Singleton
let _instance: NotificationService | null = null;

export function getNotificationService(): NotificationService {
  if (!_instance) {
    _instance = new NotificationService();
  }
  return _instance;
}
