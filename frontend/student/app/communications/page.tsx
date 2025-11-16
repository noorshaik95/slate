'use client';

import { useState } from 'react';
import { Card } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import {
  Send,
  Search,
  Filter,
  Mail,
  MessageSquare,
  Bell,
  Users,
  Star,
  Archive,
  Trash2,
  Plus,
  Paperclip,
  Eye
} from 'lucide-react';

// Mock messages data
const mockMessages = Array.from({ length: 20 }, (_, i) => ({
  id: `msg-${i}`,
  subject: [
    'Assignment Due Date Extension',
    'Course Materials Updated',
    'Grade Posted for Midterm',
    'Office Hours Change',
    'Important Announcement',
    'Group Project Update',
    'Exam Schedule Released',
    'New Course Resources Available'
  ][i % 8],
  from: [
    'Dr. Sarah Johnson',
    'Prof. Michael Chen',
    'Dr. Emily Rodriguez',
    'Prof. David Kim',
    'Admin Office'
  ][i % 5],
  preview: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore...',
  timestamp: new Date(Date.now() - Math.random() * 7 * 24 * 60 * 60 * 1000).toISOString(),
  isRead: Math.random() > 0.3,
  isStarred: Math.random() > 0.7,
  hasAttachment: Math.random() > 0.6,
  category: ['inbox', 'sent', 'archived'][Math.floor(Math.random() * 3)] as 'inbox' | 'sent' | 'archived',
  recipients: Math.floor(Math.random() * 50) + 1,
}));

// Mock announcements
const mockAnnouncements = Array.from({ length: 5 }, (_, i) => ({
  id: `announcement-${i}`,
  title: [
    'Campus Closure - Holiday Break',
    'Final Exam Schedule Posted',
    'New Library Hours',
    'Registration Opens Next Week',
    'Academic Calendar Update'
  ][i],
  content: 'This is an important announcement regarding campus operations and academic activities...',
  postedBy: 'Admin Office',
  postedAt: new Date(Date.now() - i * 24 * 60 * 60 * 1000).toISOString(),
  priority: ['high', 'medium', 'low'][Math.floor(Math.random() * 3)] as 'high' | 'medium' | 'low',
  views: Math.floor(Math.random() * 500) + 100,
}));

export default function CommunicationsPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [activeTab, setActiveTab] = useState<'messages' | 'announcements' | 'compose'>('messages');
  const [filterCategory, setFilterCategory] = useState<'all' | 'inbox' | 'sent' | 'archived'>('all');
  const [selectedMessage, setSelectedMessage] = useState<string | null>(null);

  // Filter messages
  const filteredMessages = mockMessages.filter((message) => {
    const matchesSearch =
      message.subject.toLowerCase().includes(searchQuery.toLowerCase()) ||
      message.from.toLowerCase().includes(searchQuery.toLowerCase()) ||
      message.preview.toLowerCase().includes(searchQuery.toLowerCase());

    const matchesFilter =
      filterCategory === 'all' || message.category === filterCategory;

    return matchesSearch && matchesFilter;
  });

  const unreadCount = mockMessages.filter(m => !m.isRead && m.category === 'inbox').length;
  const starredCount = mockMessages.filter(m => m.isStarred).length;

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-cyan-50 to-blue-50 dark:from-gray-900 dark:via-gray-900 dark:to-gray-900 p-6">
      <div className="max-w-7xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div>
            <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
              Communications Center
            </h1>
            <p className="text-gray-600 dark:text-gray-400 mt-1">
              Messages, announcements, and notifications
            </p>
          </div>
          <Button
            onClick={() => setActiveTab('compose')}
            className="bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-700 hover:to-blue-700 text-white shadow-lg hover:shadow-xl transition-all"
          >
            <Plus className="h-4 w-4 mr-2" />
            New Message
          </Button>
        </div>

        {/* Stats Cards */}
        <div className="grid gap-4 md:grid-cols-4">
          <Card className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl hover:shadow-lg transition-shadow">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400">Unread</p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white mt-1">
                  {unreadCount}
                </p>
              </div>
              <Mail className="h-10 w-10 text-cyan-500" />
            </div>
          </Card>

          <Card className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl hover:shadow-lg transition-shadow">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400">Starred</p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white mt-1">
                  {starredCount}
                </p>
              </div>
              <Star className="h-10 w-10 text-amber-500" />
            </div>
          </Card>

          <Card className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl hover:shadow-lg transition-shadow">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400">Messages</p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white mt-1">
                  {mockMessages.length}
                </p>
              </div>
              <MessageSquare className="h-10 w-10 text-indigo-500" />
            </div>
          </Card>

          <Card className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl hover:shadow-lg transition-shadow">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400">Announcements</p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white mt-1">
                  {mockAnnouncements.length}
                </p>
              </div>
              <Bell className="h-10 w-10 text-purple-500" />
            </div>
          </Card>
        </div>

        {/* Tabs */}
        <div className="flex gap-2 border-b border-gray-200 dark:border-gray-700">
          <Button
            variant="ghost"
            onClick={() => setActiveTab('messages')}
            className={`rounded-b-none border-b-2 ${
              activeTab === 'messages'
                ? 'border-cyan-600 text-cyan-600 dark:text-cyan-400'
                : 'border-transparent text-gray-600 dark:text-gray-400'
            }`}
          >
            <Mail className="h-4 w-4 mr-2" />
            Messages
          </Button>
          <Button
            variant="ghost"
            onClick={() => setActiveTab('announcements')}
            className={`rounded-b-none border-b-2 ${
              activeTab === 'announcements'
                ? 'border-cyan-600 text-cyan-600 dark:text-cyan-400'
                : 'border-transparent text-gray-600 dark:text-gray-400'
            }`}
          >
            <Bell className="h-4 w-4 mr-2" />
            Announcements
          </Button>
          <Button
            variant="ghost"
            onClick={() => setActiveTab('compose')}
            className={`rounded-b-none border-b-2 ${
              activeTab === 'compose'
                ? 'border-cyan-600 text-cyan-600 dark:text-cyan-400'
                : 'border-transparent text-gray-600 dark:text-gray-400'
            }`}
          >
            <Send className="h-4 w-4 mr-2" />
            Compose
          </Button>
        </div>

        {/* Messages Tab */}
        {activeTab === 'messages' && (
          <>
            {/* Search and Filters */}
            <div className="flex flex-col sm:flex-row gap-4">
              <div className="flex-1 relative">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-gray-400" />
                <Input
                  type="text"
                  placeholder="Search messages..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-10 rounded-xl border-gray-300 dark:border-gray-700 focus:border-cyan-500 focus:ring-cyan-500"
                />
              </div>
              <div className="flex gap-2">
                <Button
                  variant={filterCategory === 'all' ? 'default' : 'outline'}
                  onClick={() => setFilterCategory('all')}
                  className={filterCategory === 'all' ? 'bg-cyan-600 hover:bg-cyan-700' : ''}
                >
                  All
                </Button>
                <Button
                  variant={filterCategory === 'inbox' ? 'default' : 'outline'}
                  onClick={() => setFilterCategory('inbox')}
                  className={filterCategory === 'inbox' ? 'bg-cyan-600 hover:bg-cyan-700' : ''}
                >
                  Inbox
                </Button>
                <Button
                  variant={filterCategory === 'sent' ? 'default' : 'outline'}
                  onClick={() => setFilterCategory('sent')}
                  className={filterCategory === 'sent' ? 'bg-cyan-600 hover:bg-cyan-700' : ''}
                >
                  Sent
                </Button>
                <Button
                  variant={filterCategory === 'archived' ? 'default' : 'outline'}
                  onClick={() => setFilterCategory('archived')}
                  className={filterCategory === 'archived' ? 'bg-cyan-600 hover:bg-cyan-700' : ''}
                >
                  Archived
                </Button>
              </div>
            </div>

            {/* Messages List */}
            <Card className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl overflow-hidden">
              <div className="divide-y divide-gray-200 dark:divide-gray-700">
                {filteredMessages.map((message) => (
                  <div
                    key={message.id}
                    className={`p-4 hover:bg-gray-50 dark:hover:bg-gray-900 transition-colors cursor-pointer ${
                      !message.isRead ? 'bg-cyan-50 dark:bg-gray-900/50' : ''
                    }`}
                    onClick={() => setSelectedMessage(message.id)}
                  >
                    <div className="flex items-start gap-4">
                      <div className="flex items-center gap-2">
                        <button className="text-gray-400 hover:text-amber-500 transition-colors">
                          <Star
                            className={`h-5 w-5 ${
                              message.isStarred ? 'fill-amber-500 text-amber-500' : ''
                            }`}
                          />
                        </button>
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center justify-between gap-4 mb-1">
                          <div className="flex items-center gap-3">
                            <span className={`font-semibold text-gray-900 dark:text-white ${
                              !message.isRead ? 'font-bold' : ''
                            }`}>
                              {message.from}
                            </span>
                            {message.hasAttachment && (
                              <Paperclip className="h-4 w-4 text-gray-400" />
                            )}
                          </div>
                          <span className="text-sm text-gray-600 dark:text-gray-400 whitespace-nowrap">
                            {new Date(message.timestamp).toLocaleDateString('en-US', {
                              month: 'short',
                              day: 'numeric',
                              hour: '2-digit',
                              minute: '2-digit'
                            })}
                          </span>
                        </div>
                        <div className={`text-sm mb-1 ${
                          !message.isRead
                            ? 'font-semibold text-gray-900 dark:text-white'
                            : 'text-gray-700 dark:text-gray-300'
                        }`}>
                          {message.subject}
                        </div>
                        <div className="text-sm text-gray-600 dark:text-gray-400 truncate">
                          {message.preview}
                        </div>
                        {message.category === 'sent' && (
                          <div className="flex items-center gap-2 mt-2 text-xs text-gray-500 dark:text-gray-400">
                            <Users className="h-3 w-3" />
                            <span>{message.recipients} recipient{message.recipients > 1 ? 's' : ''}</span>
                          </div>
                        )}
                      </div>
                      <div className="flex items-center gap-1">
                        <Button variant="ghost" size="sm">
                          <Archive className="h-4 w-4" />
                        </Button>
                        <Button variant="ghost" size="sm" className="text-red-600 hover:text-red-700">
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>

              {filteredMessages.length === 0 && (
                <div className="p-12 text-center">
                  <Mail className="mx-auto h-12 w-12 text-gray-400 dark:text-gray-600" />
                  <h3 className="mt-4 text-lg font-semibold text-gray-900 dark:text-white">
                    No messages found
                  </h3>
                  <p className="mt-2 text-gray-600 dark:text-gray-400">
                    {searchQuery
                      ? 'Try adjusting your search terms'
                      : 'Your inbox is empty'}
                  </p>
                </div>
              )}
            </Card>
          </>
        )}

        {/* Announcements Tab */}
        {activeTab === 'announcements' && (
          <div className="space-y-4">
            {mockAnnouncements.map((announcement) => (
              <Card
                key={announcement.id}
                className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl hover:shadow-lg transition-shadow"
              >
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                        {announcement.title}
                      </h3>
                      <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
                        announcement.priority === 'high'
                          ? 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300'
                          : announcement.priority === 'medium'
                          ? 'bg-amber-100 text-amber-700 dark:bg-amber-900 dark:text-amber-300'
                          : 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300'
                      }`}>
                        {announcement.priority}
                      </span>
                    </div>
                    <p className="text-gray-600 dark:text-gray-400 mb-3">
                      {announcement.content}
                    </p>
                    <div className="flex items-center gap-4 text-sm text-gray-500 dark:text-gray-400">
                      <span>Posted by {announcement.postedBy}</span>
                      <span>•</span>
                      <span>
                        {new Date(announcement.postedAt).toLocaleDateString('en-US', {
                          month: 'long',
                          day: 'numeric',
                          year: 'numeric'
                        })}
                      </span>
                      <span>•</span>
                      <div className="flex items-center gap-1">
                        <Eye className="h-3 w-3" />
                        <span>{announcement.views} views</span>
                      </div>
                    </div>
                  </div>
                  <Bell className="h-8 w-8 text-purple-500" />
                </div>
              </Card>
            ))}
          </div>
        )}

        {/* Compose Tab */}
        {activeTab === 'compose' && (
          <Card className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              New Message
            </h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  To
                </label>
                <Input
                  type="text"
                  placeholder="Search students, instructors, or groups..."
                  className="rounded-xl"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Subject
                </label>
                <Input
                  type="text"
                  placeholder="Enter subject..."
                  className="rounded-xl"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Message
                </label>
                <textarea
                  rows={8}
                  placeholder="Type your message..."
                  className="w-full px-4 py-3 rounded-xl border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:border-cyan-500 focus:ring-cyan-500 resize-none"
                />
              </div>
              <div className="flex items-center justify-between">
                <Button variant="outline">
                  <Paperclip className="h-4 w-4 mr-2" />
                  Attach File
                </Button>
                <div className="flex gap-2">
                  <Button variant="outline">
                    Save Draft
                  </Button>
                  <Button className="bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-700 hover:to-blue-700 text-white">
                    <Send className="h-4 w-4 mr-2" />
                    Send Message
                  </Button>
                </div>
              </div>
            </div>
          </Card>
        )}
      </div>
    </div>
  );
}
