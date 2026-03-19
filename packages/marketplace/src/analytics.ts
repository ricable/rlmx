/**
 * MarketplaceAnalytics: install tracking, revenue metrics, retention.
 * Mirrors rlmx-marketplace/src/analytics.rs.
 */

import type { LifeDomain } from '@aix/shared';
import type {
  AgentMetrics,
  DomainMetrics,
  InstallEvent,
  UninstallEvent,
} from './types.js';

/** Compute retention rate as a percentage. */
export function retentionRate(metrics: AgentMetrics): number {
  if (metrics.totalInstalls === 0) {
    return 0;
  }
  const active = Math.max(
    0,
    metrics.totalInstalls - metrics.totalUninstalls,
  );
  return (active / metrics.totalInstalls) * 100;
}

// ---------------------------------------------------------------------------
// MarketplaceAnalytics
// ---------------------------------------------------------------------------

/** Marketplace-wide analytics engine. */
export class MarketplaceAnalytics {
  private readonly agentMetricsMap = new Map<string, AgentMetrics>();
  private readonly installs: InstallEvent[] = [];
  private readonly uninstalls: UninstallEvent[] = [];

  /** Record an agent installation. */
  recordInstall(agentId: string, userId: string, domain: LifeDomain): void {
    const event: InstallEvent = {
      agentId,
      userId,
      domain,
      timestamp: new Date().toISOString(),
    };
    this.installs.push(event);

    const metrics = this.getOrCreateMetrics(agentId);
    metrics.totalInstalls += 1;
  }

  /** Record an agent uninstall. */
  recordUninstall(agentId: string, userId: string): void {
    const event: UninstallEvent = {
      agentId,
      userId,
      timestamp: new Date().toISOString(),
    };
    this.uninstalls.push(event);

    const metrics = this.getOrCreateMetrics(agentId);
    metrics.totalUninstalls += 1;
  }

  /** Record revenue for an agent. */
  recordRevenue(agentId: string, amountCents: number): void {
    const metrics = this.getOrCreateMetrics(agentId);
    metrics.revenueCents += amountCents;
  }

  /** Get metrics for a specific agent. */
  agentMetrics(agentId: string): AgentMetrics | undefined {
    return this.agentMetricsMap.get(agentId);
  }

  /** Compute per-domain metrics from install history. */
  domainMetrics(): Map<LifeDomain, DomainMetrics> {
    const metrics = new Map<LifeDomain, DomainMetrics>();
    for (const event of this.installs) {
      let dm = metrics.get(event.domain);
      if (!dm) {
        dm = { totalInstalls: 0, totalRevenueCents: 0, agentCount: 0 };
        metrics.set(event.domain, dm);
      }
      dm.totalInstalls += 1;
    }
    return metrics;
  }

  /** Total installs across all agents. */
  totalInstalls(): number {
    return this.installs.length;
  }

  /** Total revenue across all agents. */
  totalRevenueCents(): number {
    let sum = 0;
    for (const m of this.agentMetricsMap.values()) {
      sum += m.revenueCents;
    }
    return sum;
  }

  /** Top agents by install count. */
  topByInstalls(n: number): Array<{ agentId: string; installs: number }> {
    const agents = Array.from(this.agentMetricsMap.entries()).map(
      ([id, m]) => ({ agentId: id, installs: m.totalInstalls }),
    );
    agents.sort((a, b) => b.installs - a.installs);
    return agents.slice(0, n);
  }

  /** Top agents by revenue. */
  topByRevenue(n: number): Array<{ agentId: string; revenue: number }> {
    const agents = Array.from(this.agentMetricsMap.entries()).map(
      ([id, m]) => ({ agentId: id, revenue: m.revenueCents }),
    );
    agents.sort((a, b) => b.revenue - a.revenue);
    return agents.slice(0, n);
  }

  // ---------------------------------------------------------------------------
  // Private
  // ---------------------------------------------------------------------------

  private getOrCreateMetrics(agentId: string): AgentMetrics {
    let metrics = this.agentMetricsMap.get(agentId);
    if (!metrics) {
      metrics = { totalInstalls: 0, totalUninstalls: 0, revenueCents: 0 };
      this.agentMetricsMap.set(agentId, metrics);
    }
    return metrics;
  }
}
