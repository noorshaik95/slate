'use client';

import { useState } from 'react';
import { useParams, useRouter } from 'next/navigation';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Upload, FileText, X, CheckCircle2, Clock, Calendar as CalendarIcon } from 'lucide-react';
import { mockAssignments } from '@/lib/mock-data';
import { formatDateTime } from '@/lib/utils';

export default function AssignmentDetailPage() {
  const params = useParams();
  const router = useRouter();
  const assignmentId = params.id as string;
  const [selectedFiles, setSelectedFiles] = useState<File[]>([]);
  const [textSubmission, setTextSubmission] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  const assignment = mockAssignments.find(a => a.id === assignmentId);

  if (!assignment) {
    return <div>Assignment not found</div>;
  }

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      setSelectedFiles(Array.from(e.target.files));
    }
  };

  const removeFile = (index: number) => {
    setSelectedFiles(files => files.filter((_, i) => i !== index));
  };

  const handleSubmit = async () => {
    setIsSubmitting(true);
    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 2000));
    setIsSubmitting(false);
    router.push('/assignments');
  };

  const isOverdue = new Date(assignment.dueDate) < new Date();
  const canSubmit = assignment.status === 'pending';

  return (
    <div className="space-y-6 max-w-4xl">
      {/* Assignment Header */}
      <Card>
        <CardHeader>
          <div className="space-y-4">
            <div className="flex items-start justify-between">
              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <Badge variant="secondary">{assignment.courseName}</Badge>
                  <Badge variant={assignment.status === 'pending' ? 'default' : 'success'}>
                    {assignment.status}
                  </Badge>
                  <Badge>{assignment.points} points</Badge>
                </div>
                <CardTitle className="text-3xl">{assignment.title}</CardTitle>
                <CardDescription className="text-base">{assignment.description}</CardDescription>
              </div>
            </div>

            {/* Due Date */}
            <div className="flex items-center gap-4 pt-4 border-t">
              <div className="flex items-center gap-2">
                <CalendarIcon className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm">
                  Due: <span className={isOverdue ? 'text-red-600 font-medium' : 'font-medium'}>
                    {formatDateTime(assignment.dueDate)}
                  </span>
                </span>
              </div>
              {isOverdue && (
                <Badge variant="destructive">Overdue</Badge>
              )}
            </div>
          </div>
        </CardHeader>
      </Card>

      {/* Instructions */}
      <Card>
        <CardHeader>
          <CardTitle>Instructions</CardTitle>
        </CardHeader>
        <CardContent className="prose dark:prose-invert max-w-none">
          <p>{assignment.instructions}</p>
        </CardContent>
      </Card>

      {/* Submission */}
      {canSubmit && (
        <Card>
          <CardHeader>
            <CardTitle>Submit Your Work</CardTitle>
            <CardDescription>
              {assignment.submissionType === 'file' ? 'Upload your files' : 'Enter your submission text'}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            {/* File Upload */}
            {assignment.submissionType === 'file' && (
              <div className="space-y-4">
                <div className="border-2 border-dashed rounded-lg p-8 text-center">
                  <Upload className="mx-auto h-12 w-12 text-muted-foreground mb-4" />
                  <div className="space-y-2">
                    <label htmlFor="file-upload" className="cursor-pointer">
                      <span className="text-primary font-medium hover:underline">
                        Click to upload
                      </span>
                      {' '}or drag and drop
                    </label>
                    <Input
                      id="file-upload"
                      type="file"
                      className="hidden"
                      onChange={handleFileChange}
                      multiple
                      accept={assignment.allowedFileTypes?.join(',')}
                    />
                    <p className="text-xs text-muted-foreground">
                      {assignment.allowedFileTypes?.join(', ')}
                      {assignment.maxFileSize && ` (max ${(assignment.maxFileSize / 1024 / 1024).toFixed(0)}MB)`}
                    </p>
                  </div>
                </div>

                {/* Selected Files */}
                {selectedFiles.length > 0 && (
                  <div className="space-y-2">
                    <p className="text-sm font-medium">Selected Files:</p>
                    {selectedFiles.map((file, index) => (
                      <div key={index} className="flex items-center justify-between p-3 border rounded-lg">
                        <div className="flex items-center gap-2">
                          <FileText className="h-4 w-4 text-muted-foreground" />
                          <span className="text-sm">{file.name}</span>
                          <span className="text-xs text-muted-foreground">
                            ({(file.size / 1024).toFixed(1)} KB)
                          </span>
                        </div>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => removeFile(index)}
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
                  className="w-full min-h-[300px] p-4 border rounded-lg focus:outline-none focus:ring-2 focus:ring-primary"
                />
                <p className="mt-2 text-xs text-muted-foreground">
                  {textSubmission.split(/\s+/).filter(Boolean).length} words
                </p>
              </div>
            )}

            {/* Submit Button */}
            <div className="flex gap-4">
              <Button
                onClick={handleSubmit}
                disabled={isSubmitting || (assignment.submissionType === 'file' && selectedFiles.length === 0) || (assignment.submissionType === 'text' && !textSubmission)}
                className="flex-1"
              >
                {isSubmitting ? 'Submitting...' : 'Submit Assignment'}
              </Button>
              <Button variant="outline" onClick={() => router.back()}>
                Cancel
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Already Submitted */}
      {!canSubmit && assignment.submittedAt && (
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <CheckCircle2 className="h-5 w-5 text-green-600" />
              <CardTitle>Submitted</CardTitle>
            </div>
            <CardDescription>
              You submitted this assignment on {formatDateTime(assignment.submittedAt)}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Your instructor will grade this assignment soon. Check back for feedback.
            </p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
