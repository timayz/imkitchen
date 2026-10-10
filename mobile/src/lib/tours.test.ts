import { describe, expect, it } from 'vitest';

import type { TourDto } from './api/tours.js';
import { TOURS, pickTour } from './tours.js';

const dto = (id: string, status: TourDto['status'], step = 0): TourDto => ({
  id,
  version: 1,
  page: TOURS[id]!.page,
  steps: TOURS[id]!.steps.map((s) => s.key),
  status,
  step,
});

describe('pickTour', () => {
  it('opens the first pending tour of the tab, in server order', () => {
    const tours = [dto('kitchen', 'pending'), dto('recipes', 'pending'), dto('cooking', 'pending')];
    expect(pickTour(tours, 'kitchen')?.id).toBe('kitchen');
    expect(pickTour(tours, 'recipes')?.id).toBe('recipes');
    expect(pickTour(tours, 'groceries')).toBeNull();
  });

  it('moves on to the next tour of the tab once one is done', () => {
    const tours = [dto('kitchen', 'skipped', 1), dto('cooking', 'pending')];
    expect(pickTour(tours, 'kitchen')?.id).toBe('cooking');
    expect(
      pickTour([dto('kitchen', 'completed'), dto('cooking', 'completed')], 'kitchen')
    ).toBeNull();
  });

  it('resumes an in-progress tour at its recorded step', () => {
    const pick = pickTour([dto('settings', 'in_progress', 2)], 'settings');
    expect(pick?.start).toBe(2);
    expect(pickTour([dto('settings', 'pending', 2)], 'settings')?.start).toBe(0);
  });

  it('ignores tours this app does not know', () => {
    const unknown: TourDto = {
      id: 'billing',
      version: 1,
      page: 'settings',
      steps: ['x'],
      status: 'pending',
      step: 0,
    };
    expect(pickTour([unknown], 'settings')).toBeNull();
  });
});
