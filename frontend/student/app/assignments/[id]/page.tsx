'use client';

import { useState } from 'react';
import { useParams, useRouter } from 'next/navigation';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Upload, FileText, X, CheckCircle2, Calendar as CalendarIcon } from 'lucide-react';
import { mockAssignments, mockCourses } from '@/lib/mock-data';
import { formatDateTime } from '@/lib/utils';
import { GradientType } from '@/components/common/gradient-card';

// Map course colors to gradients
const colorToGradient: Record<string, GradientType> = {
  blue: 'blue-cyan',
  purple: 'purple-pink',
  green: 'emerald-teal',
  red: 'orange-red',
  orange: 'amber-yellow',
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

export default function AssignmentDetailPage() {
  const params = useParams();
  const router = useRouter();
  const assignmentId = params.id as string;
  const [selectedFiles, setSelectedFiles] = useState<File[]>([]);
  const [textSubmission, setTextSubmission] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  const assignment = mockAssignments.find((a) => a.id === assignmentId);

  if (!assignment) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-gray-900 p-6">
        <div className="max-w-4xl mx-auto">
          <div className="bg-white dark:bg-gray-800 rounded-2xl border border-gray-200 dark:border-gray-700 p-12 text-center">
            <h2 className="text-2xl font-bold text-gray-900 dark:text-white">Assignment not found</h2>
          </div>
        </div>
      </div>
    );
  }

  const course = mockCourses.find((c) => c.id === assignment.courseId);
  const gradient = course ? colorToGradient[course.color] || 'blue-cyan' : 'blue-cyan';

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      setSelectedFiles(Array.from(e.target.files));
    }
  };

  const removeFile = (index: number) => {
    setSelectedFiles((files) => files.filter((_, i) => i !== index));
  };

  const handleSubmit = async () => {
    setIsSubmitting(true);
    // Simulate API call
    await new Promise((resolve) => setTimeout(resolve, 2000));
    setIsSubmitting(false);
    router.push('/assignments');
  };

  const isOverdue = new Date(assignment.dueDate) < new Date();
  const canSubmit = assignment.status === 'pending';

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-gray-900">
      <div className="space-y-6 p-6 max-w-4xl mx-auto">
        {/* Assignment Header */}
        <div className="bg-white dark:bg-gray-800 rounded-2xl border border-gray-200 dark:border-gray-700 shadow-sm overflow-hidden">
          <div className={`h-2 ${gradientBadgeClasses[gradient]}`} />
          <div className="p-6 space-y-4">
            <div className="flex items-start justify-between">
              <div className="space-y-3 flex-1">
                <div className="flex items-center gap-2 flex-wrap">
                  <div
                    className={`px-3 py-1 rounded-lg text-white font-semibold text-sm ${gradientBadgeClasses[gradient]}`}
                  >
                    {assignment.courseName}
                  </div>
                  <div className="px-3 py-1 rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 font-semibold text-sm">
                    {assignment.status.charAt(0).toUpperCase() + assignment.status.slice(1)}
                  </div>
                  <div className="px-3 py-1 rounded-lg bg-indigo-100 dark:bg-indigo-900 text-indigo-700 dark:text-indigo-300 font-semibold text-sm">
                    {assignment.points} points
                  </div>
                </div>
                <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
                  {assignment.title}
                </h1>
                <p className="text-gray-600 dark:text-gray-400 text-base">
                  {assignment.description}
                </p>
              </div>
            </div>

            {/* Due Date */}
            <div className="flex items-center gap-4 pt-4 border-t border-gray-200 dark:border-gray-700">
              <div className="flex items-center gap-2">
                <CalendarIcon className="h-4 w-4 text-gray-500 dark:text-gray-400" />
                <span className="text-sm text-gray-600 dark:text-gray-400">
                  Due:{' '}
                  <span
                    className={
                      isOverdue
                        ? 'text-red-600 dark:text-red-400 font-semibold'
                        : 'font-semibold text-gray-900 dark:text-white'
                    }
                  >
                    {formatDateTime(assignment.dueDate)}
                  </span>
                </span>
              </div>
              {isOverdue && (
                <div className="px-2 py-1 rounded bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-300 text-xs font-semibold">
                  Overdue
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Instructions */}
        <div className="bg-white dark:bg-gray-800 rounded-2xl border border-gray-200 dark:border-gray-700 shadow-sm p-6">
          <h2 className="text-xl font-bold text-gray-900 dark:text-white mb-4">Instructions</h2>
          <div className="prose dark:prose-invert max-w-none">
            <p className="text-gray-700 dark:text-gray-300">{assignment.instructions}</p>
          </div>
        </div>

        {/* Submission */}
        {canSubmit && (
          <div className="bg-white dark:bg-gray-800 rounded-2xl border border-gray-200 dark:border-gray-700 shadow-sm p-6">
            <div className="space-y-2 mb-6">
              <h2 className="text-xl font-bold text-gray-900 dark:text-white">Submit Your Work</h2>
              <p className="text-gray-600 dark:text-gray-400">
                {assignment.submissionType === 'file'
                  ? 'Upload your files'
                  : 'Enter your submission text'}
              </p>
            </div>

            <div className="space-y-6">
              {/* File Upload */}
              {assignment.submissionType === 'file' && (
                <div className="space-y-4">
                  <div className="border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-xl p-8 text-center hover:border-indigo-400 dark:hover:border-indigo-500 transition-colors">
                    <Upload className="mx-auto h-12 w-12 text-gray-400 dark:text-gray-500 mb-4" />
                    <div className="space-y-2">
                      <label htmlFor="file-upload" className="cursor-pointer">
                        <span className="text-indigo-600 dark:text-indigo-400 font-semibold hover:underline">
                          Click to upload
                        </span>
                        <span className="text-gray-600 dark:text-gray-400"> or drag and drop</span>
                      </label>
                      <Input
                        id="file-upload"
                        type="file"
                        className="hidden"
                        onChange={handleFileChange}
                        multiple
                        accept={assignment.allowedFileTypes?.join(',')}
                      />
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        {assignment.allowedFileTypes?.join(', ')}
                        {assignment.maxFileSize &&
                          ` (max ${(assignment.maxFileSize / 1024 / 1024).toFixed(0)}MB)`}
                      </p>
                    </div>
                  </div>

                  {/* Selected Files */}
                  {selectedFiles.length > 0 && (
                    <div className="space-y-2">
                      <p className="text-sm font-semibold text-gray-900 dark:text-white">
                        Selected Files:
                      </p>
                      {selectedFiles.map((file, index) => (
                        <div
                          key={index}
                          className="flex items-center justify-between p-3 border border-gray-200 dark:border-gray-700 rounded-xl bg-gray-50 dark:bg-gray-900"
                        >
                          <div className="flex items-center gap-2">
                            <FileText className="h-4 w-4 text-gray-500 dark:text-gray-400" />
                            <span className="text-sm text-gray-900 dark:text-white">{file.name}</span>
                            <span className="text-xs text-gray-500 dark:text-gray-400">
                              ({(file.size / 1024).toFixed(1)} KB)
                            </span>
                          </div>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => removeFile(index)}
                            className="hover:bg-red-50 dark:hover:bg-red-900/20 hover:text-red-600 dark:hover:text-red-400"
                          >
                            <X className="h-4 w-4" />
                          </Button>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              )}

              {/* Text Submission */}
              {assignment.submissionType === 'text' && (
                <div>
                  <textarea
                    value={textSubmission}
                    onChange={(e) => setTextSubmission(e.target.value)}
                    placeholder="Enter your submission here..."
                    className="w-full min-h-[300px] p-4 border border-gray-300 dark:border-gray-600 rounded-xl bg-white dark:bg-gray-900 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-indigo-500 dark:focus:ring-indigo-400 focus:border-transparent transition-all"
                  />
                  <p className="mt-2 text-xs text-gray-500 dark:text-gray-400">
                    {textSubmission.split(/\s+/).filter(Boolean).length} words
                  </p>
                </div>
              )}

              {/* Submit Button */}
              <div className="flex gap-4">
                <Button
                  onClick={handleSubmit}
                  disabled={
                    isSubmitting ||
                    (assignment.submissionType === 'file' && selectedFiles.length === 0) ||
                    (assignment.submissionType === 'text' && !textSubmission)
                  }
                  className={`flex-1 font-semibold text-white ${gradientBadgeClasses[gradient]} hover-lift disabled:opacity-50 disabled:cursor-not-allowed`}
                >
                  {isSubmitting ? 'Submitting...' : 'Submit Assignment'}
                </Button>
                <Button
                  variant="outline"
                  onClick={() => router.back()}
                  className="border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-800"
                >
                  Cancel
                </Button>
              </div>
            </div>
          </div>
        )}

        {/* Already Submitted */}
        {!canSubmit && assignment.submittedAt && (
          <div className="bg-white dark:bg-gray-800 rounded-2xl border border-gray-200 dark:border-gray-700 shadow-sm p-6">
            <div className="flex items-center gap-3 mb-3">
              <CheckCircle2 className="h-6 w-6 text-emerald-600 dark:text-emerald-400" />
              <h2 className="text-xl font-bold text-gray-900 dark:text-white">Submitted</h2>
            </div>
            <p className="text-gray-600 dark:text-gray-400 mb-4">
              You submitted this assignment on{' '}
              <span className="font-semibold text-gray-900 dark:text-white">
                {formatDateTime(assignment.submittedAt)}
              </span>
            </p>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Your instructor will grade this assignment soon. Check back for feedback.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
