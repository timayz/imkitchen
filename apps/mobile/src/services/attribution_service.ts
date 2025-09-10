import { ApiClient } from './api_client';
import type {
  RecipeAttribution,
  ContributorProfile,
  CommunityMetricsData,
  MetricsTimeframe,
  ContributorAchievement,
  RecipeAttributionResponse,
  ContributorProfileResponse,
  CommunityMetricsResponse,
  AchievementCreationResponse,
} from '@imkitchen/shared-types';

export class AttributionService {
  private apiClient: ApiClient;

  constructor() {
    this.apiClient = new ApiClient();
  }

  /**
   * Get recipe attribution information
   */
  async getRecipeAttribution(recipeId: string): Promise<RecipeAttribution> {
    try {
      const response = await this.apiClient.get<RecipeAttributionResponse>(
        `/api/v1/recipes/${recipeId}/attribution`
      );

      return response.attribution;
    } catch (error) {
      console.error('Failed to get recipe attribution:', error);
      throw new Error('Failed to get recipe attribution');
    }
  }

  /**
   * Get contributor profile information
   */
  async getContributorProfile(contributorId: string): Promise<ContributorProfile> {
    try {
      const response = await this.apiClient.get<ContributorProfileResponse>(
        `/api/v1/contributors/${contributorId}/profile`
      );

      return response.profile;
    } catch (error) {
      console.error('Failed to get contributor profile:', error);
      throw new Error('Failed to get contributor profile');
    }
  }

  /**
   * Get recipe metrics for a specific recipe
   */
  async getRecipeMetrics(
    recipeId: string,
    timeframe: MetricsTimeframe = 'week'
  ): Promise<CommunityMetricsData> {
    try {
      const response = await this.apiClient.get<CommunityMetricsResponse>(
        `/api/v1/recipes/${recipeId}/metrics?timeframe=${timeframe}`
      );

      return response.metrics;
    } catch (error) {
      console.error('Failed to get recipe metrics:', error);
      throw new Error('Failed to get recipe metrics');
    }
  }

  /**
   * Get contributor metrics
   */
  async getContributorMetrics(
    contributorId: string,
    timeframe: MetricsTimeframe = 'week'
  ): Promise<CommunityMetricsData> {
    try {
      const response = await this.apiClient.get<CommunityMetricsResponse>(
        `/api/v1/contributors/${contributorId}/metrics?timeframe=${timeframe}`
      );

      return response.metrics;
    } catch (error) {
      console.error('Failed to get contributor metrics:', error);
      throw new Error('Failed to get contributor metrics');
    }
  }

  /**
   * Get personal metrics for the current user
   */
  async getPersonalMetrics(timeframe: MetricsTimeframe = 'week'): Promise<CommunityMetricsData> {
    try {
      const response = await this.apiClient.get<CommunityMetricsResponse>(
        `/api/v1/user/metrics?timeframe=${timeframe}`
      );

      return response.metrics;
    } catch (error) {
      console.error('Failed to get personal metrics:', error);
      throw new Error('Failed to get personal metrics');
    }
  }

  /**
   * Get community overview metrics
   */
  async getCommunityOverviewMetrics(timeframe: MetricsTimeframe = 'week'): Promise<CommunityMetricsData> {
    try {
      const response = await this.apiClient.get<CommunityMetricsResponse>(
        `/api/v1/community/metrics?timeframe=${timeframe}`
      );

      return response.metrics;
    } catch (error) {
      console.error('Failed to get community overview metrics:', error);
      throw new Error('Failed to get community overview metrics');
    }
  }

  /**
   * Get contributor achievements
   */
  async getContributorAchievements(contributorId: string): Promise<ContributorAchievement[]> {
    try {
      const response = await this.apiClient.get<{ achievements: ContributorAchievement[] }>(
        `/api/v1/contributors/${contributorId}/achievements`
      );

      return response.achievements;
    } catch (error) {
      console.error('Failed to get contributor achievements:', error);
      throw new Error('Failed to get contributor achievements');
    }
  }

  /**
   * Get user's own achievements
   */
  async getPersonalAchievements(): Promise<ContributorAchievement[]> {
    try {
      const response = await this.apiClient.get<{ achievements: ContributorAchievement[] }>(
        '/api/v1/user/achievements'
      );

      return response.achievements;
    } catch (error) {
      console.error('Failed to get personal achievements:', error);
      throw new Error('Failed to get personal achievements');
    }
  }

  /**
   * Update attribution preferences
   */
  async updateAttributionPreferences(preferences: {
    preserveAttribution: boolean;
    allowDerivatives: boolean;
    requireNotification: boolean;
  }): Promise<void> {
    try {
      await this.apiClient.put('/api/v1/user/attribution-preferences', preferences);
    } catch (error) {
      console.error('Failed to update attribution preferences:', error);
      throw new Error('Failed to update attribution preferences');
    }
  }

  /**
   * Report attribution issue
   */
  async reportAttributionIssue(
    recipeId: string,
    issueType: 'missing_attribution' | 'incorrect_attribution' | 'unauthorized_use',
    description: string
  ): Promise<void> {
    try {
      await this.apiClient.post('/api/v1/attribution/report', {
        recipe_id: recipeId,
        issue_type: issueType,
        description,
      });
    } catch (error) {
      console.error('Failed to report attribution issue:', error);
      throw new Error('Failed to report attribution issue');
    }
  }

  /**
   * Get recipe chain (provenance tracking)
   */
  async getRecipeChain(recipeId: string): Promise<{
    chain: Array<{
      contributorId: string;
      contributorName: string;
      adaptedAt: Date;
      recipeId: string;
    }>;
    totalAdaptations: number;
  }> {
    try {
      const response = await this.apiClient.get<{
        chain: Array<{
          contributor_id: string;
          contributor_name: string;
          adapted_at: string;
          recipe_id: string;
        }>;
        total_adaptations: number;
      }>(`/api/v1/recipes/${recipeId}/chain`);

      return {
        chain: response.chain.map(link => ({
          contributorId: link.contributor_id,
          contributorName: link.contributor_name,
          adaptedAt: new Date(link.adapted_at),
          recipeId: link.recipe_id,
        })),
        totalAdaptations: response.total_adaptations,
      };
    } catch (error) {
      console.error('Failed to get recipe chain:', error);
      throw new Error('Failed to get recipe chain');
    }
  }

  /**
   * Get trending contributors
   */
  async getTrendingContributors(
    limit: number = 10,
    timeframe: MetricsTimeframe = 'week'
  ): Promise<Array<{
    contributor: ContributorProfile;
    metrics: {
      newRecipes: number;
      totalImports: number;
      averageRating: number;
      trendingScore: number;
    };
  }>> {
    try {
      const response = await this.apiClient.get<{
        contributors: Array<{
          contributor: ContributorProfile;
          metrics: {
            new_recipes: number;
            total_imports: number;
            average_rating: number;
            trending_score: number;
          };
        }>;
      }>(`/api/v1/contributors/trending?limit=${limit}&timeframe=${timeframe}`);

      return response.contributors.map(item => ({
        contributor: item.contributor,
        metrics: {
          newRecipes: item.metrics.new_recipes,
          totalImports: item.metrics.total_imports,
          averageRating: item.metrics.average_rating,
          trendingScore: item.metrics.trending_score,
        },
      }));
    } catch (error) {
      console.error('Failed to get trending contributors:', error);
      throw new Error('Failed to get trending contributors');
    }
  }

  /**
   * Get contributor leaderboard
   */
  async getContributorLeaderboard(
    category: 'recipes' | 'imports' | 'ratings' | 'achievements',
    timeframe: MetricsTimeframe = 'month',
    limit: number = 50
  ): Promise<Array<{
    rank: number;
    contributor: ContributorProfile;
    score: number;
    change: number; // Position change from previous period
  }>> {
    try {
      const response = await this.apiClient.get<{
        leaderboard: Array<{
          rank: number;
          contributor: ContributorProfile;
          score: number;
          change: number;
        }>;
      }>(`/api/v1/contributors/leaderboard?category=${category}&timeframe=${timeframe}&limit=${limit}`);

      return response.leaderboard;
    } catch (error) {
      console.error('Failed to get contributor leaderboard:', error);
      throw new Error('Failed to get contributor leaderboard');
    }
  }

  /**
   * Award achievement to a contributor (admin only)
   */
  async awardAchievement(
    contributorId: string,
    achievementType: string,
    metadata?: Record<string, any>
  ): Promise<ContributorAchievement> {
    try {
      const response = await this.apiClient.post<AchievementCreationResponse>(
        `/api/v1/contributors/${contributorId}/achievements`,
        {
          achievement_type: achievementType,
          metadata,
        }
      );

      return response.achievement;
    } catch (error) {
      console.error('Failed to award achievement:', error);
      throw new Error('Failed to award achievement');
    }
  }

  /**
   * Get detailed engagement analytics
   */
  async getEngagementAnalytics(
    recipeId: string,
    timeframe: MetricsTimeframe = 'week'
  ): Promise<{
    dailyViews: Array<{ date: Date; views: number }>;
    referralSources: Array<{ source: string; count: number }>;
    userActions: Array<{ action: string; count: number }>;
    geographicData: Array<{ country: string; count: number }>;
  }> {
    try {
      const response = await this.apiClient.get<{
        daily_views: Array<{ date: string; views: number }>;
        referral_sources: Array<{ source: string; count: number }>;
        user_actions: Array<{ action: string; count: number }>;
        geographic_data: Array<{ country: string; count: number }>;
      }>(`/api/v1/recipes/${recipeId}/analytics?timeframe=${timeframe}`);

      return {
        dailyViews: response.daily_views.map(item => ({
          date: new Date(item.date),
          views: item.views,
        })),
        referralSources: response.referral_sources,
        userActions: response.user_actions,
        geographicData: response.geographic_data,
      };
    } catch (error) {
      console.error('Failed to get engagement analytics:', error);
      throw new Error('Failed to get engagement analytics');
    }
  }

  /**
   * Export attribution data for GDPR compliance
   */
  async exportAttributionData(): Promise<{
    attributions: RecipeAttribution[];
    contributions: Array<{
      recipeId: string;
      title: string;
      createdAt: Date;
      totalImports: number;
    }>;
    achievements: ContributorAchievement[];
  }> {
    try {
      const response = await this.apiClient.get<{
        attributions: RecipeAttribution[];
        contributions: Array<{
          recipe_id: string;
          title: string;
          created_at: string;
          total_imports: number;
        }>;
        achievements: ContributorAchievement[];
      }>('/api/v1/user/attribution-export');

      return {
        attributions: response.attributions,
        contributions: response.contributions.map(item => ({
          recipeId: item.recipe_id,
          title: item.title,
          createdAt: new Date(item.created_at),
          totalImports: item.total_imports,
        })),
        achievements: response.achievements,
      };
    } catch (error) {
      console.error('Failed to export attribution data:', error);
      throw new Error('Failed to export attribution data');
    }
  }
}