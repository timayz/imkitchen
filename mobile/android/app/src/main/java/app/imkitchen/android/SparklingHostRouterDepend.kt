// Copyright (c) 2025 TikTok Pte. Ltd.
// Licensed under the Apache License Version 2.0 that can be found in the
// LICENSE file in the root directory of this source tree.
package app.imkitchen.android

import android.content.Context
import android.content.Intent
import android.net.Uri
import com.tiktok.sparkling.Sparkling
import com.tiktok.sparkling.SparklingContext
import com.tiktok.sparkling.hybridkit.service.HybridActivityStackManager
import com.tiktok.sparkling.method.registry.core.BridgePlatformType
import com.tiktok.sparkling.method.registry.core.IBridgeContext
import com.tiktok.sparkling.method.router.utils.IHostRouterDepend

class SparklingHostRouterDepend : IHostRouterDepend {

    override fun openScheme(
        bridgeContext: IBridgeContext?,
        scheme: String,
        extraParams: Map<String, Any>,
        platformType: BridgePlatformType,
        context: Context?
    ): Boolean {
        val uri = Uri.parse(scheme)

        // http(s) links (recipe origins) leave the app.
        if (uri.scheme == "http" || uri.scheme == "https") {
            val target = context ?: HybridActivityStackManager.getTopActivity() ?: return false
            val intent = Intent(Intent.ACTION_VIEW, uri).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            target.startActivity(intent)
            return true
        }

        val sparklingContext = SparklingContext()
        sparklingContext.scheme = scheme
        // Sparkling 2.0.1 never exposes custom scheme params to the page
        // (`queryItems` only carries `containerInitTime`), so hand them over
        // through the runtime info, which is merged into `__globalProps`.
        sparklingContext.runtimeInfo[PAGE_QUERY] = pageQuery(uri)
        context?.let { Sparkling.Companion.build(it, sparklingContext).navigate() }
        return true
    }

    override fun closeView(
        bridgeContext: IBridgeContext?,
        type: BridgePlatformType,
        containerID: String?,
        animated: Boolean?
    ): Boolean {
        val ownerActivity = bridgeContext?.ownerActivity
        if (ownerActivity != null) {
            ownerActivity.finish()
        } else {
            HybridActivityStackManager.getTopActivity()?.finish()
        }
        return true
    }

    companion object {
        /** `lynx.__globalProps.pageQuery` in the opened page. */
        const val PAGE_QUERY = "pageQuery"

        /** Every query parameter of the scheme, container ones included. */
        fun pageQuery(uri: Uri): Map<String, String> =
            uri.queryParameterNames.associateWith { uri.getQueryParameter(it) ?: "" }
    }
}
