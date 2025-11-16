'use client';

import { useEffect } from 'react';
import { initTelemetry } from '@/lib/telemetry/init';

export function TelemetryProvider({ children }: { children: React.ReactNode }) {
  useEffect(() => {
    // Initialize telemetry on mount
    if (typeof window !== 'undefined') {
      initTelemetry();
    }
  }, []);

  return <>{children}</>;
}
