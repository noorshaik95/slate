'use client';

import { useState } from 'react';
import { Card } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import {
  Search,
  Filter,
  Download,
  CheckCircle,
  Clock,
  FileText,
  Star,
  MessageSquare
} from 'lucide-react';
import { mockAssignments, mockCourses } from '@/lib/mock-data';

// Mock submissions data
const mockSubmissions = mockAssignments.flatMap((assignment) =>
  Array.from({ length: 8 }, (_, i) => ({
    id: `${assignment.id}-submission-${i}`,
    assignmentId: assignment.id,
    assignmentTitle: assignment.title,
    studentName: `Student ${i + 1}`,
    studentId: `S${1000 + i}`,
    courseCode: assignment.course,
    submittedAt: new Date(Date.now() - Math.random() * 7 * 24 * 60 * 60 * 1000).toISOString(),
    status: Math.random() > 0.3 ? 'submitted' : 'graded',
    grade: Math.random() > 0.3 ? null : Math.floor(Math.random() * 30) + 70,
    maxPoints: 100,
    filesCount: Math.floor(Math.random() * 3) + 1,
  }))
).slice(0, 25);

export default function GradingPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [filterStatus, setFilterStatus] = useState<'all' | 'submitted' | 'graded'>('all');
  const [selectedSubmission, setSelectedSubmission] = useState<string | null>(null);

  // Filter submissions
  const filteredSubmissions = mockSubmissions.filter((submission) => {
    const matchesSearch =
      submission.studentName.toLowerCase().includes(searchQuery.toLowerCase()) ||
      submission.assignmentTitle.toLowerCase().includes(searchQuery.toLowerCase()) ||
      submission.courseCode.toLowerCase().includes(searchQuery.toLowerCase());

    const matchesFilter =
      filterStatus === 'all' || submission.status === filterStatus;

    return matchesSearch && matchesFilter;
  });

  const pendingCount = mockSubmissions.filter(s => s.status === 'submitted').length;
  const gradedCount = mockSubmissions.filter(s => s.status === 'graded').length;
  const avgGrade = mockSubmissions
    .filter(s => s.grade !== null)
    .reduce((sum, s) => sum + (s.grade || 0), 0) / gradedCount || 0;

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-purple-50 to-pink-50 dark:from-gray-900 dark:via-gray-900 dark:to-gray-900 p-6">
      <div className="max-w-7xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div>
            <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
              Grading Interface
            </h1>
            <p className="text-gray-600 dark:text-gray-400 mt-1">
              Review and grade student submissions
            </p>
          </div>
          <Button className="bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-700 hover:to-pink-700 text-white shadow-lg hover:shadow-xl transition-all">
            <Download className="h-4 w-4 mr-2" />
            Export Grades
          </Button>
        </div>

        {/* Stats Cards */}
        <div className="grid gap-4 md:grid-cols-4">
          <Card className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl hover:shadow-lg transition-shadow">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400">Pending Review</p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white mt-1">
                  {pendingCount}
                </p>
              </div>
              <Clock className="h-10 w-10 text-amber-500" />
            </div>
          </Card>

          <Card className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl hover:shadow-lg transition-shadow">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400">Graded</p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white mt-1">
                  {gradedCount}
                </p>
              </div>
              <CheckCircle className="h-10 w-10 text-emerald-500" />
            </div>
          </Card>

          <Card className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl hover:shadow-lg transition-shadow">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400">Average Grade</p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white mt-1">
                  {avgGrade.toFixed(1)}%
                </p>
              </div>
              <Star className="h-10 w-10 text-purple-500" />
            </div>
          </Card>

          <Card className="p-6 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl hover:shadow-lg transition-shadow">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-gray-600 dark:text-gray-400">Total Submissions</p>
                <p className="text-2xl font-bold text-gray-900 dark:text-white mt-1">
                  {mockSubmissions.length}
                </p>
              </div>
              <FileText className="h-10 w-10 text-indigo-500" />
            </div>
          </Card>
        </div>

        {/* Filters */}
        <div className="flex flex-col sm:flex-row gap-4">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-gray-400" />
            <Input
              type="text"
              placeholder="Search by student, assignment, or course..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-10 rounded-xl border-gray-300 dark:border-gray-700 focus:border-purple-500 focus:ring-purple-500"
            />
          </div>
          <div className="flex gap-2">
            <Button
              variant={filterStatus === 'all' ? 'default' : 'outline'}
              onClick={() => setFilterStatus('all')}
              className={filterStatus === 'all' ? 'bg-purple-600 hover:bg-purple-700' : ''}
            >
              All
            </Button>
            <Button
              variant={filterStatus === 'submitted' ? 'default' : 'outline'}
              onClick={() => setFilterStatus('submitted')}
              className={filterStatus === 'submitted' ? 'bg-purple-600 hover:bg-purple-700' : ''}
            >
              Pending
            </Button>
            <Button
              variant={filterStatus === 'graded' ? 'default' : 'outline'}
              onClick={() => setFilterStatus('graded')}
              className={filterStatus === 'graded' ? 'bg-purple-600 hover:bg-purple-700' : ''}
            >
              Graded
            </Button>
          </div>
        </div>

        {/* Submissions Table */}
        <Card className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-2xl overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700">
                <tr>
                  <th className="px-6 py-4 text-left text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                    Student
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                    Assignment
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                    Course
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                    Submitted
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-4 text-left text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                    Grade
                  </th>
                  <th className="px-6 py-4 text-right text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
                {filteredSubmissions.map((submission) => (
                  <tr
                    key={submission.id}
                    className="hover:bg-gray-50 dark:hover:bg-gray-900 transition-colors"
                  >
                    <td className="px-6 py-4">
                      <div>
                        <div className="font-semibold text-gray-900 dark:text-white">
                          {submission.studentName}
                        </div>
                        <div className="text-sm text-gray-600 dark:text-gray-400">
                          {submission.studentId}
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <div className="text-gray-900 dark:text-white max-w-xs truncate">
                        {submission.assignmentTitle}
                      </div>
                      <div className="text-sm text-gray-600 dark:text-gray-400">
                        {submission.filesCount} file{submission.filesCount > 1 ? 's' : ''}
                      </div>
                    </td>
                    <td className="px-6 py-4 text-gray-900 dark:text-white">
                      {submission.courseCode}
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-600 dark:text-gray-400">
                      {new Date(submission.submittedAt).toLocaleDateString('en-US', {
                        month: 'short',
                        day: 'numeric',
                        hour: '2-digit',
                        minute: '2-digit'
                      })}
                    </td>
                    <td className="px-6 py-4">
                      <span className={`inline-flex items-center px-3 py-1 rounded-full text-xs font-medium ${
                        submission.status === 'graded'
                          ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900 dark:text-emerald-300'
                          : 'bg-amber-100 text-amber-700 dark:bg-amber-900 dark:text-amber-300'
                      }`}>
                        {submission.status === 'graded' ? (
                          <>
                            <CheckCircle className="h-3 w-3 mr-1" />
                            Graded
                          </>
                        ) : (
                          <>
                            <Clock className="h-3 w-3 mr-1" />
                            Pending
                          </>
                        )}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      {submission.grade !== null ? (
                        <span className="font-semibold text-gray-900 dark:text-white">
                          {submission.grade}/{submission.maxPoints}
                        </span>
                      ) : (
                        <span className="text-gray-400">-</span>
                      )}
                    </td>
                    <td className="px-6 py-4">
                      <div className="flex items-center justify-end gap-2">
                        <Button
                          size="sm"
                          className="bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-700 hover:to-pink-700 text-white"
                        >
                          {submission.status === 'graded' ? 'Review' : 'Grade'}
                        </Button>
                        <Button variant="ghost" size="sm">
                          <MessageSquare className="h-4 w-4" />
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {filteredSubmissions.length === 0 && (
            <div className="p-12 text-center">
              <FileText className="mx-auto h-12 w-12 text-gray-400 dark:text-gray-600" />
              <h3 className="mt-4 text-lg font-semibold text-gray-900 dark:text-white">
                No submissions found
              </h3>
              <p className="mt-2 text-gray-600 dark:text-gray-400">
                {searchQuery
                  ? 'Try adjusting your search terms or filters'
                  : 'No submissions available to grade'}
              </p>
            </div>
          )}
        </Card>
      </div>
    </div>
  );
}
