import { useCallback, useState } from '@lynx-js/react'

import '../../styles/base.css'
import { ApiError } from '../../lib/api/client.js'
import { type RecipeInput, importRecipes } from '../../lib/api/recipe-edit.js'
import { waitForRecipe } from '../../lib/api/recipes.js'
import { t } from '../../lib/i18n/index.js'
import { back } from '../../lib/nav.js'
import { Button } from '../../ui/Button.js'
import { TextField } from '../../ui/TextField.js'

type Result = { kind: 'ok'; imported: number; errors: { name: string; error: string }[] } | { kind: 'error'; text: string }

/**
 * Import up to 20 recipes from JSON (the same shape the web import takes):
 * `recipe-import.lynx.bundle`.
 */
export function App() {
  const [text, setText] = useState('')
  const [busy, setBusy] = useState(false)
  const [result, setResult] = useState<Result | null>(null)

  const run = useCallback(async () => {
    if (busy) return
    let recipes: RecipeInput[]
    try {
      const parsed: unknown = JSON.parse(text)
      recipes = Array.isArray(parsed) ? (parsed as RecipeInput[]) : [parsed as RecipeInput]
    } catch {
      setResult({ kind: 'error', text: t('import.invalid_json') })
      return
    }
    if (recipes.length === 0 || recipes.length > 20) {
      setResult({ kind: 'error', text: t('import.count') })
      return
    }
    setBusy(true)
    setResult(null)
    try {
      const outcome = await importRecipes(recipes)
      if (outcome.last_id) await waitForRecipe(outcome.last_id, true)
      setResult({ kind: 'ok', imported: recipes.length - outcome.errors.length, errors: outcome.errors })
    } catch (err) {
      setResult({ kind: 'error', text: err instanceof ApiError ? err.message : t('error.network') })
    } finally {
      setBusy(false)
    }
  }, [busy, text])

  return (
    <scroll-view className="screen" scroll-orientation="vertical">
      <view className="content">
        <view className="row" style={{ gap: '12px' }}>
          <view className="cook__close" bindtap={back}>
            <text className="cook__close-text">←</text>
          </view>
          <text className="h2">{t('import.title')}</text>
        </view>
        <text className="body">{t('import.hint')}</text>
        <TextField label={t('import.json')} value="" onChange={setText} placeholder='[{"recipe_type":"MainCourse","name":"…"}]' maxlength={200000} />
        <Button label={t('import.submit')} onTap={run} disabled={busy || text.trim() === ''} block />
        {result?.kind === 'error' && <text className="error">{result.text}</text>}
        {result?.kind === 'ok' && (
          <view className="card">
            <text className="success">{t('import.done', { n: result.imported })}</text>
            {result.errors.map((e, i) => (
              <text key={i} className="error">
                {e.name}: {e.error}
              </text>
            ))}
            <Button label={t('common.close')} onTap={back} variant="secondary" />
          </view>
        )}
      </view>
    </scroll-view>
  )
}
