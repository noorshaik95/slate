'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { User, Bell, Lock, Palette, Globe } from 'lucide-react';
import { mockInstructor } from '@/lib/mock-data';

export default function SettingsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
          Settings
        </h1>
        <p className="text-gray-600 dark:text-gray-400 mt-2">
          Manage your account and preferences
        </p>
      </div>

      <div className="grid gap-6">
        {/* Profile Settings */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <User className="h-5 w-5" />
              Profile Settings
            </CardTitle>
            <CardDescription>Manage your profile information</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-start gap-4">
              <img
                src={mockInstructor.avatar}
                alt={mockInstructor.firstName}
                className="w-20 h-20 rounded-full"
              />
              <div className="flex-1">
                <h3 className="font-semibold text-lg">
                  {mockInstructor.firstName} {mockInstructor.lastName}
                </h3>
                <p className="text-gray-600 dark:text-gray-400">{mockInstructor.email}</p>
                <p className="text-sm text-gray-500 mt-1">{mockInstructor.department}</p>
                <div className="mt-4 flex gap-2">
                  <Button size="sm">Edit Profile</Button>
                  <Button size="sm" variant="outline">
                    Change Password
                  </Button>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Notification Settings */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Bell className="h-5 w-5" />
              Notification Preferences
            </CardTitle>
            <CardDescription>Choose what notifications you receive</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-medium">Email Notifications</p>
                  <p className="text-sm text-gray-500">
                    Receive notifications via email
                  </p>
                </div>
                <Badge variant="success">Enabled</Badge>
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-medium">New Submissions</p>
                  <p className="text-sm text-gray-500">
                    Alert when students submit assignments
                  </p>
                </div>
                <Badge variant="success">Enabled</Badge>
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-medium">Student Questions</p>
                  <p className="text-sm text-gray-500">
                    Alert when students ask questions
                  </p>
                </div>
                <Badge variant="success">Enabled</Badge>
              </div>
              <Button size="sm" className="mt-4">
                Manage Notifications
              </Button>
            </div>
          </CardContent>
        </Card>

        {/* Appearance Settings */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Palette className="h-5 w-5" />
              Appearance
            </CardTitle>
            <CardDescription>Customize how the portal looks</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-medium">Theme</p>
                  <p className="text-sm text-gray-500">Select your preferred theme</p>
                </div>
                <Badge variant="default">Auto</Badge>
              </div>
              <div className="flex gap-2 mt-4">
                <Button size="sm" variant="outline">
                  Light
                </Button>
                <Button size="sm" variant="outline">
                  Dark
                </Button>
                <Button size="sm">Auto</Button>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Security Settings */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Lock className="h-5 w-5" />
              Security
            </CardTitle>
            <CardDescription>Manage your security preferences</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div>
                <p className="font-medium">Two-Factor Authentication</p>
                <p className="text-sm text-gray-500 mb-2">
                  Add an extra layer of security
                </p>
                <Button size="sm" variant="outline">
                  Enable 2FA
                </Button>
              </div>
              <div>
                <p className="font-medium">Active Sessions</p>
                <p className="text-sm text-gray-500 mb-2">Manage your active sessions</p>
                <Button size="sm" variant="outline">
                  View Sessions
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Course Settings */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Globe className="h-5 w-5" />
              Course Defaults
            </CardTitle>
            <CardDescription>Set default settings for new courses</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div>
                <p className="font-medium">Default Grading Scale</p>
                <p className="text-sm text-gray-500 mb-2">A: 90-100, B: 80-89, C: 70-79, D: 60-69, F: 0-59</p>
                <Button size="sm" variant="outline">
                  Edit Scale
                </Button>
              </div>
              <div>
                <p className="font-medium">Late Submission Policy</p>
                <p className="text-sm text-gray-500 mb-2">10% deduction per day</p>
                <Button size="sm" variant="outline">
                  Edit Policy
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
