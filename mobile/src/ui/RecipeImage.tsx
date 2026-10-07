import { API_BASE_URL } from '../lib/config.js'
import { course } from '../lib/course.js'
import type { RecipeType } from '../lib/api/recipe.js'
import './RecipeImage.css'

export interface RecipeImageProps {
  thumbnailUrl: string | null
  blurPlaceholder: string | null
  recipeType: RecipeType
  /** `cover`: full-bleed, square-cornered (the detail screen's hero). */
  size: 'thumb' | 'hero' | 'cover'
}

/** Thumbnail with the course emoji as fallback (the web's lazy_thumbnail). */
export function RecipeImage({ thumbnailUrl, blurPlaceholder, recipeType, size }: RecipeImageProps) {
  const c = course(recipeType)
  const cls = `rimg rimg--${size}`
  return (
    <view className={cls} style={{ backgroundColor: c.soft }}>
      {thumbnailUrl ? (
        <>
          {blurPlaceholder && <image className="rimg__img" src={blurPlaceholder} mode="aspectFill" />}
          <image className="rimg__img" src={`${API_BASE_URL}${thumbnailUrl}`} mode="aspectFill" />
        </>
      ) : (
        <text className={size === 'thumb' ? 'rimg__emoji' : 'rimg__emoji rimg__emoji--hero'}>{c.emoji}</text>
      )}
    </view>
  )
}
