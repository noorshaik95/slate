'use client';

import { GradientCard } from '@/components/common/gradient-card';
import { StatCard } from '@/components/common/stat-card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { User, Mail, Calendar, BookOpen, Award, Clock, ChevronRight, Edit } from 'lucide-react';
import { mockUser, mockCourses, mockGrades } from '@/lib/mock-data';
import { formatDate } from '@/lib/utils';
import Link from 'next/link';

const courseGradients = [
  'blue-cyan',
  'purple-pink',
  'emerald-teal',
  'orange-red',
] as const;

const courseColors: Record<string, string> = {
  'blue-cyan': 'text-blue-600',
  'purple-pink': 'text-purple-600',
  'emerald-teal': 'text-emerald-600',
  'orange-red': 'text-orange-600',
};

export default function ProfilePage() {
  const totalCredits = mockCourses.reduce((sum, c) => sum + c.credits, 0);
  const avgGrade = mockGrades.length > 0
    ? Math.round(mockGrades.reduce((sum, g) => sum + g.percentage, 0) / mockGrades.length)
    : 0;

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 p-6">
      <div className="max-w-6xl mx-auto space-y-6">
        {/* Header with Edit Button */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold tracking-tight text-gray-900">Profile</h1>
            <p className="text-gray-600">Manage your personal information and academic profile</p>
          </div>
          <Button
            variant="outline"
            className="bg-white border-gray-200 hover:bg-gray-50 transition-all duration-300"
          >
            <Edit className="h-4 w-4 mr-2" />
            Edit Profile
          </Button>
        </div>

        {/* Profile Header Section */}
        <GradientCard gradient="indigo-purple" className="text-white">
          <div className="flex items-start gap-6">
            {/* Avatar */}
            <div className="flex-shrink-0">
              <div className="h-24 w-24 rounded-full bg-white/20 backdrop-blur-sm flex items-center justify-center">
                <User className="h-12 w-12 text-white" />
              </div>
            </div>

            {/* User Info */}
            <div className="flex-1">
              <div className="flex items-center gap-3 mb-2">
                <h2 className="text-2xl font-bold">
                  {mockUser.firstName} {mockUser.lastName}
                </h2>
                <Badge className="bg-white/20 backdrop-blur-sm text-white border-white/30 hover:bg-white/30">
                  {mockUser.role}
                </Badge>
              </div>

              <div className="space-y-2 text-white/90">
                <div className="flex items-center gap-2">
                  <Mail className="h-4 w-4" />
                  <span>{mockUser.email}</span>
                </div>
                <div className="flex items-center gap-2">
                  <Calendar className="h-4 w-4" />
                  <span>Joined {formatDate(mockUser.createdAt)}</span>
                </div>
              </div>

              {mockUser.bio && (
                <p className="mt-4 text-white/90 leading-relaxed">
                  {mockUser.bio}
                </p>
              )}
            </div>
          </div>
        </GradientCard>

        {/* Profile Statistics Section */}
        <div className="grid gap-6 md:grid-cols-3">
          <StatCard
            icon={<BookOpen className="h-6 w-6" />}
            label="Enrolled Courses"
            value={mockCourses.length}
            subtitle={`${totalCredits} total credits`}
            gradient="blue-cyan"
            variant="outline"
          />
          <StatCard
            icon={<Award className="h-6 w-6" />}
            label="Average Grade"
            value={`${avgGrade}%`}
            subtitle={`Across ${mockGrades.length} assignments`}
            gradient="emerald-teal"
            variant="outline"
          />
          <StatCard
            icon={<Clock className="h-6 w-6" />}
            label="Study Time"
            value="42h"
            subtitle="This week"
            gradient="purple-pink"
            variant="outline"
          />
        </div>

        {/* Current Courses List */}
        <div className="bg-white rounded-2xl border border-gray-200 p-6">
          <h3 className="text-xl font-bold text-gray-900 mb-4">Current Courses</h3>
          <div className="space-y-3">
            {mockCourses.map((course, index) => {
              const gradient = courseGradients[index % courseGradients.length];
              const colorClass = courseColors[gradient];
              
              return (
                <Link
                  key={course.id}
                  href={`/courses/${course.id}`}
                  className="flex items-center justify-between p-4 bg-white border border-gray-200 rounded-xl hover-lift transition-all duration-300"
                >
                  <div className="flex items-center gap-4 flex-1">
                    <Badge className={`gradient-${gradient} text-white border-0`}>
                      {course.code}
                    </Badge>
                    <div className="flex-1">
                      <p className="font-semibold text-gray-900">{course.name}</p>
                      <p className="text-sm text-gray-600">{course.instructor}</p>
                    </div>
                  </div>
                  <div className="flex items-center gap-3">
                    <span className={`text-2xl font-bold ${colorClass}`}>
                      {course.progress}%
                    </span>
                    <ChevronRight className="h-5 w-5 text-gray-400" />
                  </div>
                </Link>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}
