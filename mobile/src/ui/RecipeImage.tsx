import { API_BASE_URL } from '../lib/config.js'
import { course } from '../lib/course.js'
import type { RecipeType } from '../lib/api/recipe.js'
import './RecipeImage.css'

export interface RecipeImageProps {
  thumbnailUrl: string | null
  blurPlaceholder: string | null
  recipeType: RecipeType
  size: 'thumb' | 'hero'
}

/** Thumbnail with the course emoji as fallback (the web's lazy_thumbnail). */
export function RecipeImage({ thumbnailUrl, blurPlaceholder, recipeType, size }: RecipeImageProps) {
  const c = course(recipeType)
  const cls = size === 'hero' ? 'rimg rimg--hero' : 'rimg rimg--thumb'
  return (
    <view className={cls} style={{ backgroundColor: c.soft }}>
      {thumbnailUrl ? (
        <>
          {blurPlaceholder && <image className="rimg__img" src={blurPlaceholder} mode="aspectFill" />}
          <image className="rimg__img" src={`${API_BASE_URL}${thumbnailUrl}`} mode="aspectFill" />
        </>
      ) : (
        <text className={size === 'hero' ? 'rimg__emoji rimg__emoji--hero' : 'rimg__emoji'}>{c.emoji}</text>
      )}
    </view>
  )
}
