import { MainLayout } from '@/components/layout/main-layout';
import { InferenceConsole } from '@/components/inference/inference-console';

export default function InferencePage() {
  return (
    <MainLayout>
      <InferenceConsole />
    </MainLayout>
  );
}