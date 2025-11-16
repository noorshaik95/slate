'use client';

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import {
  Award, Trophy, Star, Target, Zap, BookOpen, CheckCircle2, Lock,
  TrendingUp, Calendar, Users, MessageSquare, Clock, Flame
} from 'lucide-react';
import { mockAchievements, mockBadges } from '@/lib/mock-data-extended';
import { formatDate } from '@/lib/utils';

export default function AchievementsPage() {
  const [selectedCategory, setSelectedCategory] = useState<string>('all');

  const categories = [
    { id: 'all', name: 'All', icon: Award },
    { id: 'academic', name: 'Academic', icon: BookOpen },
    { id: 'participation', name: 'Participation', icon: Users },
    { id: 'milestones', name: 'Milestones', icon: Target },
  ];

  const filteredAchievements = selectedCategory === 'all'
    ? mockAchievements
    : mockAchievements.filter(a => a.category === selectedCategory);

  const completedAchievements = mockAchievements.filter(a => a.isUnlocked);
  const totalPoints = completedAchievements.reduce((sum, a) => sum + a.points, 0);
  const completionPercentage = (completedAchievements.length / mockAchievements.length) * 100;

  const getAchievementIcon = (iconName: string) => {
    const icons: Record<string, any> = {
      Trophy,
      Star,
      Target,
      Zap,
      BookOpen,
      Users,
      MessageSquare,
      Calendar,
      CheckCircle2,
      Flame,
    };
    return icons[iconName] || Award;
  };

  const getCategoryColor = (category: string) => {
    switch (category) {
      case 'academic':
        return 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300';
      case 'participation':
        return 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300';
      case 'milestones':
        return 'bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300';
      default:
        return 'bg-gray-100 text-gray-700 dark:bg-gray-900 dark:text-gray-300';
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Achievements & Badges</h1>
        <p className="text-muted-foreground">Track your progress and earn rewards</p>
      </div>

      {/* Overview Stats */}
      <div className="grid gap-6 md:grid-cols-4">
        <Card>
          <CardHeader>
            <CardDescription>Total Points</CardDescription>
            <div className="flex items-center gap-2">
              <Star className="h-5 w-5 text-yellow-500" />
              <CardTitle className="text-3xl">{totalPoints}</CardTitle>
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Rank: <span className="font-semibold text-foreground">Gold</span>
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardDescription>Achievements</CardDescription>
            <div className="flex items-center gap-2">
              <Trophy className="h-5 w-5 text-primary" />
              <CardTitle className="text-3xl">
                {completedAchievements.length}/{mockAchievements.length}
              </CardTitle>
            </div>
          </CardHeader>
          <CardContent>
            <Progress value={completionPercentage} className="h-2" />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardDescription>Badges Earned</CardDescription>
            <div className="flex items-center gap-2">
              <Award className="h-5 w-5 text-primary" />
              <CardTitle className="text-3xl">{mockBadges.length}</CardTitle>
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">3 this month</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardDescription>Current Streak</CardDescription>
            <div className="flex items-center gap-2">
              <Flame className="h-5 w-5 text-orange-500" />
              <CardTitle className="text-3xl">7</CardTitle>
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">days active</p>
          </CardContent>
        </Card>
      </div>

      {/* Earned Badges */}
      <Card>
        <CardHeader>
          <CardTitle>Earned Badges</CardTitle>
          <CardDescription>Special recognition for your accomplishments</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-3">
            {mockBadges.map((badge) => (
              <div
                key={badge.id}
                className="flex items-start gap-4 p-4 border rounded-lg bg-gradient-to-br from-yellow-50 to-orange-50 dark:from-yellow-950 dark:to-orange-950"
              >
                <div className="h-16 w-16 rounded-full bg-gradient-to-br from-yellow-400 to-orange-500 flex items-center justify-center flex-shrink-0">
                  <Award className="h-8 w-8 text-white" />
                </div>
                <div className="flex-1 min-w-0">
                  <h3 className="font-semibold mb-1">{badge.name}</h3>
                  <p className="text-sm text-muted-foreground mb-2">{badge.description}</p>
                  <div className="flex items-center gap-2">
                    <Badge variant="secondary" className="text-xs">
                      {badge.tier}
                    </Badge>
                    <span className="text-xs text-muted-foreground">
                      Earned {formatDate(badge.earnedAt)}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Category Filter */}
      <div className="flex items-center gap-2 overflow-x-auto pb-2">
        {categories.map((category) => {
          const Icon = category.icon;
          return (
            <Button
              key={category.id}
              variant={selectedCategory === category.id ? 'default' : 'outline'}
              onClick={() => setSelectedCategory(category.id)}
              className="flex-shrink-0"
            >
              <Icon className="mr-2 h-4 w-4" />
              {category.name}
            </Button>
          );
        })}
      </div>

      {/* Achievements Grid */}
      <div className="grid gap-4 md:grid-cols-2">
        {filteredAchievements.map((achievement) => {
          const Icon = getAchievementIcon(achievement.icon);
          const progressPercentage = (achievement.progress / achievement.goal) * 100;
          const isCompleted = achievement.isUnlocked;

          return (
            <Card
              key={achievement.id}
              className={`transition-all ${
                isCompleted
                  ? 'border-2 border-primary bg-gradient-to-br from-primary/5 to-primary/10'
                  : 'hover:shadow-md'
              }`}
            >
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div className="flex items-start gap-4 flex-1">
                    <div
                      className={`h-16 w-16 rounded-full flex items-center justify-center flex-shrink-0 ${
                        isCompleted
                          ? 'bg-gradient-to-br from-yellow-400 to-orange-500'
                          : 'bg-accent'
                      }`}
                    >
                      {isCompleted ? (
                        <Icon className="h-8 w-8 text-white" />
                      ) : (
                        <Icon className="h-8 w-8 text-muted-foreground" />
                      )}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <CardTitle className="text-lg">{achievement.title}</CardTitle>
                        {isCompleted && (
                          <CheckCircle2 className="h-5 w-5 text-primary flex-shrink-0" />
                        )}
                        {!isCompleted && achievement.progress === 0 && (
                          <Lock className="h-4 w-4 text-muted-foreground flex-shrink-0" />
                        )}
                      </div>
                      <CardDescription>{achievement.description}</CardDescription>
                      <div className="flex items-center gap-2 mt-2">
                        <Badge className={getCategoryColor(achievement.category)} variant="secondary">
                          {achievement.category}
                        </Badge>
                        <Badge variant="outline" className="text-yellow-600">
                          <Star className="h-3 w-3 mr-1" />
                          {achievement.points} pts
                        </Badge>
                      </div>
                    </div>
                  </div>
                </div>
              </CardHeader>
              <CardContent className="space-y-3">
                {/* Progress Bar */}
                {!isCompleted && (
                  <div className="space-y-2">
                    <div className="flex items-center justify-between text-sm">
                      <span className="text-muted-foreground">Progress</span>
                      <span className="font-medium">
                        {achievement.progress} / {achievement.goal}
                      </span>
                    </div>
                    <Progress value={progressPercentage} className="h-2" />
                  </div>
                )}

                {/* Completion Info */}
                {isCompleted && achievement.unlockedAt && (
                  <div className="flex items-center gap-2 p-3 bg-primary/10 rounded-lg">
                    <Trophy className="h-4 w-4 text-primary" />
                    <span className="text-sm font-medium">
                      Unlocked {formatDate(achievement.unlockedAt)}
                    </span>
                  </div>
                )}

                {/* Next Milestone */}
                {!isCompleted && achievement.progress > 0 && (
                  <div className="flex items-center justify-between p-3 bg-accent rounded-lg text-sm">
                    <div className="flex items-center gap-2">
                      <Target className="h-4 w-4 text-muted-foreground" />
                      <span className="text-muted-foreground">
                        {achievement.goal - achievement.progress} more to unlock
                      </span>
                    </div>
                    <Badge variant="secondary">
                      {progressPercentage.toFixed(0)}%
                    </Badge>
                  </div>
                )}
              </CardContent>
            </Card>
          );
        })}
      </div>

      {/* Empty State */}
      {filteredAchievements.length === 0 && (
        <Card className="p-12">
          <div className="flex flex-col items-center text-center space-y-4">
            <Award className="h-16 w-16 text-muted-foreground/50" />
            <div className="space-y-2">
              <h3 className="text-xl font-semibold">No achievements in this category</h3>
              <p className="text-muted-foreground">
                Try selecting a different category to see more achievements.
              </p>
            </div>
          </div>
        </Card>
      )}

      {/* Leaderboard Preview */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Leaderboard</CardTitle>
              <CardDescription>Top students this month</CardDescription>
            </div>
            <Button variant="outline" size="sm">
              View Full Leaderboard
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {[
              { rank: 1, name: 'Sarah Johnson', points: 2450, avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=Sarah' },
              { rank: 2, name: 'You', points: totalPoints, avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=You', isYou: true },
              { rank: 3, name: 'Michael Chen', points: 2180, avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=Michael' },
              { rank: 4, name: 'Emma Davis', points: 2050, avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=Emma' },
              { rank: 5, name: 'James Wilson', points: 1980, avatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=James' },
            ].map((user) => (
              <div
                key={user.rank}
                className={`flex items-center justify-between p-3 rounded-lg ${
                  user.isYou ? 'bg-primary/10 border-2 border-primary' : 'bg-accent'
                }`}
              >
                <div className="flex items-center gap-4">
                  <div
                    className={`w-8 h-8 rounded-full flex items-center justify-center font-bold ${
                      user.rank === 1
                        ? 'bg-yellow-500 text-white'
                        : user.rank === 2
                        ? 'bg-gray-400 text-white'
                        : user.rank === 3
                        ? 'bg-orange-600 text-white'
                        : 'bg-muted text-muted-foreground'
                    }`}
                  >
                    {user.rank}
                  </div>
                  <img
                    src={user.avatar}
                    alt={user.name}
                    className="h-10 w-10 rounded-full"
                  />
                  <div>
                    <p className="font-medium">
                      {user.name}
                      {user.isYou && <Badge className="ml-2" variant="secondary">You</Badge>}
                    </p>
                    <p className="text-sm text-muted-foreground">{user.points} points</p>
                  </div>
                </div>
                {user.rank <= 3 && (
                  <Trophy
                    className={`h-6 w-6 ${
                      user.rank === 1
                        ? 'text-yellow-500'
                        : user.rank === 2
                        ? 'text-gray-400'
                        : 'text-orange-600'
                    }`}
                  />
                )}
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
