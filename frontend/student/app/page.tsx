import Link from 'next/link';
import { Button } from '@/components/ui/button';
import { BookOpen, Users, Award, BarChart } from 'lucide-react';

export default function HomePage() {
  return (
    <div className="flex min-h-screen flex-col">
      {/* Hero Section */}
      <section className="flex flex-1 items-center justify-center bg-gradient-to-b from-blue-50 to-white dark:from-gray-900 dark:to-gray-800">
        <div className="container px-4 py-12 md:px-6">
          <div className="flex flex-col items-center space-y-8 text-center">
            <div className="space-y-4">
              <h1 className="text-4xl font-bold tracking-tighter sm:text-5xl md:text-6xl lg:text-7xl">
                Student & Instructor Portal
              </h1>
              <p className="mx-auto max-w-[700px] text-lg text-muted-foreground sm:text-xl">
                A comprehensive learning management system designed for modern education. Access
                courses, assignments, grades, and collaborate with peers all in one place.
              </p>
            </div>
            <div className="flex flex-col gap-4 sm:flex-row">
              <Button asChild size="lg">
                <Link href="/dashboard">Go to Dashboard</Link>
              </Button>
              <Button asChild variant="outline" size="lg">
                <Link href="/login">Login</Link>
              </Button>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="bg-white py-12 dark:bg-gray-800 md:py-24">
        <div className="container px-4 md:px-6">
          <div className="grid gap-8 md:grid-cols-2 lg:grid-cols-4">
            <div className="flex flex-col items-center space-y-3 text-center">
              <div className="flex h-16 w-16 items-center justify-center rounded-full bg-primary/10">
                <BookOpen className="h-8 w-8 text-primary" />
              </div>
              <h3 className="text-xl font-bold">Course Management</h3>
              <p className="text-sm text-muted-foreground">
                Access all your courses, materials, and resources in one organized dashboard
              </p>
            </div>

            <div className="flex flex-col items-center space-y-3 text-center">
              <div className="flex h-16 w-16 items-center justify-center rounded-full bg-primary/10">
                <Users className="h-8 w-8 text-primary" />
              </div>
              <h3 className="text-xl font-bold">Collaboration</h3>
              <p className="text-sm text-muted-foreground">
                Connect with instructors and classmates through discussions and group projects
              </p>
            </div>

            <div className="flex flex-col items-center space-y-3 text-center">
              <div className="flex h-16 w-16 items-center justify-center rounded-full bg-primary/10">
                <Award className="h-8 w-8 text-primary" />
              </div>
              <h3 className="text-xl font-bold">Assignments & Grading</h3>
              <p className="text-sm text-muted-foreground">
                Submit assignments easily and track your progress with real-time grade updates
              </p>
            </div>

            <div className="flex flex-col items-center space-y-3 text-center">
              <div className="flex h-16 w-16 items-center justify-center rounded-full bg-primary/10">
                <BarChart className="h-8 w-8 text-primary" />
              </div>
              <h3 className="text-xl font-bold">Analytics & Insights</h3>
              <p className="text-sm text-muted-foreground">
                View detailed analytics of your performance and identify areas for improvement
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t py-6">
        <div className="container px-4 md:px-6">
          <p className="text-center text-sm text-muted-foreground">
            2025 Student Portal. All rights reserved.
          </p>
        </div>
      </footer>
    </div>
  );
}
