/**
 * Authentication utilities
 */

import { User } from '@/types';

export const TOKEN_KEY = 'sentinelle_token';
export const USER_KEY = 'sentinelle_user';

export function setAuth(token: string, user: User): void {
  if (typeof window !== 'undefined') {
    localStorage.setItem(TOKEN_KEY, token);
    localStorage.setItem(USER_KEY, JSON.stringify(user));
  }
}

export function clearAuth(): void {
  if (typeof window !== 'undefined') {
    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(USER_KEY);
  }
}

export function getToken(): string | null {
  if (typeof window !== 'undefined') {
    return localStorage.getItem(TOKEN_KEY);
  }
  return null;
}

export function getUser(): User | null {
  if (typeof window !== 'undefined') {
    const userStr = localStorage.getItem(USER_KEY);
    if (userStr) {
      try {
        return JSON.parse(userStr);
      } catch (e) {
        return null;
      }
    }
  }
  return null;
}

export function isAuthenticated(): boolean {
  return getToken() !== null;
}

export function isAdmin(): boolean {
  const user = getUser();
  return user?.role === 'admin';
}

export function isClient(): boolean {
  const user = getUser();
  return user?.role === 'client';
}
