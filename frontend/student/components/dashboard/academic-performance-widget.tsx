'use client';

import { GradientCard } from '@/components/common/gradient-card';
import { GraduationCap, BookOpen, Clock, TrendingUp, Award } from 'lucide-react';

export function AcademicPerformanceWidget() {
  return (
    <GradientCard gradient="indigo-purple" className="text-white">
      <div className="space-y-6">
        {/* Header */}
        <div>
          <h2 className="text-2xl font-bold mb-2">Academic Performance</h2>
          <p className="text-white/80 text-sm">Your overall academic statistics</p>
        </div>

        {/* Main Stats */}
        <div className="grid grid-cols-2 gap-4">
          {/* GPA */}
          <div className="bg-white/20 backdrop-blur-sm rounded-xl p-4">
            <div className="flex items-center gap-2 mb-2">
              <GraduationCap className="w-5 h-5 text-white/80" />
              <span className="text-sm font-medium text-white/80">GPA</span>
            </div>
            <p className="text-3xl font-bold">3.85</p>
            <p className="text-xs text-white/70 mt-1">Out of 4.0</p>
          </div>

          {/* Average Grade */}
          <div className="bg-white/20 backdrop-blur-sm rounded-xl p-4">
            <div className="flex items-center gap-2 mb-2">
              <TrendingUp className="w-5 h-5 text-white/80" />
              <span className="text-sm font-medium text-white/80">Average</span>
            </div>
            <p className="text-3xl font-bold">88%</p>
            <p className="text-xs text-white/70 mt-1">All courses</p>
          </div>
        </div>

        {/* Additional Stats */}
        <div className="space-y-3">
          {/* Total Credits */}
          <div className="flex items-center justify-between bg-white/10 backdrop-blur-sm rounded-lg p-3">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-white/20 rounded-lg">
                <Award className="w-5 h-5" />
              </div>
              <div>
                <p className="text-sm font-medium">Total Credits</p>
                <p className="text-xs text-white/70">Completed</p>
              </div>
            </div>
            <p className="text-2xl font-bold">45</p>
          </div>

          {/* Enrolled Courses */}
          <div className="flex items-center justify-between bg-white/10 backdrop-blur-sm rounded-lg p-3">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-white/20 rounded-lg">
                <BookOpen className="w-5 h-5" />
              </div>
              <div>
                <p className="text-sm font-medium">Enrolled Courses</p>
                <p className="text-xs text-white/70">This semester</p>
              </div>
            </div>
            <p className="text-2xl font-bold">4</p>
          </div>

          {/* Study Time */}
          <div className="flex items-center justify-between bg-white/10 backdrop-blur-sm rounded-lg p-3">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-white/20 rounded-lg">
                <Clock className="w-5 h-5" />
              </div>
              <div>
                <p className="text-sm font-medium">Study Time</p>
                <p className="text-xs text-white/70">This week</p>
              </div>
            </div>
            <p className="text-2xl font-bold">24h</p>
          </div>
        </div>
      </div>
    </GradientCard>
  );
}
