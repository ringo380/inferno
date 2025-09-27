'use client';

import { useState, useCallback } from 'react';
import { useDropzone } from 'react-dropzone';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import {
  Upload,
  X,
  File,
  AlertCircle,
  CheckCircle,
  Cloud,
  HardDrive,
} from 'lucide-react';
import { formatBytes, validateModelFormat, getModelFormatFromFilename } from '@/lib/utils';
import { tauriApi } from '@/lib/tauri-api';

interface ModelUploadProps {
  onClose: () => void;
  onUpload?: (modelPath: string) => void;
  onRefresh?: () => void;
}

interface UploadFile {
  file: File;
  id: string;
  progress: number;
  status: 'pending' | 'uploading' | 'processing' | 'completed' | 'error';
  error?: string;
  selectedPath?: string;
}

export function ModelUpload({ onClose, onUpload, onRefresh }: ModelUploadProps) {
  const [files, setFiles] = useState<UploadFile[]>([]);
  const [uploadMethod, setUploadMethod] = useState<'local' | 'url'>('local');
  const [modelUrl, setModelUrl] = useState('');
  const [isUploading, setIsUploading] = useState(false);

  const onDrop = useCallback((acceptedFiles: File[]) => {
    const newFiles = acceptedFiles.map((file) => ({
      file,
      id: Math.random().toString(36).substr(2, 9),
      progress: 0,
      status: 'pending' as const,
    }));
    setFiles((prev) => [...prev, ...newFiles]);
  }, []);

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      'application/octet-stream': ['.gguf', '.onnx', '.pt', '.pth', '.safetensors'],
    },
    multiple: true,
  });

  const removeFile = (id: string) => {
    setFiles((prev) => prev.filter((f) => f.id !== id));
  };

  const openFileDialog = async () => {
    try {
      const selectedPath = await tauriApi.openFileDialog();
      if (selectedPath) {
        // Create a virtual file object for display purposes
        const fileName = selectedPath.split('/').pop() || 'unknown';
        const virtualFile = {
          file: { name: fileName, size: 0 } as File,
          id: Math.random().toString(36).substr(2, 9),
          progress: 0,
          status: 'pending' as const,
          selectedPath
        };
        setFiles((prev) => [...prev, virtualFile]);
      }
    } catch (error) {
      console.error('Failed to open file dialog:', error);
    }
  };

  const startUpload = async () => {
    if (isUploading) return;
    setIsUploading(true);

    for (const fileItem of files) {
      if (fileItem.status === 'pending') {
        try {
          // Update status to uploading
          setFiles((prev) =>
            prev.map((f) =>
              f.id === fileItem.id ? { ...f, status: 'uploading', progress: 0 } : f
            )
          );

          // Simulate progress for UI feedback
          const progressInterval = setInterval(() => {
            setFiles((prev) =>
              prev.map((f) => {
                if (f.id === fileItem.id && f.progress < 90) {
                  return { ...f, progress: Math.min(f.progress + 10, 90) };
                }
                return f;
              })
            );
          }, 100);

          // Upload the file using Tauri
          const targetPath = await tauriApi.uploadModel(
            (fileItem as any).selectedPath || fileItem.file.name,
            fileItem.file.name
          );

          clearInterval(progressInterval);

          // Mark as completed
          setFiles((prev) =>
            prev.map((f) =>
              f.id === fileItem.id
                ? { ...f, progress: 100, status: 'completed' }
                : f
            )
          );

          // Notify parent of successful upload
          if (onUpload) {
            onUpload(targetPath);
          }
        } catch (error) {
          console.error('Upload failed:', error);
          setFiles((prev) =>
            prev.map((f) =>
              f.id === fileItem.id
                ? { ...f, status: 'error', error: String(error) }
                : f
            )
          );
        }
      }
    }

    setIsUploading(false);

    // Refresh the models list after upload
    if (onRefresh) {
      onRefresh();
    }
  };

  const canUpload = files.length > 0 && files.some(f => f.status === 'pending');

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <Card className="w-full max-w-2xl mx-4 max-h-[90vh] overflow-auto">
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Upload AI Models</CardTitle>
              <CardDescription>
                Upload GGUF, ONNX, PyTorch, or SafeTensors model files
              </CardDescription>
            </div>
            <Button variant="ghost" size="icon" onClick={onClose}>
              <X className="h-4 w-4" />
            </Button>
          </div>
        </CardHeader>

        <CardContent className="space-y-6">
          {/* Upload Method Toggle */}
          <div className="flex items-center space-x-1 border rounded-md p-1">
            <Button
              variant={uploadMethod === 'local' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setUploadMethod('local')}
              className="flex-1"
            >
              <HardDrive className="h-4 w-4 mr-2" />
              Local Files
            </Button>
            <Button
              variant={uploadMethod === 'url' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setUploadMethod('url')}
              className="flex-1"
            >
              <Cloud className="h-4 w-4 mr-2" />
              From URL
            </Button>
          </div>

          {uploadMethod === 'local' ? (
            <>
              {/* File Selection */}
              <div className="space-y-4">
                <div
                  {...getRootProps()}
                  className={`border-2 border-dashed rounded-lg p-8 text-center cursor-pointer transition-colors ${
                    isDragActive
                      ? 'border-primary bg-primary/5'
                      : 'border-muted-foreground/25 hover:border-primary/50 hover:bg-accent/50'
                  }`}
                >
                  <input {...getInputProps()} />
                  <Upload className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
                  <h3 className="text-lg font-medium mb-2">
                    {isDragActive ? 'Drop files here' : 'Upload model files'}
                  </h3>
                  <p className="text-muted-foreground mb-4">
                    Drag and drop your model files here, or use the file picker below
                  </p>
                  <div className="flex flex-wrap justify-center gap-2 mb-4">
                    <Badge variant="secondary">.gguf</Badge>
                    <Badge variant="secondary">.onnx</Badge>
                    <Badge variant="secondary">.pt</Badge>
                    <Badge variant="secondary">.safetensors</Badge>
                  </div>
                </div>

                <div className="flex justify-center">
                  <Button onClick={openFileDialog} variant="outline">
                    <File className="h-4 w-4 mr-2" />
                    Choose Files
                  </Button>
                </div>
              </div>

              {/* File List */}
              {files.length > 0 && (
                <div className="space-y-3">
                  <h4 className="font-medium">Files to Upload</h4>
                  {files.map((fileItem) => (
                    <div
                      key={fileItem.id}
                      className="flex items-center justify-between p-3 border rounded-lg"
                    >
                      <div className="flex items-center space-x-3">
                        <div className="p-2 rounded-md bg-primary/10">
                          <File className="h-4 w-4 text-primary" />
                        </div>
                        <div>
                          <div className="font-medium text-sm">
                            {fileItem.file.name}
                          </div>
                          <div className="flex items-center space-x-2 text-xs text-muted-foreground">
                            <span>{formatBytes(fileItem.file.size)}</span>
                            <span>•</span>
                            <Badge
                              variant={
                                validateModelFormat(fileItem.file.name)
                                  ? 'success'
                                  : 'destructive'
                              }
                              className="text-xs"
                            >
                              {getModelFormatFromFilename(fileItem.file.name).toUpperCase()}
                            </Badge>
                          </div>
                        </div>
                      </div>

                      <div className="flex items-center space-x-3">
                        {/* Status */}
                        {fileItem.status === 'pending' && (
                          <Badge variant="secondary">Pending</Badge>
                        )}
                        {fileItem.status === 'uploading' && (
                          <div className="flex items-center space-x-2">
                            <Progress
                              value={fileItem.progress}
                              className="w-16 h-2"
                            />
                            <span className="text-xs text-muted-foreground w-8">
                              {Math.round(fileItem.progress)}%
                            </span>
                          </div>
                        )}
                        {fileItem.status === 'completed' && (
                          <CheckCircle className="h-5 w-5 text-green-500" />
                        )}
                        {fileItem.status === 'error' && (
                          <AlertCircle className="h-5 w-5 text-red-500" />
                        )}

                        {/* Remove Button */}
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => removeFile(fileItem.id)}
                          className="h-8 w-8"
                        >
                          <X className="h-4 w-4" />
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </>
          ) : (
            /* URL Input */
            <div className="space-y-4">
              <div>
                <label className="text-sm font-medium">Model URL</label>
                <input
                  type="url"
                  placeholder="https://huggingface.co/model/file.gguf"
                  value={modelUrl}
                  onChange={(e) => setModelUrl(e.target.value)}
                  className="w-full mt-2 px-3 py-2 border rounded-md bg-background focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                />
                <p className="text-xs text-muted-foreground mt-1">
                  Supports direct download links from HuggingFace, GitHub, or other hosting services
                </p>
              </div>
            </div>
          )}

          {/* Actions */}
          <div className="flex items-center justify-between pt-4 border-t">
            <div className="text-sm text-muted-foreground">
              {uploadMethod === 'local' && files.length > 0 && (
                <>Total size: {formatBytes(files.reduce((sum, f) => sum + f.file.size, 0))}</>
              )}
            </div>
            <div className="flex items-center space-x-2">
              <Button variant="outline" onClick={onClose}>
                Cancel
              </Button>
              <Button
                onClick={startUpload}
                disabled={uploadMethod === 'local' ? !canUpload || isUploading : !modelUrl.trim()}
              >
                <Upload className="h-4 w-4 mr-2" />
                {isUploading ? 'Uploading...' : uploadMethod === 'local' ? 'Upload Files' : 'Download Model'}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}