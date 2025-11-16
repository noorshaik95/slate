'use client';

import { useState } from 'react';
import Link from 'next/link';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Video, VideoOff, Mic, MicOff, Monitor, Users, Settings,
  Calendar, Clock, Plus, Search, Play, X
} from 'lucide-react';
import { mockVideoRooms } from '@/lib/mock-data-extended';
import { formatDateTime } from '@/lib/utils';

export default function VideoPage() {
  const [isMicOn, setIsMicOn] = useState(true);
  const [isVideoOn, setIsVideoOn] = useState(true);
  const [isScreenSharing, setIsScreenSharing] = useState(false);
  const [inMeeting, setInMeeting] = useState(false);
  const [selectedRoom, setSelectedRoom] = useState<typeof mockVideoRooms[0] | null>(null);

  const activeRooms = mockVideoRooms.filter(room => room.isActive);
  const scheduledRooms = mockVideoRooms.filter(room => !room.isActive);

  const joinMeeting = (room: typeof mockVideoRooms[0]) => {
    setSelectedRoom(room);
    setInMeeting(true);
  };

  const leaveMeeting = () => {
    setInMeeting(false);
    setSelectedRoom(null);
    setIsMicOn(true);
    setIsVideoOn(true);
    setIsScreenSharing(false);
  };

  if (inMeeting && selectedRoom) {
    return (
      <div className="fixed inset-0 bg-black z-50 flex flex-col">
        {/* Meeting Header */}
        <div className="bg-gray-900 text-white p-4 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Badge variant="destructive" className="animate-pulse">
              ● LIVE
            </Badge>
            <div>
              <h2 className="font-semibold">{selectedRoom.name}</h2>
              <p className="text-sm text-gray-400">Hosted by {selectedRoom.hostName}</p>
            </div>
          </div>
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2 text-sm">
              <Users className="h-4 w-4" />
              <span>{selectedRoom.participants} participants</span>
            </div>
            <Button variant="ghost" size="sm" className="text-white">
              <Settings className="h-4 w-4" />
            </Button>
          </div>
        </div>

        {/* Video Grid */}
        <div className="flex-1 grid grid-cols-2 md:grid-cols-3 gap-4 p-4 overflow-auto">
          {/* Main video (self) */}
          <div className="relative aspect-video bg-gray-800 rounded-lg overflow-hidden">
            <div className="absolute inset-0 flex items-center justify-center">
              {isVideoOn ? (
                <div className="text-white text-center">
                  <Video className="h-16 w-16 mx-auto mb-2" />
                  <p className="text-sm">You</p>
                </div>
              ) : (
                <div className="text-white text-center">
                  <div className="h-16 w-16 mx-auto mb-2 bg-primary rounded-full flex items-center justify-center text-2xl">
                    J
                  </div>
                  <p className="text-sm">You (Video Off)</p>
                </div>
              )}
            </div>
            <div className="absolute bottom-2 left-2 bg-black/50 px-2 py-1 rounded text-white text-xs">
              You
            </div>
          </div>

          {/* Other participants */}
          {Array.from({ length: Math.min(selectedRoom.participants - 1, 8) }).map((_, i) => (
            <div key={i} className="relative aspect-video bg-gray-800 rounded-lg overflow-hidden">
              <div className="absolute inset-0 flex items-center justify-center text-white">
                <div className="text-center">
                  <div className="h-16 w-16 mx-auto mb-2 bg-blue-600 rounded-full flex items-center justify-center text-2xl">
                    {String.fromCharCode(65 + i)}
                  </div>
                  <p className="text-sm">Participant {i + 1}</p>
                </div>
              </div>
              <div className="absolute bottom-2 left-2 bg-black/50 px-2 py-1 rounded text-white text-xs">
                Participant {i + 1}
              </div>
            </div>
          ))}
        </div>

        {/* Controls */}
        <div className="bg-gray-900 p-6 flex items-center justify-center gap-4">
          <Button
            variant={isMicOn ? 'default' : 'destructive'}
            size="lg"
            className="rounded-full h-14 w-14"
            onClick={() => setIsMicOn(!isMicOn)}
          >
            {isMicOn ? <Mic className="h-6 w-6" /> : <MicOff className="h-6 w-6" />}
          </Button>
          <Button
            variant={isVideoOn ? 'default' : 'destructive'}
            size="lg"
            className="rounded-full h-14 w-14"
            onClick={() => setIsVideoOn(!isVideoOn)}
          >
            {isVideoOn ? <Video className="h-6 w-6" /> : <VideoOff className="h-6 w-6" />}
          </Button>
          <Button
            variant={isScreenSharing ? 'default' : 'secondary'}
            size="lg"
            className="rounded-full h-14 w-14"
            onClick={() => setIsScreenSharing(!isScreenSharing)}
          >
            <Monitor className="h-6 w-6" />
          </Button>
          <Button
            variant="destructive"
            size="lg"
            className="rounded-full px-8"
            onClick={leaveMeeting}
          >
            <X className="h-6 w-6 mr-2" />
            Leave
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Video Conferencing</h1>
          <p className="text-muted-foreground">Join live classes and study sessions</p>
        </div>
        <Button>
          <Plus className="mr-2 h-4 w-4" />
          Create Meeting
        </Button>
      </div>

      {/* Quick Join */}
      <Card>
        <CardHeader>
          <CardTitle>Quick Join</CardTitle>
          <CardDescription>Enter a meeting code or link</CardDescription>
        </CardHeader>
        <CardContent className="flex gap-4">
          <Input placeholder="Enter meeting code or paste link" className="flex-1" />
          <Button>Join</Button>
        </CardContent>
      </Card>

      {/* Active Meetings */}
      {activeRooms.length > 0 && (
        <div className="space-y-4">
          <h2 className="text-xl font-semibold">Active Now</h2>
          <div className="grid gap-4 md:grid-cols-2">
            {activeRooms.map((room) => (
              <Card key={room.id} className="border-2 border-green-500">
                <CardHeader>
                  <div className="flex items-start justify-between">
                    <div className="space-y-1">
                      <div className="flex items-center gap-2">
                        <Badge variant="destructive" className="animate-pulse">
                          ● LIVE
                        </Badge>
                        <Badge variant="secondary">{room.courseName}</Badge>
                      </div>
                      <CardTitle>{room.name}</CardTitle>
                      <CardDescription>Hosted by {room.hostName}</CardDescription>
                    </div>
                  </div>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div className="flex items-center gap-2">
                      <Users className="h-4 w-4 text-muted-foreground" />
                      <span>{room.participants}/{room.maxParticipants} participants</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <Clock className="h-4 w-4 text-muted-foreground" />
                      <span>Started {new Date(room.startTime).toLocaleTimeString()}</span>
                    </div>
                  </div>

                  <div className="flex gap-2">
                    {room.recordingEnabled && (
                      <Badge variant="outline">Recording</Badge>
                    )}
                    {room.chatEnabled && (
                      <Badge variant="outline">Chat</Badge>
                    )}
                  </div>

                  <Button className="w-full" onClick={() => joinMeeting(room)}>
                    <Play className="mr-2 h-4 w-4" />
                    Join Meeting
                  </Button>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      )}

      {/* Scheduled Meetings */}
      <div className="space-y-4">
        <h2 className="text-xl font-semibold">Scheduled Meetings</h2>
        <div className="grid gap-4 md:grid-cols-2">
          {scheduledRooms.map((room) => (
            <Card key={room.id}>
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div className="space-y-1">
                    <Badge variant="secondary">{room.courseName}</Badge>
                    <CardTitle className="text-lg">{room.name}</CardTitle>
                    <CardDescription>Hosted by {room.hostName}</CardDescription>
                  </div>
                </div>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2 text-sm">
                  <div className="flex items-center gap-2">
                    <Calendar className="h-4 w-4 text-muted-foreground" />
                    <span>{formatDateTime(room.startTime)}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Users className="h-4 w-4 text-muted-foreground" />
                    <span>Up to {room.maxParticipants} participants</span>
                  </div>
                </div>

                <div className="flex gap-2">
                  {room.recordingEnabled && (
                    <Badge variant="outline">Recording</Badge>
                  )}
                  {room.chatEnabled && (
                    <Badge variant="outline">Chat</Badge>
                  )}
                </div>

                <Button variant="outline" className="w-full">
                  Add to Calendar
                </Button>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>

      {/* Empty State */}
      {activeRooms.length === 0 && scheduledRooms.length === 0 && (
        <Card className="p-12">
          <div className="flex flex-col items-center text-center space-y-4">
            <Video className="h-16 w-16 text-muted-foreground/50" />
            <div className="space-y-2">
              <h3 className="text-xl font-semibold">No Meetings</h3>
              <p className="text-muted-foreground">
                No active or scheduled meetings. Create one to get started.
              </p>
            </div>
            <Button>
              <Plus className="mr-2 h-4 w-4" />
              Create Meeting
            </Button>
          </div>
        </Card>
      )}
    </div>
  );
}
