'use client';

import { MainLayout } from '@/components/layout/main-layout';
import { StreamingInference } from '@/components/inference/streaming-inference';
import { PageHeader } from '@/components/layout/page-header';
import { Play, Zap } from 'lucide-react';

export default function InferencePage() {
  return (
    <MainLayout>
      <div className="space-y-6">
        <PageHeader
          icon={Play}
          badge={{ text: 'Real-time', variant: 'success' }}
          actions={[
            {
              label: 'Quick Test',
              onClick: () => console.log('Quick test'),
              variant: 'outline',
              icon: Zap
            }
          ]}
        />
        <StreamingInference />
      </div>
    </MainLayout>
  );
}