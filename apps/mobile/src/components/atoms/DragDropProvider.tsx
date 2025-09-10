import React, { createContext, useContext, useState, ReactNode } from 'react';
import type { DragDropMealData, DropTargetData } from '@imkitchen/shared-types';

interface DragDropContextType {
  dragData: DragDropMealData | null;
  dropTargetData: DropTargetData | null;
  startDrag: (data: DragDropMealData) => void;
  endDrag: () => void;
  setDropTarget: (target: DropTargetData | null) => void;
  isDragging: boolean;
  isValidDropTarget: (target: DropTargetData) => boolean;
}

const DragDropContext = createContext<DragDropContextType | null>(null);

export const useDragDrop = () => {
  const context = useContext(DragDropContext);
  if (!context) {
    throw new Error('useDragDrop must be used within a DragDropProvider');
  }
  return context;
};

interface DragDropProviderProps {
  children: ReactNode;
}

export const DragDropProvider: React.FC<DragDropProviderProps> = ({ children }) => {
  const [dragData, setDragData] = useState<DragDropMealData | null>(null);
  const [dropTargetData, setDropTargetData] = useState<DropTargetData | null>(null);

  const startDrag = (data: DragDropMealData) => {
    setDragData(data);
  };

  const endDrag = () => {
    setDragData(null);
    setDropTargetData(null);
  };

  const setDropTarget = (target: DropTargetData | null) => {
    setDropTargetData(target);
  };

  const isDragging = dragData !== null;

  const isValidDropTarget = (target: DropTargetData): boolean => {
    if (!dragData) return false;
    
    // Prevent dropping on the same slot
    if (
      dragData.sourceMealPlanId === target.mealPlanId &&
      dragData.sourceDay === target.day &&
      dragData.sourceMealType === target.mealType
    ) {
      return false;
    }
    
    // Check if target accepts this type of data
    return target.accepts === 'recipe' || target.accepts === 'meal';
  };

  const value: DragDropContextType = {
    dragData,
    dropTargetData,
    startDrag,
    endDrag,
    setDropTarget,
    isDragging,
    isValidDropTarget,
  };

  return (
    <DragDropContext.Provider value={value}>
      {children}
    </DragDropContext.Provider>
  );
};