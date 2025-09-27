import { MainLayout } from '@/components/layout/main-layout';
import { ModelManagement } from '@/components/models/model-management';
import { PageHeader } from '@/components/layout/page-header';
import { Brain, Plus, RefreshCw } from 'lucide-react';

export default function ModelsPage() {
  const handleRefreshModels = () => {
    // This would trigger a refresh of the models list
    window.location.reload();
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