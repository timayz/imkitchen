import { createClient, SupabaseClient, AuthResponse, User, Session } from '@supabase/supabase-js';
import AsyncStorage from '@react-native-async-storage/async-storage';
import * as Keychain from 'react-native-keychain';

export interface RegisterRequest {
  email: string;
  password: string;
  name: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface AuthUser {
  id: string;
  email: string;
  emailVerified: boolean;
  name: string;
  createdAt: string;
  updatedAt: string;
}

export interface AuthData {
  accessToken: string;
  refreshToken: string;
  expiresIn: number;
  tokenType: string;
  user: AuthUser;
}

class AuthService {
  private supabase: SupabaseClient;

  constructor() {
    const supabaseUrl = process.env.EXPO_PUBLIC_SUPABASE_URL;
    const supabaseAnonKey = process.env.EXPO_PUBLIC_SUPABASE_ANON_KEY;

    if (!supabaseUrl || !supabaseAnonKey) {
      throw new Error('Missing Supabase environment variables');
    }

    this.supabase = createClient(supabaseUrl, supabaseAnonKey, {
      auth: {
        storage: AsyncStorage,
        autoRefreshToken: true,
        persistSession: true,
        detectSessionInUrl: false,
      },
    });
  }

  async register(request: RegisterRequest): Promise<AuthData> {
    const { data, error } = await this.supabase.auth.signUp({
      email: request.email,
      password: request.password,
      options: {
        data: {
          name: request.name,
        },
      },
    });

    if (error) {
      throw new Error(`Registration failed: ${error.message}`);
    }

    if (!data.user || !data.session) {
      throw new Error('Registration failed: No user or session data returned');
    }

    const authData = this.convertToAuthData(data.user, data.session);
    await this.storeTokensSecurely(data.session.access_token, data.session.refresh_token);
    
    return authData;
  }

  async login(request: LoginRequest): Promise<AuthData> {
    const { data, error } = await this.supabase.auth.signInWithPassword({
      email: request.email,
      password: request.password,
    });

    if (error) {
      throw new Error(`Login failed: ${error.message}`);
    }

    if (!data.user || !data.session) {
      throw new Error('Login failed: No user or session data returned');
    }

    const authData = this.convertToAuthData(data.user, data.session);
    await this.storeTokensSecurely(data.session.access_token, data.session.refresh_token);
    
    return authData;
  }

  async logout(): Promise<void> {
    const { error } = await this.supabase.auth.signOut();
    
    if (error) {
      throw new Error(`Logout failed: ${error.message}`);
    }

    await this.clearStoredTokens();
  }

  async refreshToken(): Promise<AuthData | null> {
    const { data, error } = await this.supabase.auth.refreshSession();

    if (error) {
      throw new Error(`Token refresh failed: ${error.message}`);
    }

    if (!data.user || !data.session) {
      return null;
    }

    const authData = this.convertToAuthData(data.user, data.session);
    await this.storeTokensSecurely(data.session.access_token, data.session.refresh_token);
    
    return authData;
  }

  async getCurrentUser(): Promise<AuthUser | null> {
    const { data: { user }, error } = await this.supabase.auth.getUser();

    if (error || !user) {
      return null;
    }

    return this.convertToAuthUser(user);
  }

  async getCurrentSession(): Promise<Session | null> {
    const { data: { session }, error } = await this.supabase.auth.getSession();

    if (error || !session) {
      return null;
    }

    return session;
  }

  async forgotPassword(email: string): Promise<void> {
    const { error } = await this.supabase.auth.resetPasswordForEmail(email, {
      redirectTo: 'imkitchen://auth/reset-password',
    });

    if (error) {
      throw new Error(`Password reset failed: ${error.message}`);
    }
  }

  async resetPassword(newPassword: string): Promise<void> {
    const { error } = await this.supabase.auth.updateUser({
      password: newPassword,
    });

    if (error) {
      throw new Error(`Password update failed: ${error.message}`);
    }
  }

  async restoreSession(): Promise<AuthData | null> {
    try {
      const { data: { session }, error } = await this.supabase.auth.getSession();

      if (error || !session || !session.user) {
        await this.clearStoredTokens();
        return null;
      }

      // Check if session is expired
      if (session.expires_at && session.expires_at < Date.now() / 1000) {
        // Try to refresh
        return await this.refreshToken();
      }

      return this.convertToAuthData(session.user, session);
    } catch (error) {
      console.error('Session restore failed:', error);
      await this.clearStoredTokens();
      return null;
    }
  }

  onAuthStateChange(callback: (event: string, session: Session | null) => void) {
    return this.supabase.auth.onAuthStateChange(callback);
  }

  async syncSessionAcrossDevices(): Promise<void> {
    try {
      const { data: { session }, error } = await this.supabase.auth.getSession();
      
      if (error) {
        console.error('Session sync failed:', error);
        return;
      }

      if (session) {
        await this.storeTokensSecurely(session.access_token, session.refresh_token);
      }
    } catch (error) {
      console.error('Cross-device session sync failed:', error);
    }
  }

  private convertToAuthData(user: User, session: Session): AuthData {
    return {
      accessToken: session.access_token,
      refreshToken: session.refresh_token,
      expiresIn: session.expires_in || 3600,
      tokenType: session.token_type || 'bearer',
      user: this.convertToAuthUser(user),
    };
  }

  private convertToAuthUser(user: User): AuthUser {
    return {
      id: user.id,
      email: user.email || '',
      emailVerified: !!user.email_confirmed_at,
      name: user.user_metadata?.name || user.user_metadata?.full_name || '',
      createdAt: user.created_at,
      updatedAt: user.updated_at || user.created_at,
    };
  }

  private async storeTokensSecurely(accessToken: string, refreshToken: string): Promise<void> {
    try {
      await Keychain.setInternetCredentials(
        'imkitchen_tokens',
        accessToken,
        refreshToken
      );
    } catch (error) {
      console.error('Failed to store tokens securely:', error);
      // Fallback to AsyncStorage if Keychain fails
      await AsyncStorage.setItem('auth_tokens', JSON.stringify({
        accessToken,
        refreshToken,
      }));
    }
  }

  private async clearStoredTokens(): Promise<void> {
    try {
      await Keychain.resetInternetCredentials('imkitchen_tokens');
    } catch (error) {
      console.error('Failed to clear keychain tokens:', error);
    }

    try {
      await AsyncStorage.removeItem('auth_tokens');
    } catch (error) {
      console.error('Failed to clear AsyncStorage tokens:', error);
    }
  }
}

export default new AuthService();