'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { MessagesSquare, Plus, Pin } from 'lucide-react';
import { mockAnnouncements, mockStudentQuestions } from '@/lib/mock-data';
import { formatRelativeTime } from '@/lib/utils';

export default function CommunicationsPage() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
            Communications
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-2">
            Announcements, messages, and discussions
          </p>
        </div>
        <Button size="lg">
          <Plus className="h-5 w-5" />
          New Announcement
        </Button>
      </div>

      <Tabs defaultValue="announcements" className="w-full">
        <TabsList>
          <TabsTrigger value="announcements">Announcements</TabsTrigger>
          <TabsTrigger value="questions">Student Questions</TabsTrigger>
          <TabsTrigger value="messages">Messages</TabsTrigger>
        </TabsList>

        <TabsContent value="announcements" className="space-y-4 mt-6">
          <div className="space-y-3">
            {mockAnnouncements.map((announcement) => (
              <Card key={announcement.id}>
                <CardHeader>
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-1">
                        <CardTitle className="text-lg">{announcement.title}</CardTitle>
                        {announcement.isPinned && (
                          <Pin className="h-4 w-4 text-primary" />
                        )}
                      </div>
                      <CardDescription>
                        <Badge variant="outline" className="mr-2">
                          {announcement.courseName}
                        </Badge>
                        <span className="text-xs">
                          {formatRelativeTime(announcement.createdAt)}
                        </span>
                      </CardDescription>
                    </div>
                    <Badge
                      variant={
                        announcement.priority === 'high'
                          ? 'destructive'
                          : announcement.priority === 'medium'
                          ? 'default'
                          : 'secondary'
                      }
                    >
                      {announcement.priority}
                    </Badge>
                  </div>
                </CardHeader>
                <CardContent>
                  <p className="text-gray-700 dark:text-gray-300 mb-4">
                    {announcement.message}
                  </p>
                  <div className="flex gap-2">
                    <Button size="sm" variant="outline">
                      Edit
                    </Button>
                    <Button size="sm" variant="outline">
                      Delete
                    </Button>
                    <Button size="sm" variant="outline">
                      Pin
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        <TabsContent value="questions" className="space-y-4 mt-6">
          <div className="space-y-3">
            {mockStudentQuestions.map((question) => (
              <Card key={question.id}>
                <CardContent className="p-4">
                  <div className="flex items-start gap-3">
                    <img
                      src={question.studentAvatar}
                      alt={question.studentName}
                      className="w-10 h-10 rounded-full"
                    />
                    <div className="flex-1">
                      <div className="flex items-center justify-between mb-2">
                        <div>
                          <h4 className="font-semibold text-sm">{question.studentName}</h4>
                          <div className="flex items-center gap-2 mt-1">
                            <Badge variant="outline" className="text-xs">
                              {question.courseName}
                            </Badge>
                            <span className="text-xs text-gray-500">
                              {formatRelativeTime(question.createdAt)}
                            </span>
                          </div>
                        </div>
                        <Badge
                          variant={
                            question.status === 'pending' ? 'warning' : 'success'
                          }
                        >
                          {question.status}
                        </Badge>
                      </div>
                      <p className="text-sm text-gray-700 dark:text-gray-300 mb-3">
                        {question.question}
                      </p>
                      <div className="flex gap-2">
                        <Button size="sm">Reply</Button>
                        <Button size="sm" variant="outline">
                          Mark as Answered
                        </Button>
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        <TabsContent value="messages" className="mt-6">
          <Card>
            <CardContent className="p-12 text-center">
              <MessagesSquare className="h-12 w-12 mx-auto mb-4 text-gray-400" />
              <h3 className="text-lg font-semibold mb-2">Messages Coming Soon</h3>
              <p className="text-gray-500">
                Direct messaging with students will be available soon.
              </p>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
