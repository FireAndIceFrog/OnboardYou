/**
 * Contract for an HTTP client passed in from the platform.
 * The platform's ApiClient class satisfies this interface
 * via structural typing — config never constructs its own.
 */
export interface ApiClient {
  get<T = unknown>(path: string): Promise<T>;
  post<T = unknown>(path: string, body?: unknown): Promise<T>;
  put<T = unknown>(path: string, body?: unknown): Promise<T>;
  patch<T = unknown>(path: string, body?: unknown): Promise<T>;
  del<T = unknown>(path: string): Promise<T>;
}
