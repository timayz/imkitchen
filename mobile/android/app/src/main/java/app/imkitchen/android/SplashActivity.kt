// Copyright (c) 2025 TikTok Pte. Ltd.
// Licensed under the Apache License Version 2.0 that can be found in the
// LICENSE file in the root directory of this source tree.
package app.imkitchen.android

import android.content.res.Configuration
import android.net.Uri
import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import com.tiktok.sparkling.Sparkling
import com.tiktok.sparkling.SparklingContext

class SplashActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // `singleTask` + `finish()` below: once the app is running this
        // activity is gone from the task, so an icon or notification tap
        // recreates it *on top* of the live containers. Opening main again
        // from there would bury the screen the user was on under a second
        // one; the task is already in front, so just leave.
        if (!isTaskRoot) {
            finish()
            return
        }
        openMain()
        finish()
    }

    private fun openMain() {
        // `--color-paper` in src/styles/tokens.css: what the container shows
        // until the page has rendered. Pushed pages get it from src/lib/nav.ts.
        val night = resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK ==
            Configuration.UI_MODE_NIGHT_YES
        val paper = if (night) "#1b140c" else "#fbf5e9"
        val scheme = Uri.parse(MAIN_SCHEME).buildUpon()
            .appendQueryParameter("container_bg_color", paper)
            .appendQueryParameter("loading_bg_color", paper)
            .build()
        val context = SparklingContext()
        context.scheme = scheme.toString()
        context.runtimeInfo[SparklingHostRouterDepend.PAGE_QUERY] =
            SparklingHostRouterDepend.pageQuery(scheme)
        context.withInitData("{ \"initial_data\":{}}")
        Sparkling.build(this, context).navigate()
    }

    private companion object {
        const val MAIN_SCHEME =
            "hybrid://lynxview_page?bundle=main.lynx.bundle&hide_nav_bar=1&hide_loading=1&screen_orientation=portrait"
    }
}
