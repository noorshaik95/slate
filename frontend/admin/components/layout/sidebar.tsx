'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { cn } from '@/lib/utils';
import {
  LayoutDashboard,
  Users,
  Settings,
  CreditCard,
  Shield,
  BarChart3,
  Upload,
  LogOut,
  Menu,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { authService } from '@/lib/api/auth';
import { useRouter } from 'next/navigation';
import { useState } from 'react';

const navigation = [
  {
    name: 'Dashboard',
    href: '/dashboard',
    icon: LayoutDashboard,
  },
  {
    name: 'Onboarding',
    href: '/onboarding',
    icon: Upload,
    children: [
      { name: 'Bulk Import', href: '/onboarding/bulk-import' },
      { name: 'Integrations', href: '/onboarding/integrations' },
      { name: 'Jobs', href: '/onboarding/jobs' },
    ],
  },
  {
    name: 'Admin Service',
    href: '/admin-service',
    icon: Settings,
    children: [
      { name: 'Usage Analytics', href: '/admin-service/analytics' },
      { name: 'Billing', href: '/admin-service/billing' },
      { name: 'Plans & Upgrades', href: '/admin-service/plans' },
      { name: 'Cost Optimization', href: '/admin-service/optimization' },
    ],
  },
  {
    name: 'IAM Policies',
    href: '/iam',
    icon: Shield,
    children: [
      { name: 'Users', href: '/iam/users' },
      { name: 'Policies', href: '/iam/policies' },
    ],
  },
];

export function Sidebar() {
  const pathname = usePathname();
  const router = useRouter();
  const [isOpen, setIsOpen] = useState(true);

  const handleLogout = async () => {
    await authService.logout();
    router.push('/login');
  };

  return (
    <>
      <Button
        variant="ghost"
        size="icon"
        className="fixed top-4 left-4 z-50 lg:hidden"
        onClick={() => setIsOpen(!isOpen)}
      >
        <Menu className="h-6 w-6" />
      </Button>

      <aside
        className={cn(
          'fixed left-0 top-0 z-40 h-screen w-64 bg-card border-r transition-transform lg:translate-x-0',
          isOpen ? 'translate-x-0' : '-translate-x-full'
        )}
      >
        <div className="flex h-full flex-col">
          <div className="flex h-16 items-center border-b px-6">
            <Shield className="h-6 w-6 text-primary" />
            <span className="ml-2 text-lg font-semibold">Admin Portal</span>
          </div>

          <nav className="flex-1 space-y-1 overflow-y-auto p-4">
            {navigation.map((item) => (
              <div key={item.name}>
                <Link
                  href={item.href}
                  className={cn(
                    'flex items-center rounded-lg px-3 py-2 text-sm font-medium transition-colors',
                    pathname === item.href
                      ? 'bg-primary text-primary-foreground'
                      : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
                  )}
                >
                  <item.icon className="mr-3 h-5 w-5" />
                  {item.name}
                </Link>
                {item.children && (
                  <div className="ml-8 mt-1 space-y-1">
                    {item.children.map((child) => (
                      <Link
                        key={child.href}
                        href={child.href}
                        className={cn(
                          'block rounded-lg px-3 py-2 text-sm transition-colors',
                          pathname === child.href
                            ? 'bg-primary/10 text-primary font-medium'
                            : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
                        )}
                      >
                        {child.name}
                      </Link>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </nav>

          <div className="border-t p-4">
            <Button
              variant="ghost"
              className="w-full justify-start"
              onClick={handleLogout}
            >
              <LogOut className="mr-3 h-5 w-5" />
              Logout
            </Button>
          </div>
        </div>
      </aside>
    </>
  );
}
