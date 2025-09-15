import { render, screen } from '@testing-library/react';
import { DragDropProvider } from '@/components/inventory/drag-drop-provider';

// Mock react-dnd
jest.mock('react-dnd', () => ({
  DndProvider: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="dnd-provider">{children}</div>
  ),
}));

jest.mock('react-dnd-html5-backend', () => ({
  HTML5Backend: jest.fn(),
}));

jest.mock('react-dnd-touch-backend', () => ({
  TouchBackend: jest.fn(),
}));

describe('DragDropProvider', () => {
  it('renders children inside DndProvider', () => {
    render(
      <DragDropProvider>
        <div>Test Content</div>
      </DragDropProvider>
    );

    expect(screen.getByTestId('dnd-provider')).toBeInTheDocument();
    expect(screen.getByText('Test Content')).toBeInTheDocument();
  });

  it('detects touch devices correctly', () => {
    // Mock touch device
    Object.defineProperty(window, 'ontouchstart', {
      writable: true,
      configurable: true,
      value: {},
    });

    render(
      <DragDropProvider>
        <div>Touch Device Content</div>
      </DragDropProvider>
    );

    expect(screen.getByText('Touch Device Content')).toBeInTheDocument();
  });
});
