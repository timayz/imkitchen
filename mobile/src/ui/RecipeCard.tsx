import type { Summary } from '../lib/api/recipes.js'
import { course, minutes } from '../lib/course.js'
import { t } from '../lib/i18n/index.js'
import { RecipeImage } from './RecipeImage.js'
import './RecipeCard.css'

export interface RecipeCardProps {
  recipe: Summary
  showShared?: boolean
  onTap: () => void
}

export function RecipeCard({ recipe, showShared, onTap }: RecipeCardProps) {
  const c = course(recipe.recipe_type)
  return (
    <view className="rcard" bindtap={onTap}>
      <RecipeImage
        thumbnailUrl={recipe.thumbnail_url}
        blurPlaceholder={recipe.blur_placeholder}
        recipeType={recipe.recipe_type}
        size="thumb"
      />
      <view className="rcard__body">
        <text className="rcard__course" style={{ color: c.ink }}>
          {c.label.toUpperCase()}
        </text>
        <text className="rcard__name">{recipe.name}</text>
        <view className="row">
          <text className="rcard__meta">⏱ {minutes(recipe.total_time)}</text>
          {recipe.owner_name && <text className="rcard__meta">@{recipe.owner_name}</text>}
          {showShared && recipe.is_shared && <text className="rcard__shared">{t('recipes.shared')}</text>}
        </view>
      </view>
    </view>
  )
}
