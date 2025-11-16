'use client';

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { User, Mail, Calendar, Award, BookOpen, Clock } from 'lucide-react';
import { mockUser, mockCourses, mockGrades } from '@/lib/mock-data';
import { formatDate } from '@/lib/utils';

export default function ProfilePage() {
  const [isEditing, setIsEditing] = useState(false);
  const [formData, setFormData] = useState({
    firstName: mockUser.firstName,
    lastName: mockUser.lastName,
    email: mockUser.email,
    bio: mockUser.bio || '',
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // Save profile changes
    setIsEditing(false);
  };

  const totalCredits = mockCourses.reduce((sum, c) => sum + c.credits, 0);
  const avgGrade = mockGrades.length > 0
    ? Math.round(mockGrades.reduce((sum, g) => sum + g.percentage, 0) / mockGrades.length)
    : 0;

  return (
    <div className="space-y-6 max-w-4xl">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Profile</h1>
        <p className="text-muted-foreground">Manage your personal information and academic profile</p>
      </div>

      {/* Profile Card */}
      <Card>
        <CardHeader>
          <div className="flex items-start justify-between">
            <div className="flex items-center gap-4">
              <img
                src={mockUser.avatar}
                alt={`${mockUser.firstName} ${mockUser.lastName}`}
                className="h-20 w-20 rounded-full"
              />
              <div>
                <CardTitle className="text-2xl">
                  {mockUser.firstName} {mockUser.lastName}
                </CardTitle>
                <CardDescription className="flex items-center gap-2">
                  <Badge>{mockUser.role}</Badge>
                  <span>•</span>
                  <span>Joined {formatDate(mockUser.createdAt)}</span>
                </CardDescription>
              </div>
            </div>
            <Button
              variant={isEditing ? 'default' : 'outline'}
              onClick={() => setIsEditing(!isEditing)}
            >
              {isEditing ? 'Save Changes' : 'Edit Profile'}
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {isEditing ? (
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium">First Name</label>
                  <Input
                    value={formData.firstName}
                    onChange={(e) => setFormData({ ...formData, firstName: e.target.value })}
                  />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">Last Name</label>
                  <Input
                    value={formData.lastName}
                    onChange={(e) => setFormData({ ...formData, lastName: e.target.value })}
                  />
                </div>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Email</label>
                <Input
                  type="email"
                  value={formData.email}
                  onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Bio</label>
                <textarea
                  value={formData.bio}
                  onChange={(e) => setFormData({ ...formData, bio: e.target.value })}
                  className="w-full min-h-[100px] p-3 border rounded-lg"
                  placeholder="Tell us about yourself..."
                />
              </div>
            </form>
          ) : (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1">
                  <p className="text-sm text-muted-foreground">Email</p>
                  <div className="flex items-center gap-2">
                    <Mail className="h-4 w-4 text-muted-foreground" />
                    <p className="font-medium">{mockUser.email}</p>
                  </div>
                </div>
                <div className="space-y-1">
                  <p className="text-sm text-muted-foreground">Member Since</p>
                  <div className="flex items-center gap-2">
                    <Calendar className="h-4 w-4 text-muted-foreground" />
                    <p className="font-medium">{formatDate(mockUser.createdAt)}</p>
                  </div>
                </div>
              </div>
              {mockUser.bio && (
                <div className="space-y-1">
                  <p className="text-sm text-muted-foreground">Bio</p>
                  <p>{mockUser.bio}</p>
                </div>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Academic Overview */}
      <div className="grid gap-6 md:grid-cols-3">
        <Card>
          <CardHeader>
            <CardDescription>Enrolled Courses</CardDescription>
            <div className="flex items-center gap-2">
              <BookOpen className="h-5 w-5 text-primary" />
              <CardTitle className="text-3xl">{mockCourses.length}</CardTitle>
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">{totalCredits} total credits</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardDescription>Average Grade</CardDescription>
            <div className="flex items-center gap-2">
              <Award className="h-5 w-5 text-primary" />
              <CardTitle className="text-3xl">{avgGrade}%</CardTitle>
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">Across {mockGrades.length} assignments</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardDescription>Study Time</CardDescription>
            <div className="flex items-center gap-2">
              <Clock className="h-5 w-5 text-primary" />
              <CardTitle className="text-3xl">42h</CardTitle>
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">This week</p>
          </CardContent>
        </Card>
      </div>

      {/* Current Courses */}
      <Card>
        <CardHeader>
          <CardTitle>Current Courses</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          {mockCourses.map(course => (
            <div key={course.id} className="flex items-center justify-between p-3 border rounded-lg">
              <div>
                <p className="font-medium">{course.name}</p>
                <p className="text-sm text-muted-foreground">{course.code} • {course.instructor}</p>
              </div>
              <Badge>{course.progress}% complete</Badge>
            </div>
          ))}
        </CardContent>
      </Card>
    </div>
  );
}
