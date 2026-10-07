package app.imkitchen.android

import android.annotation.SuppressLint
import android.content.Context
import android.webkit.WebView
import android.webkit.WebViewClient
import com.lynx.tasm.behavior.LynxContext
import com.lynx.tasm.behavior.LynxProp
import com.lynx.tasm.behavior.ui.LynxUI

/**
 * `<webview src="…">` for Lynx pages. The Lynx SDK bundled with Sparkling
 * 2.0.1 ships no webview element on Android, and the cooking screen frames
 * an imported recipe's origin page when that origin allows it.
 */
class LynxWebViewComponent(context: LynxContext?) : LynxUI<WebView>(context) {

  @SuppressLint("SetJavaScriptEnabled")
  override fun createView(context: Context): WebView {
    return WebView(context).apply {
      settings.javaScriptEnabled = true
      settings.domStorageEnabled = true
      settings.loadWithOverviewMode = true
      settings.useWideViewPort = true
      // Keep navigation inside the frame instead of bouncing to the browser.
      webViewClient = WebViewClient()
    }
  }

  @LynxProp(name = "src")
  fun setSrc(value: String?) {
    if (!value.isNullOrBlank() && value != mView.url) {
      mView.loadUrl(value)
    }
  }

  override fun destroy() {
    mView.stopLoading()
    mView.destroy()
    super.destroy()
  }
}
