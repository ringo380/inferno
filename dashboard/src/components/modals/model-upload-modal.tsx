'use client';

import { useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Progress } from '@/components/ui/progress';
import { toast } from 'react-hot-toast';
import { invoke } from '@tauri-apps/api/core';
import { Loader2, Upload, FileUp } from 'lucide-react';
import { useQueryClient } from '@tanstack/react-query';

interface ModelUploadModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function ModelUploadModal({ open, onOpenChange }: ModelUploadModalProps) {
  const queryClient = useQueryClient();
  const [isUploading, setIsUploading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [targetName, setTargetName] = useState('');

  const handleFileSelect = async () => {
    try {
      const filePath = await invoke<string | null>('open_file_dialog');

      if (filePath) {
        setSelectedFile(filePath);
        // Extract filename from path
        const fileName = filePath.split(/[\\/]/).pop() || '';
        setTargetName(fileName);
      }
    } catch (error) {
      console.error('File selection error:', error);
      toast.error('Failed to open file dialog');
    }
  };

  const handleUpload = async () => {
    if (!selectedFile) {
      toast.error('Please select a file first');
      return;
    }

    setIsUploading(true);
    setUploadProgress(0);

    try {
      // Simulate progress for user feedback (actual upload is very fast for local files)
      const progressInterval = setInterval(() => {
        setUploadProgress(prev => {
          if (prev >= 90) {
            clearInterval(progressInterval);
            return 90;
          }
          return prev + 10;
        });
      }, 100);

      const targetPath = await invoke<string>('upload_model', {
        sourcePath: selectedFile,
        targetName: targetName || undefined,
      });

      clearInterval(progressInterval);
      setUploadProgress(100);

      toast.success(`Model uploaded successfully to: ${targetPath}`);

      // Invalidate models queries to refresh the list
      await queryClient.invalidateQueries({ queryKey: ['models'] });
      await queryClient.invalidateQueries({ queryKey: ['loaded-models'] });

      // Close modal after a brief delay
      setTimeout(() => {
        handleClose();
      }, 500);

    } catch (error) {
      console.error('Upload error:', error);
      toast.error(`Failed to upload model: ${error}`);
    } finally {
      setIsUploading(false);
    }
  };

  const handleClose = () => {
    if (!isUploading) {
      onOpenChange(false);
      setSelectedFile(null);
      setTargetName('');
      setUploadProgress(0);
    }
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Upload Model</DialogTitle>
          <DialogDescription>
            Upload a new AI model file to your models directory
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* File Selection */}
          <div className="space-y-2">
            <Label>Model File</Label>
            <div className="flex gap-2">
              <Button
                variant="outline"
                onClick={handleFileSelect}
                disabled={isUploading}
                className="flex-1"
              >
                <FileUp className="h-4 w-4 mr-2" />
                {selectedFile ? 'Change File' : 'Select File'}
              </Button>
            </div>
            {selectedFile && (
              <div className="text-sm text-muted-foreground bg-muted p-2 rounded-md break-all">
                {selectedFile}
              </div>
            )}
          </div>

          {/* Target Name */}
          <div className="space-y-2">
            <Label htmlFor="target-name">Target Filename (Optional)</Label>
            <Input
              id="target-name"
              placeholder="Leave empty to use original filename"
              value={targetName}
              onChange={(e) => setTargetName(e.target.value)}
              disabled={isUploading}
            />
            <p className="text-xs text-muted-foreground">
              Customize the filename in your models directory
            </p>
          </div>

          {/* Upload Progress */}
          {isUploading && (
            <div className="space-y-2">
              <Label>Upload Progress</Label>
              <Progress value={uploadProgress} className="w-full" />
              <p className="text-xs text-center text-muted-foreground">
                {uploadProgress}% complete
              </p>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={handleClose}
            disabled={isUploading}
          >
            Cancel
          </Button>
          <Button
            onClick={handleUpload}
            disabled={isUploading || !selectedFile}
          >
            {isUploading ? (
              <>
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                Uploading...
              </>
            ) : (
              <>
                <Upload className="h-4 w-4 mr-2" />
                Upload Model
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
