'use client';

import { useState } from 'react';
import { useParams } from 'next/navigation';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { AnimatedProgress } from '@/components/common/animated-progress';
import { GradientType } from '@/components/common/gradient-card';
import {
  BookOpen, Clock, Users, FileText, MessageSquare,
  BarChart3, Play, CheckCircle2, Circle
} from 'lucide-react';
import { mockCourses, mockModules } from '@/lib/mock-data';

// Map course colors to gradient types
const colorToGradient: Record<string, GradientType> = {
  blue: 'blue-cyan',
  purple: 'purple-pink',
  green: 'emerald-teal',
  red: 'orange-red',
  amber: 'amber-yellow',
  violet: 'violet-indigo',
  indigo: 'indigo-purple',
};

const gradientBadgeClasses: Record<GradientType, string> = {
  'blue-cyan': 'gradient-blue-cyan',
  'purple-pink': 'gradient-purple-pink',
  'emerald-teal': 'gradient-emerald-teal',
  'orange-red': 'gradient-orange-red',
  'amber-yellow': 'gradient-amber-yellow',
  'violet-indigo': 'gradient-violet-indigo',
  'indigo-purple': 'gradient-indigo-purple',
};

export default function CourseDetailPage() {
  const params = useParams();
  const courseId = params.id as string;
  const [activeTab, setActiveTab] = useState<'content' | 'assignments' | 'discussions' | 'grades'>('content');

  const course = mockCourses.find(c => c.id === courseId);
  const courseModules = mockModules.filter(m => m.courseId === courseId);

  if (!course) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-gray-900 p-6">
        <div className="max-w-7xl mx-auto">
          <Card className="p-12 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl">
            <div className="text-center">
              <h2 className="text-2xl font-bold text-gray-900 dark:text-white">Course not found</h2>
              <p className="text-gray-600 dark:text-gray-400 mt-2">The course you&apos;re looking for doesn&apos;t exist.</p>
            </div>
          </Card>
        </div>
      </div>
    );
  }

  const gradient = colorToGradient[course.color] || 'blue-cyan';

  const tabs = [
    { id: 'content' as const, label: 'Content', icon: BookOpen },
    { id: 'assignments' as const, label: 'Assignments', icon: FileText },
    { id: 'discussions' as const, label: 'Discussions', icon: MessageSquare },
    { id: 'grades' as const, label: 'Grades', icon: BarChart3 },
  ];

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-gray-900 p-6">
      <div className="max-w-7xl mx-auto space-y-6">
        {/* Course Header */}
        <Card className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl shadow-sm">
          <CardHeader>
            <div className="flex items-start justify-between">
              <div className="space-y-3">
                <div className="flex items-center gap-2">
                  <div className={`inline-block px-4 py-1.5 rounded-lg text-white font-semibold text-sm ${gradientBadgeClasses[gradient]}`}>
                    {course.code}
                  </div>
                  <Badge variant="outline" className="border-gray-300 dark:border-gray-600">
                    {course.semester} {course.year}
                  </Badge>
                </div>
                <CardTitle className="text-3xl text-gray-900 dark:text-white">{course.name}</CardTitle>
                <CardDescription className="text-base text-gray-600 dark:text-gray-400">
                  {course.description}
                </CardDescription>
              </div>
              <div className="flex flex-col items-end gap-2">
                <div className="text-right">
                  <p className="text-sm text-gray-600 dark:text-gray-400">Instructor</p>
                  <p className="font-medium text-gray-900 dark:text-white">{course.instructor}</p>
                </div>
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Progress */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Course Progress</span>
                <span className="text-2xl font-bold text-gray-900 dark:text-white">{course.progress}%</span>
              </div>
              <AnimatedProgress
                value={course.progress}
                gradient={gradient}
                size="lg"
                animated={true}
              />
            </div>

            {/* Stats */}
            <div className="grid grid-cols-3 gap-4 pt-4 border-t border-gray-200 dark:border-gray-700">
              <div className="flex items-center gap-2">
                <Users className="h-4 w-4 text-gray-600 dark:text-gray-400" />
                <span className="text-sm text-gray-700 dark:text-gray-300">{course.enrollmentCount} Students</span>
              </div>
              <div className="flex items-center gap-2">
                <Clock className="h-4 w-4 text-gray-600 dark:text-gray-400" />
                <span className="text-sm text-gray-700 dark:text-gray-300">{course.credits} Credits</span>
              </div>
              <div className="flex items-center gap-2">
                <FileText className="h-4 w-4 text-gray-600 dark:text-gray-400" />
                <span className="text-sm text-gray-700 dark:text-gray-300">{courseModules.length} Modules</span>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Tabs */}
        <div className="border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 rounded-t-2xl">
          <nav className="-mb-px flex space-x-8 px-6">
            {tabs.map((tab) => {
              const Icon = tab.icon;
              return (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={`
                    flex items-center gap-2 border-b-2 py-4 px-1 text-sm font-medium transition-colors
                    ${activeTab === tab.id
                      ? 'border-indigo-600 text-indigo-600 dark:border-indigo-400 dark:text-indigo-400'
                      : 'border-transparent text-gray-600 dark:text-gray-400 hover:border-gray-300 dark:hover:border-gray-600 hover:text-gray-900 dark:hover:text-gray-200'
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
              <Card key={module.id} className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl shadow-sm">
                <CardHeader>
                  <div className="flex items-start justify-between">
                    <div>
                      <CardTitle className="text-gray-900 dark:text-white">
                        Module {module.order}: {module.title}
                      </CardTitle>
                      <CardDescription className="text-gray-600 dark:text-gray-400">
                        {module.description}
                      </CardDescription>
                    </div>
                    <Badge 
                      variant={module.isPublished ? 'default' : 'secondary'}
                      className={module.isPublished ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900 dark:text-emerald-300' : ''}
                    >
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
                        className="flex items-center justify-between p-3 rounded-xl border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-all duration-300 cursor-pointer hover-lift"
                      >
                        <div className="flex items-center gap-3">
                          <StatusIcon
                            className={`h-5 w-5 ${item.isCompleted ? 'text-emerald-600 dark:text-emerald-400' : 'text-gray-400 dark:text-gray-600'}`}
                          />
                          <Icon className="h-4 w-4 text-gray-600 dark:text-gray-400" />
                          <div>
                            <p className="font-medium text-gray-900 dark:text-white">{item.title}</p>
                            <div className="flex items-center gap-4 text-sm text-gray-600 dark:text-gray-400">
                              {'duration' in item && item.duration && <span>{item.duration} min</span>}
                              {'points' in item && item.points && <span>{item.points} points</span>}
                              {'dueDate' in item && item.dueDate && (
                                <span className="text-orange-600 dark:text-orange-400 font-medium">
                                  Due {new Date(item.dueDate).toLocaleDateString()}
                                </span>
                              )}
                            </div>
                          </div>
                        </div>
                        {!item.isCompleted && item.type === 'lecture' && (
                          <Button size="sm" className="bg-indigo-600 hover:bg-indigo-700 text-white">
                            Start
                          </Button>
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
          <Card className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl shadow-sm">
            <CardHeader>
              <CardTitle className="text-gray-900 dark:text-white">Assignments</CardTitle>
              <CardDescription className="text-gray-600 dark:text-gray-400">
                View and submit course assignments
              </CardDescription>
            </CardHeader>
            <CardContent>
              <p className="text-gray-600 dark:text-gray-400">Assignment list will appear here</p>
            </CardContent>
          </Card>
        )}

        {activeTab === 'discussions' && (
          <Card className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl shadow-sm">
            <CardHeader>
              <CardTitle className="text-gray-900 dark:text-white">Discussions</CardTitle>
              <CardDescription className="text-gray-600 dark:text-gray-400">
                Participate in course discussions
              </CardDescription>
            </CardHeader>
            <CardContent>
              <p className="text-gray-600 dark:text-gray-400">Discussion threads will appear here</p>
            </CardContent>
          </Card>
        )}

        {activeTab === 'grades' && (
          <Card className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl shadow-sm">
            <CardHeader>
              <CardTitle className="text-gray-900 dark:text-white">Grades</CardTitle>
              <CardDescription className="text-gray-600 dark:text-gray-400">
                View your grades for this course
              </CardDescription>
            </CardHeader>
            <CardContent>
              <p className="text-gray-600 dark:text-gray-400">Grade breakdown will appear here</p>
            </CardContent>
          </Card>
        )}
      </div>
    </div>
  );
}
