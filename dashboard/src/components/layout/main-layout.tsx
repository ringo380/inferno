'use client';

import { Header } from './header';
import { Sidebar } from './sidebar';
import { MenuHandler } from './menu-handler';
import { useFileDrop } from '@/hooks/use-file-drop';
import { FileUp } from 'lucide-react';

interface MainLayoutProps {
  children: React.ReactNode;
}

export function MainLayout({ children }: MainLayoutProps) {
  const { isDropping } = useFileDrop();

  return (
    <div className="flex h-screen bg-background relative">
      {/* Global menu event handler */}
      <MenuHandler />

      {/* File drop overlay */}
      {isDropping && (
        <div className="absolute inset-0 z-50 bg-blue-500/20 backdrop-blur-sm flex items-center justify-center pointer-events-none">
          <div className="bg-blue-500 text-white px-8 py-6 rounded-2xl shadow-2xl flex flex-col items-center gap-4 animate-pulse">
            <FileUp className="h-16 w-16" />
            <p className="text-2xl font-bold">Drop Model Files Here</p>
            <p className="text-sm opacity-90">.gguf or .onnx files accepted</p>
          </div>
        </div>
      )}

      {/* Sidebar */}
      <Sidebar />

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Header */}
        <Header />

        {/* Page Content */}
        <main className="flex-1 overflow-auto">
          <div className="container mx-auto p-6">
            {children}
          </div>
        </main>
      </div>
    </div>
  );
}