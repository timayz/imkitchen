export interface ComplexityTrendData {
  week: string;
  averageComplexity: number;
  prepTimeMinutes: number;
  recipeCount: number;
}

export interface WeeklyAnalysisData {
  weekNumber: number;
  weekStartDate: string;
  varietyScore: number;
  patternAdherence: number;
  favoritesUsed: number;
  totalMeals: number;
}

export interface RotationAnalytics {
  varietyScore: number;
  rotationEfficiency: number;
  weeksAnalyzed: number;
  complexityDistribution: Record<string, number>;
  complexityTrends: ComplexityTrendData[];
  favoritesFrequency: Record<string, number>;
  favoritesImpact: number;
  weeklyPatterns: WeeklyAnalysisData[];
  calculatedAt: string;
}

export interface RotationResetOptions {
  confirmReset: boolean;
  preservePatterns?: boolean;
  preserveFavorites?: boolean;
}

export interface RotationDebugLog {
  id: string;
  timestamp: string;
  decisionType: 'recipe_selection' | 'constraint_violation' | 'fallback_triggered';
  recipeId?: string;
  recipeName?: string;
  constraintViolated?: string;
  fallbackReason?: string;
  algorithmVersion: string;
}

export interface AnalyticsExportOptions {
  format: 'json' | 'csv';
  dateRange?: {
    startDate: string;
    endDate: string;
  };
  includeDebugLogs?: boolean;
}