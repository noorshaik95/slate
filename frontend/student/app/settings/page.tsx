'use client';

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Bell, Lock, Globe, Palette, Moon, Sun, Monitor } from 'lucide-react';
import { useTheme } from 'next-themes';

export default function SettingsPage() {
  const { theme, setTheme } = useTheme();
  const [notifications, setNotifications] = useState({
    email: true,
    push: true,
    assignments: true,
    grades: true,
    announcements: true,
    discussions: false,
  });

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 p-6">
      <div className="space-y-6 max-w-4xl mx-auto">
        {/* Header */}
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Settings</h1>
          <p className="text-slate-600 mt-1">Manage your account settings and preferences</p>
        </div>

        {/* Appearance Section */}
        <div className="bg-white border border-slate-200 rounded-2xl p-6 space-y-4">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center">
              <Palette className="h-5 w-5 text-white" />
            </div>
            <div>
              <h2 className="text-lg font-semibold">Appearance</h2>
              <p className="text-sm text-slate-600">Customize how the portal looks on your device</p>
            </div>
          </div>
          
          <div className="space-y-2 pt-2">
            <label className="text-sm font-medium text-slate-700">Theme</label>
            <div className="grid grid-cols-3 gap-3">
              <button
                onClick={() => setTheme('light')}
                className={`flex flex-col items-center gap-2 p-4 border-2 rounded-xl transition-all duration-300 ${
                  theme === 'light' 
                    ? 'border-indigo-600 bg-indigo-50' 
                    : 'border-slate-200 hover:border-indigo-300 hover:bg-slate-50'
                }`}
              >
                <Sun className="h-6 w-6 text-slate-700" />
                <span className="text-sm font-medium">Light</span>
              </button>
              <button
                onClick={() => setTheme('dark')}
                className={`flex flex-col items-center gap-2 p-4 border-2 rounded-xl transition-all duration-300 ${
                  theme === 'dark' 
                    ? 'border-indigo-600 bg-indigo-50' 
                    : 'border-slate-200 hover:border-indigo-300 hover:bg-slate-50'
                }`}
              >
                <Moon className="h-6 w-6 text-slate-700" />
                <span className="text-sm font-medium">Dark</span>
              </button>
              <button
                onClick={() => setTheme('system')}
                className={`flex flex-col items-center gap-2 p-4 border-2 rounded-xl transition-all duration-300 ${
                  theme === 'system' 
                    ? 'border-indigo-600 bg-indigo-50' 
                    : 'border-slate-200 hover:border-indigo-300 hover:bg-slate-50'
                }`}
              >
                <Monitor className="h-6 w-6 text-slate-700" />
                <span className="text-sm font-medium">System</span>
              </button>
            </div>
          </div>
        </div>

        {/* Notifications Section */}
        <div className="bg-white border border-slate-200 rounded-2xl p-6 space-y-4">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-500 to-teal-600 flex items-center justify-center">
              <Bell className="h-5 w-5 text-white" />
            </div>
            <div>
              <h2 className="text-lg font-semibold">Notifications</h2>
              <p className="text-sm text-slate-600">Choose what notifications you want to receive</p>
            </div>
          </div>

          {/* Notification Channels */}
          <div className="space-y-3 pt-2">
            <label className="text-sm font-medium text-slate-700">Notification Channels</label>
            <div className="space-y-2">
              <div className="flex items-center justify-between p-4 border border-slate-200 rounded-xl">
                <div>
                  <p className="font-medium text-slate-900">Email Notifications</p>
                  <p className="text-sm text-slate-600">Receive updates via email</p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={notifications.email}
                    onChange={(e) => setNotifications({ ...notifications, email: e.target.checked })}
                    className="sr-only peer"
                  />
                  <div className="w-11 h-6 bg-slate-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-indigo-200 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-600"></div>
                </label>
              </div>
              <div className="flex items-center justify-between p-4 border border-slate-200 rounded-xl">
                <div>
                  <p className="font-medium text-slate-900">Push Notifications</p>
                  <p className="text-sm text-slate-600">Receive instant alerts</p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={notifications.push}
                    onChange={(e) => setNotifications({ ...notifications, push: e.target.checked })}
                    className="sr-only peer"
                  />
                  <div className="w-11 h-6 bg-slate-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-indigo-200 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-600"></div>
                </label>
              </div>
            </div>
          </div>

          {/* Notification Types */}
          <div className="space-y-3 pt-2">
            <label className="text-sm font-medium text-slate-700">Notification Types</label>
            <div className="space-y-2">
              {[
                { key: 'assignments' as const, label: 'Assignments', desc: 'New assignments and due dates' },
                { key: 'grades' as const, label: 'Grades', desc: 'When your work is graded' },
                { key: 'announcements' as const, label: 'Announcements', desc: 'Course and system announcements' },
                { key: 'discussions' as const, label: 'Discussions', desc: 'Replies to your posts' },
              ].map(({ key, label, desc }) => (
                <div key={key} className="flex items-center justify-between p-4 border border-slate-200 rounded-xl">
                  <div>
                    <p className="font-medium text-slate-900">{label}</p>
                    <p className="text-sm text-slate-600">{desc}</p>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input
                      type="checkbox"
                      checked={notifications[key]}
                      onChange={(e) => setNotifications({ ...notifications, [key]: e.target.checked })}
                      className="sr-only peer"
                    />
                    <div className="w-11 h-6 bg-slate-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-indigo-200 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-600"></div>
                  </label>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Privacy */}
        <div className="bg-white border border-slate-200 rounded-2xl p-6 space-y-4">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-blue-500 to-cyan-600 flex items-center justify-center">
              <Globe className="h-5 w-5 text-white" />
            </div>
            <div>
              <h2 className="text-lg font-semibold">Privacy</h2>
              <p className="text-sm text-slate-600">Control who can see your information</p>
            </div>
          </div>
          
          <div className="space-y-2 pt-2">
            <div className="flex items-center justify-between p-4 border border-slate-200 rounded-xl">
              <div>
                <p className="font-medium text-slate-900">Show Profile</p>
                <p className="text-sm text-slate-600">Allow others to view your profile</p>
              </div>
              <Badge className="bg-emerald-100 text-emerald-700 hover:bg-emerald-100">Public</Badge>
            </div>
            <div className="flex items-center justify-between p-4 border border-slate-200 rounded-xl">
              <div>
                <p className="font-medium text-slate-900">Show Grades</p>
                <p className="text-sm text-slate-600">Display grades on leaderboards</p>
              </div>
              <Badge className="bg-slate-100 text-slate-700 hover:bg-slate-100">Private</Badge>
            </div>
          </div>
        </div>

        {/* Security */}
        <div className="bg-white border border-slate-200 rounded-2xl p-6 space-y-4">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-purple-500 to-pink-600 flex items-center justify-center">
              <Lock className="h-5 w-5 text-white" />
            </div>
            <div>
              <h2 className="text-lg font-semibold">Security</h2>
              <p className="text-sm text-slate-600">Manage your account security</p>
            </div>
          </div>
          
          <div className="space-y-2 pt-2">
            <Button variant="outline" className="w-full rounded-xl border-slate-200 hover:bg-slate-50">
              Change Password
            </Button>
            <Button variant="outline" className="w-full rounded-xl border-slate-200 hover:bg-slate-50">
              Enable Two-Factor Authentication
            </Button>
            <Button variant="outline" className="w-full rounded-xl border-slate-200 hover:bg-slate-50">
              Download Account Data
            </Button>
          </div>
        </div>

        {/* Save Button */}
        <div className="flex justify-end gap-3">
          <Button variant="outline" className="rounded-xl border-slate-200 hover:bg-slate-50">
            Reset to Defaults
          </Button>
          <Button className="rounded-xl bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-700 hover:to-purple-700">
            Save Changes
          </Button>
        </div>
      </div>
    </div>
  );
}
