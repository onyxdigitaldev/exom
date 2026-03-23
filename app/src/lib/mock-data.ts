// Mock data for Exom chat app

export type Role = 'builder' | 'prefect' | 'moderator' | 'agent' | 'fellow';

export const roleConfig: Record<Role, { label: string; color: string; icon: string }> = {
  builder: { label: 'Hall Builder', color: '#F59E0B', icon: 'crown' },
  prefect: { label: 'Hall Prefect', color: '#EF4444', icon: 'shield' },
  moderator: { label: 'Hall Moderator', color: '#3B82F6', icon: 'wrench' },
  agent: { label: 'Hall Agent', color: '#22C55E', icon: 'user' },
  fellow: { label: 'Hall Fellow', color: '#9CA3AF', icon: 'eye' },
};

export interface User {
  id: string;
  username: string;
  discriminator: string;
  avatar?: string;
  status: 'online' | 'idle' | 'dnd' | 'offline';
  role: Role;
  activity?: string;
}

export interface Hall {
  id: string;
  name: string;
  icon?: string;
  color: string;
  description?: string;
  memberCount?: number;
}

export interface Channel {
  id: string;
  name: string;
  type: 'text' | 'voice' | 'announcement';
  category: string;
  topic?: string;
  unread?: boolean;
  mentionCount?: number;
  connectedUsers?: User[];
}

export interface Message {
  id: string;
  author: User;
  content: string;
  timestamp: Date;
  edited?: boolean;
  replyTo?: { author: User; content: string };
  reactions?: { emoji: string; count: number; reacted: boolean }[];
  image?: string;
  file?: { name: string; size: string };
  threadCount?: number;
}

export const currentUser: User = {
  id: 'user-1',
  username: 'oskodiak',
  discriminator: '0001',
  status: 'online',
  role: 'agent',
};

export const mockUsers: User[] = [
  { id: 'user-2', username: 'Zephyr', discriminator: '4521', status: 'online', role: 'builder', activity: 'Playing Factorio' },
  { id: 'user-3', username: 'NovaStar', discriminator: '7832', status: 'online', role: 'prefect', activity: 'Listening to Spotify' },
  { id: 'user-4', username: 'CrimsonBlade', discriminator: '1234', status: 'idle', role: 'moderator' },
  { id: 'user-5', username: 'EchoWave', discriminator: '5678', status: 'online', role: 'agent', activity: 'VS Code' },
  { id: 'user-6', username: 'LunarFrost', discriminator: '9012', status: 'dnd', role: 'agent' },
  { id: 'user-7', username: 'ShadowPulse', discriminator: '3456', status: 'offline', role: 'fellow' },
  { id: 'user-8', username: 'VoltStrike', discriminator: '7890', status: 'offline', role: 'fellow' },
  { id: 'user-9', username: 'CyberNinja', discriminator: '2345', status: 'online', role: 'agent' },
  { id: 'user-10', username: 'PixelDream', discriminator: '6789', status: 'offline', role: 'fellow' },
];

export const mockHalls: Hall[] = [
  { id: 'hall-1', name: 'Developers Hub', color: '#5865F2', description: 'A community for developers', memberCount: 1234 },
  { id: 'hall-2', name: 'Gaming Lounge', color: '#57F287', description: 'Chill and play games', memberCount: 5678 },
  { id: 'hall-3', name: 'Music Masters', color: '#FEE75C', description: 'Share and discover music', memberCount: 890 },
  { id: 'hall-4', name: 'Art Gallery', color: '#EB459E', description: 'Showcase your artwork', memberCount: 456 },
  { id: 'hall-5', name: 'Tech Talk', color: '#ED4245', description: 'Latest tech discussions', memberCount: 2345 },
];

export const mockChannels: Channel[] = [
  { id: 'ch-1', name: 'general', type: 'text', category: 'TEXT CHANNELS', topic: 'General discussion for everyone', unread: true },
  { id: 'ch-2', name: 'announcements', type: 'announcement', category: 'TEXT CHANNELS', topic: 'Important updates and news' },
  { id: 'ch-3', name: 'off-topic', type: 'text', category: 'TEXT CHANNELS', topic: 'Random conversations' },
  { id: 'ch-4', name: 'dev-talk', type: 'text', category: 'TEXT CHANNELS', topic: 'Developer discussions', mentionCount: 3 },
  { id: 'ch-5', name: 'Lounge', type: 'voice', category: 'VOICE CHANNELS', connectedUsers: [mockUsers[0], mockUsers[1]] },
  { id: 'ch-6', name: 'Gaming', type: 'voice', category: 'VOICE CHANNELS', connectedUsers: [mockUsers[3]] },
  { id: 'ch-7', name: 'Music', type: 'voice', category: 'VOICE CHANNELS' },
];

export const mockMessages: Message[] = [
  {
    id: 'msg-1',
    author: mockUsers[0],
    content: 'Hey everyone! Welcome to the Developers Hub. Feel free to introduce yourselves!',
    timestamp: new Date('2026-03-23T10:00:00'),
  },
  {
    id: 'msg-2',
    author: mockUsers[1],
    content: 'Thanks for having me here! I\'m excited to be part of this community.',
    timestamp: new Date('2026-03-23T10:02:00'),
    reactions: [
      { emoji: '👋', count: 5, reacted: false },
      { emoji: '❤️', count: 3, reacted: true },
      { emoji: '🎉', count: 2, reacted: false },
    ],
  },
  {
    id: 'msg-3',
    author: mockUsers[2],
    content: 'Just pushed a new update to the repository. Check it out!',
    timestamp: new Date('2026-03-23T10:15:00'),
  },
  {
    id: 'msg-4',
    author: mockUsers[2],
    content: 'Also, remember that the code review session is tomorrow at 3 PM.',
    timestamp: new Date('2026-03-23T10:16:00'),
  },
  {
    id: 'msg-5',
    author: mockUsers[3],
    content: 'Has anyone tried the new React 19 features yet?',
    timestamp: new Date('2026-03-23T11:30:00'),
    threadCount: 3,
  },
  {
    id: 'msg-6',
    author: mockUsers[4],
    content: 'Yes! The new compiler is amazing. Here\'s what I\'ve been working on:',
    timestamp: new Date('2026-03-23T11:35:00'),
    replyTo: { author: mockUsers[3], content: 'Has anyone tried the new React 19 features yet?' },
  },
  {
    id: 'msg-7',
    author: mockUsers[4],
    content: '',
    timestamp: new Date('2026-03-23T11:36:00'),
    image: 'gradient',
  },
  {
    id: 'msg-8',
    author: mockUsers[0],
    content: 'That looks incredible! Great work on the UI.',
    timestamp: new Date('2026-03-23T11:40:00'),
  },
  {
    id: 'msg-9',
    author: mockUsers[5],
    content: 'I\'ve attached the quarterly report for everyone to review.',
    timestamp: new Date('2026-03-23T12:00:00'),
    file: { name: 'report.zip', size: '24.5 KB' },
  },
  {
    id: 'msg-10',
    author: mockUsers[1],
    content: 'Quick question: what\'s the best approach for handling state in large applications?',
    timestamp: new Date('2026-03-23T13:00:00'),
  },
  {
    id: 'msg-11',
    author: mockUsers[0],
    content: 'It depends on the use case. For server state, I\'d recommend TanStack Query. For client state, Zustand or Jotai work great.',
    timestamp: new Date('2026-03-23T13:05:00'),
    edited: true,
  },
  {
    id: 'msg-12',
    author: mockUsers[0],
    content: 'You can also use the Context API for simpler cases.',
    timestamp: new Date('2026-03-23T13:06:00'),
  },
  {
    id: 'msg-13',
    author: mockUsers[2],
    content: 'I second that. We migrated from Redux to Zustand last month and it was a game changer.',
    timestamp: new Date('2026-03-23T14:00:00'),
  },
  {
    id: 'msg-14',
    author: mockUsers[7],
    content: 'Hey folks! Just joined the hall. Looking forward to learning from everyone here!',
    timestamp: new Date('2026-03-23T14:30:00'),
  },
  {
    id: 'msg-15',
    author: mockUsers[0],
    content: 'Welcome aboard! Feel free to ask any questions.',
    timestamp: new Date('2026-03-23T14:32:00'),
    reactions: [
      { emoji: '🙌', count: 4, reacted: true },
    ],
  },
  {
    id: 'msg-16',
    author: mockUsers[3],
    content: 'Don\'t forget to check out the Hall Chest for shared resources!',
    timestamp: new Date('2026-03-23T15:00:00'),
  },
  {
    id: 'msg-17',
    author: mockUsers[4],
    content: 'The TypeScript strict mode is really helpful for catching bugs early.',
    timestamp: new Date('2026-03-23T15:30:00'),
  },
  {
    id: 'msg-18',
    author: mockUsers[1],
    content: 'Absolutely! Combined with ESLint, it makes the codebase much more maintainable.',
    timestamp: new Date('2026-03-23T15:35:00'),
  },
  {
    id: 'msg-19',
    author: mockUsers[0],
    content: 'Anyone up for a pair programming session later today?',
    timestamp: new Date('2026-03-23T15:42:00'),
  },
];

export const mockDMs = [
  { user: mockUsers[0], lastMessage: 'See you tomorrow!', timestamp: new Date('2026-03-23T15:00:00') },
  { user: mockUsers[1], lastMessage: 'Thanks for the help!', timestamp: new Date('2026-03-23T14:30:00') },
  { user: mockUsers[3], lastMessage: 'Check out this cool project', timestamp: new Date('2026-03-22T20:00:00') },
];

export const discoveryHalls: Hall[] = [
  { id: 'disc-1', name: 'JavaScript Masters', color: '#F7DF1E', description: 'Everything JavaScript - from vanilla to frameworks. Learn, share, and grow together.', memberCount: 15234 },
  { id: 'disc-2', name: 'Indie Game Devs', color: '#9B59B6', description: 'Indie developers sharing progress, getting feedback, and collaborating on projects.', memberCount: 8765 },
  { id: 'disc-3', name: 'Design Systems', color: '#E74C3C', description: 'Building scalable design systems. Figma, tokens, components, and documentation.', memberCount: 4521 },
  { id: 'disc-4', name: 'AI & ML Hub', color: '#00CED1', description: 'Discuss the latest in AI and machine learning. From theory to practice.', memberCount: 23456 },
  { id: 'disc-5', name: 'Music Production', color: '#FF6B6B', description: 'Producers, beatmakers, and musicians. Share your work and get feedback.', memberCount: 6789 },
  { id: 'disc-6', name: 'Startup Founders', color: '#2ECC71', description: 'Connect with other founders. Share experiences and grow your startup.', memberCount: 3456 },
];
