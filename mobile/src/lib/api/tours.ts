import { request } from './client.js';

export type TourStatus = 'pending' | 'in_progress' | 'completed' | 'skipped';

/** One catalog tour with the user's progress (`GET /api/v1/tours`). */
export interface TourDto {
  id: string;
  version: number;
  /** The tab the tour belongs to: `kitchen`, `recipes`, `groceries`, `settings`. */
  page: string;
  /** Step keys in order; progress is an index into this list. */
  steps: string[];
  status: TourStatus;
  step: number;
}

export function getTours(): Promise<{ tours: TourDto[] }> {
  return request<{ tours: TourDto[] }>('/api/v1/tours');
}

export function advanceTour(id: string, step: number): Promise<void> {
  return request<void>(`/api/v1/tours/${encodeURIComponent(id)}/advance`, {
    method: 'POST',
    body: { step },
  });
}

export function completeTour(id: string): Promise<void> {
  return request<void>(`/api/v1/tours/${encodeURIComponent(id)}/complete`, { method: 'POST' });
}

export function skipTour(id: string, step: number): Promise<void> {
  return request<void>(`/api/v1/tours/${encodeURIComponent(id)}/skip`, {
    method: 'POST',
    body: { step },
  });
}

/** Every tour pending again. */
export function resetTours(): Promise<void> {
  return request<void>('/api/v1/tours/reset', { method: 'POST' });
}
