'use client';

import { DashboardLayout } from '@/components/layout/dashboard-layout';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Users, HardDrive, Activity, DollarSign, TrendingUp, TrendingDown } from 'lucide-react';
import { useEffect, useState } from 'react';
import { adminService, UsageStats } from '@/lib/api/admin';
import { formatBytes, formatNumber, formatCurrency } from '@/lib/utils';

export default function DashboardPage() {
  const [stats, setStats] = useState<UsageStats | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchStats = async () => {
      try {
        const data = await adminService.getUsageStats();
        setStats(data);
      } catch (error) {
        console.error('Failed to fetch stats:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchStats();
  }, []);

  const statCards = [
    {
      title: 'Total Users',
      value: stats ? formatNumber(stats.total_users) : '-',
      description: `${stats?.active_users || 0} active`,
      icon: Users,
      trend: 'up',
      trendValue: '+12%',
    },
    {
      title: 'Storage Used',
      value: stats ? formatBytes(stats.total_storage_used) : '-',
      description: `of ${stats ? formatBytes(stats.total_storage_limit) : '-'}`,
      icon: HardDrive,
      trend: 'up',
      trendValue: '+5%',
    },
    {
      title: 'Bandwidth',
      value: stats ? formatBytes(stats.bandwidth_used) : '-',
      description: `of ${stats ? formatBytes(stats.bandwidth_limit) : '-'}`,
      icon: Activity,
      trend: 'down',
      trendValue: '-3%',
    },
    {
      title: 'AI Credits',
      value: stats ? formatNumber(stats.ai_credits_used) : '-',
      description: `of ${stats ? formatNumber(stats.ai_credits_limit) : '-'}`,
      icon: DollarSign,
      trend: 'up',
      trendValue: '+8%',
    },
  ];

  return (
    <DashboardLayout>
      <div className="space-y-6">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
          <p className="text-muted-foreground">
            Welcome to the university admin dashboard
          </p>
        </div>

        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          {statCards.map((stat) => (
            <Card key={stat.title}>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">
                  {stat.title}
                </CardTitle>
                <stat.icon className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{stat.value}</div>
                <p className="text-xs text-muted-foreground">
                  {stat.description}
                </p>
                <div className="flex items-center pt-1">
                  {stat.trend === 'up' ? (
                    <TrendingUp className="mr-1 h-3 w-3 text-green-500" />
                  ) : (
                    <TrendingDown className="mr-1 h-3 w-3 text-red-500" />
                  )}
                  <span className={`text-xs ${stat.trend === 'up' ? 'text-green-500' : 'text-red-500'}`}>
                    {stat.trendValue}
                  </span>
                  <span className="text-xs text-muted-foreground ml-1">vs last month</span>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>

        <div className="grid gap-4 md:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle>Quick Actions</CardTitle>
              <CardDescription>Common administrative tasks</CardDescription>
            </CardHeader>
            <CardContent className="space-y-2">
              <a
                href="/onboarding/bulk-import"
                className="block w-full rounded-lg border p-4 hover:bg-accent transition-colors"
              >
                <div className="font-medium">Bulk Import Users</div>
                <div className="text-sm text-muted-foreground">
                  Import users via CSV or directory sync
                </div>
              </a>
              <a
                href="/admin-service/billing"
                className="block w-full rounded-lg border p-4 hover:bg-accent transition-colors"
              >
                <div className="font-medium">Manage Billing</div>
                <div className="text-sm text-muted-foreground">
                  View invoices and payment methods
                </div>
              </a>
              <a
                href="/iam/users"
                className="block w-full rounded-lg border p-4 hover:bg-accent transition-colors"
              >
                <div className="font-medium">Manage IAM Users</div>
                <div className="text-sm text-muted-foreground">
                  Create and manage policy users
                </div>
              </a>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>System Status</CardTitle>
              <CardDescription>All systems operational</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <span className="text-sm">API Gateway</span>
                <span className="inline-flex items-center rounded-full bg-green-100 px-2.5 py-0.5 text-xs font-medium text-green-800 dark:bg-green-900/30 dark:text-green-400">
                  Operational
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm">Database</span>
                <span className="inline-flex items-center rounded-full bg-green-100 px-2.5 py-0.5 text-xs font-medium text-green-800 dark:bg-green-900/30 dark:text-green-400">
                  Operational
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm">Storage Service</span>
                <span className="inline-flex items-center rounded-full bg-green-100 px-2.5 py-0.5 text-xs font-medium text-green-800 dark:bg-green-900/30 dark:text-green-400">
                  Operational
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm">Onboarding Service</span>
                <span className="inline-flex items-center rounded-full bg-green-100 px-2.5 py-0.5 text-xs font-medium text-green-800 dark:bg-green-900/30 dark:text-green-400">
                  Operational
                </span>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </DashboardLayout>
  );
}
