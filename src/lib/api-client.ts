const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || '/api';

class ApiClient {
  private baseURL: string;

  constructor(baseURL: string = API_BASE_URL) {
    this.baseURL = baseURL;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseURL}${endpoint}`;

    const config: RequestInit = {
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
      ...options,
    };

    const response = await fetch(url, config);

    if (!response.ok) {
      const error = await response.json();
      throw new ApiError(error.message, response.status, error);
    }

    return response.json();
  }

  get<T>(endpoint: string, params?: Record<string, string>): Promise<T> {
    const searchParams = params ? `?${new URLSearchParams(params)}` : '';
    return this.request<T>(`${endpoint}${searchParams}`);
  }

  post<T>(endpoint: string, data?: unknown): Promise<T> {
    const options: RequestInit = {
      method: 'POST',
    };
    if (data) {
      options.body = JSON.stringify(data);
    }
    return this.request<T>(endpoint, options);
  }

  put<T>(endpoint: string, data?: unknown): Promise<T> {
    const options: RequestInit = {
      method: 'PUT',
    };
    if (data) {
      options.body = JSON.stringify(data);
    }
    return this.request<T>(endpoint, options);
  }

  delete<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: 'DELETE' });
  }
}

export const apiClient = new ApiClient();

class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public details?: unknown
  ) {
    super(message);
    this.name = 'ApiError';
  }
}
