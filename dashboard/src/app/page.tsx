'use client';

import { MainLayout } from '@/components/layout/main-layout';
import { DashboardOverview } from '@/components/dashboard/dashboard-overview';

export default function HomePage() {
  return (
    <MainLayout>
      <DashboardOverview />
    </MainLayout>
  );
}
