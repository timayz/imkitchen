// Copyright (c) 2025 TikTok Pte. Ltd.
// Licensed under the Apache License Version 2.0 that can be found in the
// LICENSE file in the root directory of this source tree.
package app.imkitchen.android

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import com.tiktok.sparkling.Sparkling
import com.tiktok.sparkling.SparklingContext
import com.tiktok.sparkling.method.registry.core.utils.JsonUtils

class SplashActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // `singleTask` + `finish()` below: once the app is running this
        // activity is gone from the task, so an icon or notification tap
        // recreates it *on top* of the live containers. Opening main again
        // from there would bury the cooking screen under a second kitchen;
        // the task is already in front, so just leave.
        if (!isTaskRoot && intent?.data == null) {
            finish()
            return
        }
        gotoSparklingPage(intent)
    }

    // `singleTask`: a link tapped while the app is open lands here instead
    // of in a second instance.
    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        gotoSparklingPage(intent)
    }

    private fun gotoSparklingPage(intent: Intent?) {
        val initData = mapOf<Any, Any>()
        val initialData: String = JsonUtils.toJson(initData)

        val context = SparklingContext()
        val resetId = intent?.data?.let(::passwordResetId)
        if (resetId != null) {
            // The emailed link: straight to the new-password screen. The page
            // reads `id` from `__globalProps.pageQuery` (see the host router).
            val scheme = Uri.parse(PAGE_SCHEME).buildUpon()
                .appendQueryParameter("bundle", "reset.lynx.bundle")
                .appendQueryParameter("hide_nav_bar", "1")
                .appendQueryParameter("hide_loading", "1")
                .appendQueryParameter("screen_orientation", "portrait")
                .appendQueryParameter("id", resetId)
                .build()
            context.scheme = scheme.toString()
            context.runtimeInfo[SparklingHostRouterDepend.PAGE_QUERY] =
                SparklingHostRouterDepend.pageQuery(scheme)
        } else {
            context.scheme = "$PAGE_SCHEME?bundle=main.lynx.bundle&hide_nav_bar=1&screen_orientation=portrait"
        }
        context.withInitData("{ \"initial_data\":$initialData}")
        Sparkling.build(this, context).navigate()
        finish()
    }

    companion object {
        private const val PAGE_SCHEME = "hybrid://lynxview_page"

        /**
         * The reset id from `https://imkitchen.app/reset-password/new/<id>`
         * or `imkitchen://reset-password/new/<id>`, else null.
         */
        fun passwordResetId(uri: Uri): String? {
            val segments = uri.pathSegments
            val onSite = (uri.scheme == "https" || uri.scheme == "http") &&
                segments.size == 3 && segments[0] == "reset-password" && segments[1] == "new"
            val inApp = uri.scheme == "imkitchen" && uri.host == "reset-password" &&
                segments.size == 2 && segments[0] == "new"
            return if (onSite || inApp) segments.last().takeIf { it.isNotBlank() } else null
        }
    }
}
