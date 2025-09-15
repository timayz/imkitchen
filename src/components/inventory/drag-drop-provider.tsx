'use client';

import { ReactNode, useEffect, useState } from 'react';
import { DndProvider } from 'react-dnd/dist/core';
import { HTML5Backend } from 'react-dnd-html5-backend';
import { TouchBackend } from 'react-dnd-touch-backend';

interface DragDropProviderProps {
  children: ReactNode;
}

export function DragDropProvider({ children }: DragDropProviderProps) {
  const [isTouchDevice, setIsTouchDevice] = useState(false);

  useEffect(() => {
    // Detect touch device
    const checkTouchDevice = () => {
      return 'ontouchstart' in window || navigator.maxTouchPoints > 0;
    };

    setIsTouchDevice(checkTouchDevice());
  }, []);

  // Use touch backend for mobile devices, HTML5 backend for desktop
  const backend = isTouchDevice ? TouchBackend : HTML5Backend;

  const backendOptions = isTouchDevice
    ? {
        enableMouseEvents: true,
        delayTouchStart: 200,
        delayMouseStart: 0,
      }
    : {};

  return (
    <DndProvider backend={backend} options={backendOptions}>
      {children}
    </DndProvider>
  );
}
