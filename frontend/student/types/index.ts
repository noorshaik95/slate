// User Types
export interface User {
  id: string;
  email: string;
  firstName: string;
  lastName: string;
  role: 'student' | 'instructor' | 'admin';
  avatar?: string;
  createdAt: string;
}

// Course Types
export interface Course {
  id: string;
  code: string;
  name: string;
  instructor: string;
  progress: number;
  nextDeadline: string;
}

// Assignment Types
export interface Assignment {
  id: string;
  title: string;
  course: string;
  dueDate: string;
  points: number;
  status: 'pending' | 'in-progress' | 'submitted' | 'graded';
}

// Grade Types
export interface Grade {
  id: string;
  assignmentName: string;
  course: string;
  grade: number;
  maxGrade: number;
  gradedAt: string;
  trend: 'up' | 'down' | 'neutral';
}

// Announcement Types
export interface Announcement {
  id: string;
  title: string;
  course: string;
  content: string;
  priority: 'low' | 'medium' | 'high';
  publishedAt: string;
  isRead: boolean;
}

// Calendar Event Types
export interface CalendarEvent {
  id: string;
  title: string;
  type: 'class' | 'deadline' | 'event' | 'exam';
  startTime: string;
  endTime: string;
  location?: string;
  date?: string;
  time?: string;
}
