import { useCallback, useEffect, useState } from '@lynx-js/react'

import '../../styles/base.css'
import './App.css'
import { errorMessage } from '../../lib/api/client.js'
import { type Dish, type DishRecipe, addDishToList, getDishes } from '../../lib/api/dishes.js'
import { course, minutes } from '../../lib/course.js'
import { t } from '../../lib/i18n/index.js'
import { back, push } from '../../lib/nav.js'
import { useResource } from '../../lib/use-resource.js'
import { Button } from '../../ui/Button.js'
import { Spinner } from '../../ui/Spinner.js'

/**
 * The user's dishes: composed meals that generation uses first and that
 * can be added to the list by hand. `dishes.lynx.bundle`.
 */
export function App() {
  const dishes = useResource<Dish[]>('dishes', getDishes, [])
  const [busy, setBusy] = useState<string | null>(null)
  const [notice, setNotice] = useState<{ kind: 'ok' | 'error'; text: string } | null>(null)

  // Returning from the composer: show what was saved.
  useEffect(() => {
    const emitter = lynx.getJSModule('GlobalEventEmitter')
    const onShow = () => dishes.refresh()
    emitter.addListener('onShow', onShow)
    return () => emitter.removeListener('onShow', onShow)
  }, [dishes.refresh])

  const add = useCallback(
    async (dish: Dish) => {
      if (busy) return
      setBusy(dish.id)
      setNotice(null)
      try {
        await addDishToList(dish.id)
        setNotice({ kind: 'ok', text: t('dishes.added', { name: dish.name }) })
        dishes.refresh()
      } catch (err) {
        setNotice({ kind: 'error', text: errorMessage(err) })
      } finally {
        setBusy(null)
      }
    },
    [busy, dishes.refresh]
  )

  const list = dishes.data

  return (
    <view className="screen">
      <view className="dishes__bar">
        <view className="dishes__back" bindtap={back}>
          <text className="dishes__back-glyph">←</text>
        </view>
        <view className="dishes__bar-text">
          <text className="h2">{t('dishes.title')}</text>
          <text className="muted">{list ? t('dishes.count', { n: list.length }) : t('tabs.kitchen')}</text>
        </view>
        <view className="dishes__add" bindtap={() => void push('dish-edit')}>
          <text className="dishes__add-glyph">+</text>
        </view>
      </view>

      <scroll-view className="dishes__scroll" scroll-orientation="vertical">
        <view className="content">
          {notice && (
            <view className={notice.kind === 'ok' ? 'dishes__notice dishes__notice--ok' : 'dishes__notice'}>
              <text
                className={notice.kind === 'ok' ? 'dishes__notice-text dishes__notice-text--ok' : 'dishes__notice-text'}
              >
                {notice.text}
              </text>
            </view>
          )}
          <text className="body">{t('dishes.intro')}</text>

          {!list && dishes.error && (
            <view className="card">
              <text className="error">{dishes.error}</text>
              <Button label={t('common.retry')} onTap={dishes.refresh} variant="secondary" />
            </view>
          )}
          {!list && !dishes.error && (
            <view className="content--center">
              <Spinner size="lg" />
            </view>
          )}

          {list && list.length === 0 && (
            <view className="card dishes__empty">
              <text className="dishes__empty-emoji">🍽️</text>
              <text className="dishes__empty-title">{t('dishes.empty_title')}</text>
              <text className="body dishes__center">{t('dishes.empty_hint')}</text>
              <Button label={t('dishes.compose')} onTap={() => void push('dish-edit')} variant="ink" block />
            </view>
          )}

          {list?.map((dish) => (
            <view key={dish.id} className="card">
              <view className="dishes__head">
                <view className="dishes__head-text">
                  <text className="dishes__name">{dish.name}</text>
                  <text className="muted">
                    {t('dishes.recipes', { n: dish.recipes.length })}
                    {dish.total_time > 0 ? ` · ${minutes(dish.total_time)}` : ''}
                  </text>
                </view>
                <view className="dishes__edit" bindtap={() => void push('dish-edit', { id: dish.id })}>
                  <text className="dishes__edit-text">{t('dishes.edit')}</text>
                </view>
              </view>
              {dish.recipes.length === 0 ? (
                <text className="muted">{t('dishes.all_deleted')}</text>
              ) : (
                <view className="dishes__recipes">
                  {dish.recipes.map((recipe) => (
                    <Row key={recipe.id} recipe={recipe} />
                  ))}
                </view>
              )}
              <Button
                label={dish.in_list ? `✓ ${t('dishes.in_list')}` : t('dishes.add')}
                onTap={() => void add(dish)}
                variant={dish.in_list ? 'secondary' : 'primary'}
                disabled={busy !== null || dish.recipes.length === 0}
                block
              />
            </view>
          ))}
        </view>
      </scroll-view>
    </view>
  )
}

function Row({ recipe }: { recipe: DishRecipe }) {
  const c = course(recipe.recipe_type)
  const main = recipe.recipe_type === 'MainCourse'
  return (
    <view className="dishes__row" bindtap={() => void push('recipe', { id: recipe.id })}>
      <view className="dishes__tile" style={{ backgroundColor: c.soft }}>
        <text className="dishes__tile-emoji">{c.emoji}</text>
      </view>
      <text className={main ? 'dishes__row-name dishes__row-name--main' : 'dishes__row-name'}>{recipe.name}</text>
      <text className="dishes__row-time">{minutes(recipe.total_time)}</text>
    </view>
  )
}
