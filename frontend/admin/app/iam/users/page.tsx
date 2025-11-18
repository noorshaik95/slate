'use client';

import { useState } from 'react';
import { DashboardLayout } from '@/components/layout/dashboard-layout';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { useToast } from '@/hooks/use-toast';
import {
  Plus,
  Search,
  Edit,
  Trash2,
  Shield,
  CheckCircle2,
  XCircle,
  Clock,
} from 'lucide-react';

const MOCK_USERS = [
  {
    id: '1',
    email: 'admin@university.edu',
    name: 'System Administrator',
    policies: ['Full Access', 'Billing Admin'],
    status: 'active',
    created_at: '2024-01-01',
    last_login: '2 hours ago',
  },
  {
    id: '2',
    email: 'onboarding@university.edu',
    name: 'Onboarding Manager',
    policies: ['Onboarding Admin'],
    status: 'active',
    created_at: '2024-01-05',
    last_login: '1 day ago',
  },
  {
    id: '3',
    email: 'billing@university.edu',
    name: 'Billing Specialist',
    policies: ['Billing Read', 'Usage Analytics'],
    status: 'active',
    created_at: '2024-01-10',
    last_login: '3 hours ago',
  },
  {
    id: '4',
    email: 'analyst@university.edu',
    name: 'Data Analyst',
    policies: ['Usage Analytics', 'Reports Read'],
    status: 'inactive',
    created_at: '2024-01-15',
    last_login: '2 weeks ago',
  },
];

export default function IAMUsersPage() {
  const [users, setUsers] = useState(MOCK_USERS);
  const [searchTerm, setSearchTerm] = useState('');
  const { toast } = useToast();

  const filteredUsers = users.filter(
    (user) =>
      user.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      user.email.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const handleDeleteUser = (id: string, name: string) => {
    setUsers(users.filter((user) => user.id !== id));
    toast({
      title: 'User Deleted',
      description: `${name} has been removed`,
    });
  };

  const handleToggleStatus = (id: string) => {
    setUsers(
      users.map((user) =>
        user.id === id
          ? { ...user, status: user.status === 'active' ? 'inactive' : 'active' }
          : user
      )
    );
    toast({
      title: 'Status Updated',
      description: 'User status has been changed',
    });
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'active':
        return (
          <Badge variant="success">
            <CheckCircle2 className="mr-1 h-3 w-3" />
            Active
          </Badge>
        );
      case 'inactive':
        return (
          <Badge variant="outline">
            <XCircle className="mr-1 h-3 w-3" />
            Inactive
          </Badge>
        );
      case 'suspended':
        return (
          <Badge variant="destructive">
            <XCircle className="mr-1 h-3 w-3" />
            Suspended
          </Badge>
        );
      default:
        return <Badge>{status}</Badge>;
    }
  };

  const activeUsers = users.filter((u) => u.status === 'active').length;
  const inactiveUsers = users.filter((u) => u.status === 'inactive').length;

  return (
    <DashboardLayout>
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold tracking-tight">IAM Users</h1>
            <p className="text-muted-foreground">
              Manage users with custom access to admin services
            </p>
          </div>
          <Button>
            <Plus className="mr-2 h-4 w-4" />
            Create User
          </Button>
        </div>

        <div className="grid gap-4 md:grid-cols-3">
          <Card>
            <CardHeader className="pb-3">
              <CardDescription>Total Users</CardDescription>
              <CardTitle className="text-3xl">{users.length}</CardTitle>
            </CardHeader>
          </Card>
          <Card>
            <CardHeader className="pb-3">
              <CardDescription>Active Users</CardDescription>
              <CardTitle className="text-3xl text-green-600">{activeUsers}</CardTitle>
            </CardHeader>
          </Card>
          <Card>
            <CardHeader className="pb-3">
              <CardDescription>Inactive Users</CardDescription>
              <CardTitle className="text-3xl text-muted-foreground">{inactiveUsers}</CardTitle>
            </CardHeader>
          </Card>
        </div>

        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle>Users</CardTitle>
                <CardDescription>Manage IAM policy users and their permissions</CardDescription>
              </div>
              <div className="relative w-64">
                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  placeholder="Search users..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-10"
                />
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>User</TableHead>
                  <TableHead>Policies</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Last Login</TableHead>
                  <TableHead>Created</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredUsers.map((user) => (
                  <TableRow key={user.id}>
                    <TableCell>
                      <div>
                        <div className="font-medium">{user.name}</div>
                        <div className="text-sm text-muted-foreground">{user.email}</div>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="flex flex-wrap gap-1">
                        {user.policies.map((policy, idx) => (
                          <Badge key={idx} variant="outline" className="text-xs">
                            <Shield className="mr-1 h-3 w-3" />
                            {policy}
                          </Badge>
                        ))}
                      </div>
                    </TableCell>
                    <TableCell>{getStatusBadge(user.status)}</TableCell>
                    <TableCell className="text-sm text-muted-foreground">
                      {user.last_login}
                    </TableCell>
                    <TableCell className="text-sm text-muted-foreground">
                      {new Date(user.created_at).toLocaleDateString()}
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex justify-end gap-2">
                        <Button variant="ghost" size="sm">
                          <Edit className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleToggleStatus(user.id)}
                        >
                          {user.status === 'active' ? (
                            <XCircle className="h-4 w-4" />
                          ) : (
                            <CheckCircle2 className="h-4 w-4" />
                          )}
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleDeleteUser(user.id, user.name)}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      </div>
    </DashboardLayout>
  );
}
