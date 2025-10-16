'use client';

import { MainLayout } from '@/components/layout/main-layout';
import { ModelManagement } from '@/components/models/model-management';
import { PageHeader } from '@/components/layout/page-header';
import { Brain, Plus, RefreshCw } from 'lucide-react';
import { useQueryClient } from '@tanstack/react-query';
import { toast } from 'react-hot-toast';

export default function ModelsPage() {
  const queryClient = useQueryClient();

  const handleRefreshModels = async () => {
    try {
      // Invalidate all model-related queries to trigger refetch
      await queryClient.invalidateQueries({ queryKey: ['models'] });
      await queryClient.invalidateQueries({ queryKey: ['loaded-models'] });
      toast.success('Models refreshed successfully');
    } catch (error) {
      console.error('Refresh failed:', error);
      toast.error('Failed to refresh models');
    }
  };

  const handleAddModel = () => {
    // This would open a modal or navigate to add model page
    console.log('Add model clicked');
  };

  return (
    <MainLayout>
      <div className="space-y-6">
        <PageHeader
          icon={Brain}
          actions={[
            {
              label: 'Refresh',
              onClick: handleRefreshModels,
              variant: 'outline',
              icon: RefreshCw
            },
            {
              label: 'Add Model',
              onClick: handleAddModel,
              variant: 'default',
              icon: Plus
            }
          ]}
        />
        <ModelManagement />
      </div>
    </MainLayout>
  );
}