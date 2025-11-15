'use client';

import { useState } from 'react';
import { useParams } from 'next/navigation';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import {
  BookOpen, Clock, Users, FileText, MessageSquare,
  BarChart3, Play, CheckCircle2, Circle, Lock
} from 'lucide-react';
import { mockCourses, mockModules } from '@/lib/mock-data';

export default function CourseDetailPage() {
  const params = useParams();
  const courseId = params.id as string;
  const [activeTab, setActiveTab] = useState<'content' | 'assignments' | 'discussions' | 'grades'>('content');

  const course = mockCourses.find(c => c.id === courseId);
  const courseModules = mockModules.filter(m => m.courseId === courseId);

  if (!course) {
    return <div>Course not found</div>;
  }

  const tabs = [
    { id: 'content' as const, label: 'Content', icon: BookOpen },
    { id: 'assignments' as const, label: 'Assignments', icon: FileText },
    { id: 'discussions' as const, label: 'Discussions', icon: MessageSquare },
    { id: 'grades' as const, label: 'Grades', icon: BarChart3 },
  ];

  return (
    <div className="space-y-6">
      {/* Course Header */}
      <Card>
        <CardHeader>
          <div className="flex items-start justify-between">
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <Badge>{course.code}</Badge>
                <Badge variant="outline">{course.semester} {course.year}</Badge>
              </div>
              <CardTitle className="text-3xl">{course.name}</CardTitle>
              <CardDescription className="text-base">{course.description}</CardDescription>
            </div>
            <div className="flex flex-col items-end gap-2">
              <div className="text-right">
                <p className="text-sm text-muted-foreground">Instructor</p>
                <p className="font-medium">{course.instructor}</p>
              </div>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Progress */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">Course Progress</span>
              <span className="text-sm font-medium">{course.progress}%</span>
            </div>
            <Progress value={course.progress} className="h-2" />
          </div>

          {/* Stats */}
          <div className="grid grid-cols-3 gap-4 pt-4 border-t">
            <div className="flex items-center gap-2">
              <Users className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm">{course.enrollmentCount} Students</span>
            </div>
            <div className="flex items-center gap-2">
              <Clock className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm">{course.credits} Credits</span>
            </div>
            <div className="flex items-center gap-2">
              <FileText className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm">{courseModules.length} Modules</span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Tabs */}
      <div className="border-b">
        <nav className="-mb-px flex space-x-8">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`
                  flex items-center gap-2 border-b-2 py-4 px-1 text-sm font-medium transition-colors
                  ${activeTab === tab.id
                    ? 'border-primary text-primary'
                    : 'border-transparent text-muted-foreground hover:border-gray-300 hover:text-foreground'
                  }
                `}
              >
                <Icon className="h-4 w-4" />
                {tab.label}
              </button>
            );
          })}
        </nav>
      </div>

      {/* Tab Content */}
      {activeTab === 'content' && (
        <div className="space-y-4">
          {courseModules.map((module) => (
            <Card key={module.id}>
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div>
                    <CardTitle>Module {module.order}: {module.title}</CardTitle>
                    <CardDescription>{module.description}</CardDescription>
                  </div>
                  <Badge variant={module.isPublished ? 'default' : 'secondary'}>
                    {module.isPublished ? 'Published' : 'Draft'}
                  </Badge>
                </div>
              </CardHeader>
              <CardContent className="space-y-2">
                {module.items.map((item) => {
                  const Icon = item.type === 'lecture' ? Play :
                               item.type === 'assignment' ? FileText :
                               item.type === 'quiz' ? MessageSquare : FileText;
                  const StatusIcon = item.isCompleted ? CheckCircle2 : Circle;

                  return (
                    <div
                      key={item.id}
                      className="flex items-center justify-between p-3 rounded-lg border hover:bg-accent transition-colors cursor-pointer"
                    >
                      <div className="flex items-center gap-3">
                        <StatusIcon
                          className={`h-5 w-5 ${item.isCompleted ? 'text-green-600' : 'text-muted-foreground'}`}
                        />
                        <Icon className="h-4 w-4 text-muted-foreground" />
                        <div>
                          <p className="font-medium">{item.title}</p>
                          <div className="flex items-center gap-4 text-sm text-muted-foreground">
                            {item.duration && <span>{item.duration} min</span>}
                            {item.points && <span>{item.points} points</span>}
                            {item.dueDate && (
                              <span className="text-orange-600">
                                Due {new Date(item.dueDate).toLocaleDateString()}
                              </span>
                            )}
                          </div>
                        </div>
                      </div>
                      {!item.isCompleted && item.type === 'lecture' && (
                        <Button size="sm">Start</Button>
                      )}
                    </div>
                  );
                })}
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {activeTab === 'assignments' && (
        <Card>
          <CardHeader>
            <CardTitle>Assignments</CardTitle>
            <CardDescription>View and submit course assignments</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground">Assignment list will appear here</p>
          </CardContent>
        </Card>
      )}

      {activeTab === 'discussions' && (
        <Card>
          <CardHeader>
            <CardTitle>Discussions</CardTitle>
            <CardDescription>Participate in course discussions</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground">Discussion threads will appear here</p>
          </CardContent>
        </Card>
      )}

      {activeTab === 'grades' && (
        <Card>
          <CardHeader>
            <CardTitle>Grades</CardTitle>
            <CardDescription>View your grades for this course</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground">Grade breakdown will appear here</p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
