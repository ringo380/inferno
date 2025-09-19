'use client';

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import {
  X,
  Download,
  Settings,
  Info,
  AlertTriangle,
  CheckCircle,
} from 'lucide-react';
import { ModelInfo } from '@/types/inferno';
import { formatBytes } from '@/lib/utils';

interface ModelQuantizationProps {
  model: ModelInfo;
  onClose: () => void;
  onQuantize: (options: QuantizationOptions) => void;
}

interface QuantizationOptions {
  quantization: string;
  optimization_level: 'fast' | 'balanced' | 'aggressive';
  precision: 'fp16' | 'fp32' | 'int8';
  target_format?: string;
}

const quantizationMethods = [
  {
    id: 'q4_0',
    name: 'Q4_0',
    description: '4-bit quantization, balanced quality/size',
    sizeReduction: '75%',
    qualityLoss: 'Low',
    speed: 'Fast',
    recommended: true,
  },
  {
    id: 'q4_1',
    name: 'Q4_1',
    description: '4-bit quantization with improved accuracy',
    sizeReduction: '73%',
    qualityLoss: 'Very Low',
    speed: 'Fast',
    recommended: false,
  },
  {
    id: 'q5_0',
    name: 'Q5_0',
    description: '5-bit quantization, higher quality',
    sizeReduction: '65%',
    qualityLoss: 'Minimal',
    speed: 'Medium',
    recommended: false,
  },
  {
    id: 'q5_1',
    name: 'Q5_1',
    description: '5-bit quantization with improved accuracy',
    sizeReduction: '63%',
    qualityLoss: 'Minimal',
    speed: 'Medium',
    recommended: false,
  },
  {
    id: 'q8_0',
    name: 'Q8_0',
    description: '8-bit quantization, excellent quality',
    sizeReduction: '50%',
    qualityLoss: 'None',
    speed: 'Slow',
    recommended: false,
  },
  {
    id: 'f16',
    name: 'FP16',
    description: '16-bit floating point',
    sizeReduction: '50%',
    qualityLoss: 'None',
    speed: 'Medium',
    recommended: false,
  },
];

const optimizationLevels = [
  {
    id: 'fast',
    name: 'Fast',
    description: 'Quick conversion with basic optimizations',
    time: '~5 minutes',
  },
  {
    id: 'balanced',
    name: 'Balanced',
    description: 'Moderate optimization for size and performance',
    time: '~15 minutes',
  },
  {
    id: 'aggressive',
    name: 'Aggressive',
    description: 'Maximum optimization, slower conversion',
    time: '~30 minutes',
  },
];

export function ModelQuantization({ model, onClose, onQuantize }: ModelQuantizationProps) {
  const [selectedQuantization, setSelectedQuantization] = useState('q4_0');
  const [optimizationLevel, setOptimizationLevel] = useState<'fast' | 'balanced' | 'aggressive'>('balanced');
  const [precision, setPrecision] = useState<'fp16' | 'fp32' | 'int8'>('fp16');
  const [isQuantizing, setIsQuantizing] = useState(false);
  const [progress, setProgress] = useState(0);

  const selectedMethod = quantizationMethods.find(m => m.id === selectedQuantization);
  const estimatedSize = model.size * (1 - parseInt(selectedMethod?.sizeReduction || '0') / 100);

  const startQuantization = () => {
    setIsQuantizing(true);
    setProgress(0);

    // Simulate quantization progress
    const interval = setInterval(() => {
      setProgress(prev => {
        if (prev >= 100) {
          clearInterval(interval);
          setIsQuantizing(false);
          onQuantize({
            quantization: selectedQuantization,
            optimization_level: optimizationLevel,
            precision,
          });
          return 100;
        }
        return prev + Math.random() * 5;
      });
    }, 500);
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <Card className="w-full max-w-4xl mx-4 max-h-[90vh] overflow-auto">
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Model Quantization</CardTitle>
              <CardDescription>
                Compress {model.name} to reduce size and improve inference speed
              </CardDescription>
            </div>
            <Button variant="ghost" size="icon" onClick={onClose}>
              <X className="h-4 w-4" />
            </Button>
          </div>
        </CardHeader>

        <CardContent className="space-y-6">
          {/* Model Info */}
          <Card className="bg-muted/50">
            <CardContent className="p-4">
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                <div>
                  <span className="text-muted-foreground">Current Size:</span>
                  <div className="font-medium">{formatBytes(model.size)}</div>
                </div>
                <div>
                  <span className="text-muted-foreground">Format:</span>
                  <div className="font-medium uppercase">{model.format}</div>
                </div>
                <div>
                  <span className="text-muted-foreground">Current Quant:</span>
                  <div className="font-medium">
                    {model.metadata?.quantization || 'Unquantized'}
                  </div>
                </div>
                <div>
                  <span className="text-muted-foreground">Parameters:</span>
                  <div className="font-medium">
                    {model.metadata?.parameters
                      ? `${(model.metadata.parameters / 1000000000).toFixed(1)}B`
                      : 'Unknown'}
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Quantization Progress */}
          {isQuantizing && (
            <Card>
              <CardContent className="p-4">
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <span className="font-medium">Quantizing model...</span>
                    <span className="text-sm text-muted-foreground">
                      {Math.round(progress)}%
                    </span>
                  </div>
                  <Progress value={progress} className="h-2" />
                  <p className="text-sm text-muted-foreground">
                    This may take several minutes depending on model size and optimization level.
                  </p>
                </div>
              </CardContent>
            </Card>
          )}

          {/* Quantization Methods */}
          <div className="space-y-4">
            <h3 className="font-medium">Quantization Method</h3>
            <div className="grid gap-3">
              {quantizationMethods.map((method) => (
                <Card
                  key={method.id}
                  className={`cursor-pointer transition-colors ${
                    selectedQuantization === method.id
                      ? 'ring-2 ring-primary bg-primary/5'
                      : 'hover:bg-accent'
                  }`}
                  onClick={() => setSelectedQuantization(method.id)}
                >
                  <CardContent className="p-4">
                    <div className="flex items-center justify-between">
                      <div className="flex-1">
                        <div className="flex items-center space-x-2">
                          <h4 className="font-medium">{method.name}</h4>
                          {method.recommended && (
                            <Badge variant="success" className="text-xs">
                              Recommended
                            </Badge>
                          )}
                        </div>
                        <p className="text-sm text-muted-foreground mt-1">
                          {method.description}
                        </p>
                      </div>
                      <div className="grid grid-cols-3 gap-4 text-center text-sm">
                        <div>
                          <div className="text-muted-foreground">Size Reduction</div>
                          <div className="font-medium text-green-600">
                            {method.sizeReduction}
                          </div>
                        </div>
                        <div>
                          <div className="text-muted-foreground">Quality Loss</div>
                          <div className="font-medium">{method.qualityLoss}</div>
                        </div>
                        <div>
                          <div className="text-muted-foreground">Speed</div>
                          <div className="font-medium">{method.speed}</div>
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>

          {/* Optimization Level */}
          <div className="space-y-4">
            <h3 className="font-medium">Optimization Level</h3>
            <div className="grid gap-3">
              {optimizationLevels.map((level) => (
                <Card
                  key={level.id}
                  className={`cursor-pointer transition-colors ${
                    optimizationLevel === level.id
                      ? 'ring-2 ring-primary bg-primary/5'
                      : 'hover:bg-accent'
                  }`}
                  onClick={() => setOptimizationLevel(level.id as any)}
                >
                  <CardContent className="p-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <h4 className="font-medium">{level.name}</h4>
                        <p className="text-sm text-muted-foreground">
                          {level.description}
                        </p>
                      </div>
                      <div className="text-sm">
                        <div className="text-muted-foreground">Est. Time</div>
                        <div className="font-medium">{level.time}</div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>

          {/* Precision Setting */}
          <div className="space-y-4">
            <h3 className="font-medium">Precision</h3>
            <div className="flex items-center space-x-1 border rounded-md p-1">
              {(['fp16', 'fp32', 'int8'] as const).map((prec) => (
                <Button
                  key={prec}
                  variant={precision === prec ? 'default' : 'ghost'}
                  size="sm"
                  onClick={() => setPrecision(prec)}
                  className="flex-1"
                >
                  {prec.toUpperCase()}
                </Button>
              ))}
            </div>
          </div>

          {/* Summary */}
          <Card className="bg-blue-50 dark:bg-blue-950/20 border-blue-200 dark:border-blue-800">
            <CardContent className="p-4">
              <div className="flex items-start space-x-3">
                <Info className="h-5 w-5 text-blue-600 dark:text-blue-400 mt-0.5" />
                <div className="space-y-2">
                  <h4 className="font-medium text-blue-900 dark:text-blue-100">
                    Quantization Summary
                  </h4>
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <span className="text-blue-700 dark:text-blue-300">Original Size:</span>
                      <div className="font-medium">{formatBytes(model.size)}</div>
                    </div>
                    <div>
                      <span className="text-blue-700 dark:text-blue-300">Estimated Size:</span>
                      <div className="font-medium text-green-600">
                        {formatBytes(estimatedSize)}
                      </div>
                    </div>
                    <div>
                      <span className="text-blue-700 dark:text-blue-300">Size Reduction:</span>
                      <div className="font-medium text-green-600">
                        {selectedMethod?.sizeReduction}
                      </div>
                    </div>
                    <div>
                      <span className="text-blue-700 dark:text-blue-300">Quality Loss:</span>
                      <div className="font-medium">{selectedMethod?.qualityLoss}</div>
                    </div>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Actions */}
          <div className="flex items-center justify-between pt-4 border-t">
            <div className="flex items-center space-x-2 text-sm text-muted-foreground">
              <AlertTriangle className="h-4 w-4" />
              <span>Original model will be preserved</span>
            </div>
            <div className="flex items-center space-x-2">
              <Button variant="outline" onClick={onClose} disabled={isQuantizing}>
                Cancel
              </Button>
              <Button onClick={startQuantization} disabled={isQuantizing}>
                <Download className="h-4 w-4 mr-2" />
                {isQuantizing ? 'Quantizing...' : 'Start Quantization'}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}