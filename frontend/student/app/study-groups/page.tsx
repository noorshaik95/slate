'use client';

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Users, Plus, Search, Calendar, Clock, MapPin, Video, MessageSquare,
  UserPlus, Star, Globe, Lock, CheckCircle2
} from 'lucide-react';
import { mockStudyGroups } from '@/lib/mock-data-extended';
import { formatDateTime } from '@/lib/utils';

export default function StudyGroupsPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [joinedGroups, setJoinedGroups] = useState<string[]>(['group-1']);
  const [newGroup, setNewGroup] = useState({
    name: '',
    description: '',
    courseName: '',
    maxMembers: '',
    isPrivate: false,
  });

  const filteredGroups = mockStudyGroups.filter(group =>
    group.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    group.courseName.toLowerCase().includes(searchQuery.toLowerCase()) ||
    group.description.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const myGroups = filteredGroups.filter(g => joinedGroups.includes(g.id));
  const availableGroups = filteredGroups.filter(g => !joinedGroups.includes(g.id));

  const handleJoinGroup = (groupId: string) => {
    setJoinedGroups([...joinedGroups, groupId]);
  };

  const handleLeaveGroup = (groupId: string) => {
    setJoinedGroups(joinedGroups.filter(id => id !== groupId));
  };

  const handleCreateGroup = () => {
    // Handle group creation
    console.log('Creating group:', newGroup);
    setShowCreateDialog(false);
    setNewGroup({
      name: '',
      description: '',
      courseName: '',
      maxMembers: '',
      isPrivate: false,
    });
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Study Groups</h1>
          <p className="text-muted-foreground">Collaborate with peers and join study sessions</p>
        </div>
        <Button onClick={() => setShowCreateDialog(!showCreateDialog)}>
          <Plus className="mr-2 h-4 w-4" />
          Create Group
        </Button>
      </div>

      {/* Search */}
      <Card>
        <CardContent className="pt-6">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="Search study groups..."
              className="pl-10"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
        </CardContent>
      </Card>

      {/* Create Group Form */}
      {showCreateDialog && (
        <Card className="border-2 border-primary">
          <CardHeader>
            <CardTitle>Create New Study Group</CardTitle>
            <CardDescription>Set up a study group for your course</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-2">
                <label className="text-sm font-medium">Group Name</label>
                <Input
                  placeholder="e.g., CS 101 Study Group"
                  value={newGroup.name}
                  onChange={(e) => setNewGroup({ ...newGroup, name: e.target.value })}
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Course Name</label>
                <Input
                  placeholder="e.g., Introduction to Programming"
                  value={newGroup.courseName}
                  onChange={(e) => setNewGroup({ ...newGroup, courseName: e.target.value })}
                />
              </div>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium">Description</label>
              <textarea
                className="w-full min-h-[100px] p-3 border rounded-lg"
                placeholder="Describe the purpose and goals of this study group..."
                value={newGroup.description}
                onChange={(e) => setNewGroup({ ...newGroup, description: e.target.value })}
              />
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-2">
                <label className="text-sm font-medium">Max Members</label>
                <Input
                  type="number"
                  placeholder="10"
                  value={newGroup.maxMembers}
                  onChange={(e) => setNewGroup({ ...newGroup, maxMembers: e.target.value })}
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">Privacy</label>
                <div className="flex items-center gap-4 h-10">
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="privacy"
                      checked={!newGroup.isPrivate}
                      onChange={() => setNewGroup({ ...newGroup, isPrivate: false })}
                      className="cursor-pointer"
                    />
                    <Globe className="h-4 w-4" />
                    <span className="text-sm">Public</span>
                  </label>
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="privacy"
                      checked={newGroup.isPrivate}
                      onChange={() => setNewGroup({ ...newGroup, isPrivate: true })}
                      className="cursor-pointer"
                    />
                    <Lock className="h-4 w-4" />
                    <span className="text-sm">Private</span>
                  </label>
                </div>
              </div>
            </div>

            <div className="flex gap-2">
              <Button onClick={handleCreateGroup}>
                <CheckCircle2 className="mr-2 h-4 w-4" />
                Create Group
              </Button>
              <Button variant="outline" onClick={() => setShowCreateDialog(false)}>
                Cancel
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* My Study Groups */}
      {myGroups.length > 0 && (
        <div className="space-y-4">
          <h2 className="text-xl font-semibold">My Study Groups</h2>
          <div className="grid gap-4 md:grid-cols-2">
            {myGroups.map((group) => (
              <Card key={group.id} className="border-2 border-primary">
                <CardHeader>
                  <div className="flex items-start justify-between">
                    <div className="space-y-1 flex-1">
                      <div className="flex items-center gap-2">
                        <Badge variant="secondary">{group.courseName}</Badge>
                        {group.isPrivate ? (
                          <Lock className="h-4 w-4 text-muted-foreground" />
                        ) : (
                          <Globe className="h-4 w-4 text-muted-foreground" />
                        )}
                      </div>
                      <CardTitle className="text-lg">{group.name}</CardTitle>
                      <CardDescription>{group.description}</CardDescription>
                    </div>
                  </div>
                </CardHeader>
                <CardContent className="space-y-4">
                  {/* Stats */}
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div className="flex items-center gap-2">
                      <Users className="h-4 w-4 text-muted-foreground" />
                      <span>{group.memberCount}/{group.maxMembers} members</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <Star className="h-4 w-4 text-yellow-500" />
                      <span>{group.studySessionsCount} sessions</span>
                    </div>
                  </div>

                  {/* Next Session */}
                  {group.nextSession && (
                    <div className="p-3 bg-accent rounded-lg space-y-2">
                      <p className="text-sm font-medium">Next Session</p>
                      <div className="space-y-1 text-sm text-muted-foreground">
                        <div className="flex items-center gap-2">
                          <Calendar className="h-3 w-3" />
                          <span>{formatDateTime(group.nextSession.date)}</span>
                        </div>
                        <div className="flex items-center gap-2">
                          <MapPin className="h-3 w-3" />
                          <span>{group.nextSession.location}</span>
                        </div>
                      </div>
                    </div>
                  )}

                  {/* Actions */}
                  <div className="flex gap-2">
                    <Button className="flex-1" size="sm">
                      <MessageSquare className="mr-2 h-4 w-4" />
                      Open Chat
                    </Button>
                    <Button variant="outline" size="sm">
                      <Video className="mr-2 h-4 w-4" />
                      Join Session
                    </Button>
                  </div>

                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full"
                    onClick={() => handleLeaveGroup(group.id)}
                  >
                    Leave Group
                  </Button>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      )}

      {/* Available Groups */}
      <div className="space-y-4">
        <h2 className="text-xl font-semibold">
          {myGroups.length > 0 ? 'Discover More Groups' : 'All Study Groups'}
        </h2>
        {availableGroups.length > 0 ? (
          <div className="grid gap-4 md:grid-cols-2">
            {availableGroups.map((group) => {
              const isFull = group.memberCount >= group.maxMembers;

              return (
                <Card key={group.id}>
                  <CardHeader>
                    <div className="flex items-start justify-between">
                      <div className="space-y-1 flex-1">
                        <div className="flex items-center gap-2">
                          <Badge variant="secondary">{group.courseName}</Badge>
                          {group.isPrivate ? (
                            <Lock className="h-4 w-4 text-muted-foreground" />
                          ) : (
                            <Globe className="h-4 w-4 text-muted-foreground" />
                          )}
                          {isFull && (
                            <Badge variant="destructive" className="text-xs">Full</Badge>
                          )}
                        </div>
                        <CardTitle className="text-lg">{group.name}</CardTitle>
                        <CardDescription>{group.description}</CardDescription>
                      </div>
                    </div>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    {/* Stats */}
                    <div className="grid grid-cols-2 gap-4 text-sm">
                      <div className="flex items-center gap-2">
                        <Users className="h-4 w-4 text-muted-foreground" />
                        <span>{group.memberCount}/{group.maxMembers} members</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <Star className="h-4 w-4 text-yellow-500" />
                        <span>{group.studySessionsCount} sessions</span>
                      </div>
                    </div>

                    {/* Members Preview */}
                    <div className="flex items-center gap-2">
                      <div className="flex -space-x-2">
                        {group.members.slice(0, 3).map((member, i) => (
                          <img
                            key={i}
                            src={member.avatar}
                            alt={member.name}
                            className="h-8 w-8 rounded-full border-2 border-background"
                            title={member.name}
                          />
                        ))}
                      </div>
                      {group.members.length > 3 && (
                        <span className="text-sm text-muted-foreground">
                          +{group.members.length - 3} more
                        </span>
                      )}
                    </div>

                    {/* Next Session */}
                    {group.nextSession && (
                      <div className="p-3 bg-accent rounded-lg space-y-2">
                        <p className="text-sm font-medium">Next Session</p>
                        <div className="space-y-1 text-sm text-muted-foreground">
                          <div className="flex items-center gap-2">
                            <Calendar className="h-3 w-3" />
                            <span>{formatDateTime(group.nextSession.date)}</span>
                          </div>
                          <div className="flex items-center gap-2">
                            <MapPin className="h-3 w-3" />
                            <span>{group.nextSession.location}</span>
                          </div>
                        </div>
                      </div>
                    )}

                    {/* Join Button */}
                    <Button
                      className="w-full"
                      onClick={() => handleJoinGroup(group.id)}
                      disabled={isFull}
                    >
                      <UserPlus className="mr-2 h-4 w-4" />
                      {isFull ? 'Group Full' : 'Join Group'}
                    </Button>
                  </CardContent>
                </Card>
              );
            })}
          </div>
        ) : (
          <Card className="p-12">
            <div className="flex flex-col items-center text-center space-y-4">
              <Users className="h-16 w-16 text-muted-foreground/50" />
              <div className="space-y-2">
                <h3 className="text-xl font-semibold">
                  {searchQuery ? 'No groups found' : 'No study groups available'}
                </h3>
                <p className="text-muted-foreground">
                  {searchQuery
                    ? 'Try adjusting your search query'
                    : 'Be the first to create a study group for your course'}
                </p>
              </div>
              {!searchQuery && (
                <Button onClick={() => setShowCreateDialog(true)}>
                  <Plus className="mr-2 h-4 w-4" />
                  Create Study Group
                </Button>
              )}
            </div>
          </Card>
        )}
      </div>
    </div>
  );
}
